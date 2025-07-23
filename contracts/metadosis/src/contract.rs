use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{
    Config, DailyRunInfo, RunInfo, RunType, CONFIG, CREATOR, DAILY_RUNS, DAILY_RUNS_INFO,
    TRIBUTES_DISTRIBUTION,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Decimal, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
    SubMsg, Uint128, WasmMsg,
};
use outbe_utils::date;
use outbe_utils::date::WorldwideDay;
use price_oracle::types::DayType;
use std::collections::HashSet;
use tribute::query::FullTributeData;

const CONTRACT_NAME: &str = "outbe.net:metadosis";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // use info.sender if None is passed
    let creator: &str = match msg.creator.as_deref() {
        Some(creator) => creator,
        None => info.sender.as_str(),
    };

    CREATOR.initialize_owner(deps.storage, deps.api, Some(creator))?;

    CONFIG.save(
        deps.storage,
        &Config {
            vector: msg.vector,
            tribute: msg.tribute,
            nod: msg.nod,
            token_allocator: msg.token_allocator,
            price_oracle: msg.price_oracle,
            random_oracle: msg.random_oracle,
            deficit: msg.deficit,
        },
    )?;

    Ok(Response::default()
        .add_attribute("action", "metadosis::instantiate")
        .add_event(Event::new("metadosis::instantiate").add_attribute("creator", creator)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Execute { run_date } => execute_run(deps, env, info, run_date),
        ExecuteMsg::BurnAll {} => execute_burn_all(deps, &env, &info),
    }
}

fn execute_run(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    run_date: Option<WorldwideDay>,
) -> Result<Response, ContractError> {
    // todo verify ownership to run raffle

    let execution_date = run_date.unwrap_or(date::normalize_to_date(&env.block.time));
    println!("execution date time = {}", execution_date);

    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config.tribute.ok_or(ContractError::NotInitialized {})?;
    let token_allocator_address = config
        .token_allocator
        .ok_or(ContractError::NotInitialized {})?;
    let vector_address = config.vector.ok_or(ContractError::NotInitialized {})?;
    let price_oracle_address = config
        .price_oracle
        .ok_or(ContractError::NotInitialized {})?;
    let nod_address = config.nod.ok_or(ContractError::NotInitialized {})?;

    let exchange_rate: price_oracle::types::TokenPairPrice = deps.querier.query_wasm_smart(
        &price_oracle_address,
        &price_oracle::query::QueryMsg::GetPrice {},
    )?;

    let run_today = DAILY_RUNS.may_load(deps.storage, execution_date)?;
    let run_today = run_today.unwrap_or_default();
    let run_today = run_today + 1;

    if run_today == 1 {
        schedule_executions(
            deps.branch(),
            tribute_address.clone(),
            token_allocator_address,
            execution_date,
            config.deficit,
            exchange_rate.day_type,
        )?;
    }

    DAILY_RUNS.save(deps.storage, execution_date, &run_today)?;
    // execute metadosis or touch

    execute_lysis_or_touch(
        deps,
        execution_date,
        run_today,
        tribute_address,
        nod_address,
        vector_address,
        exchange_rate.price,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_lysis_or_touch(
    deps: DepsMut,
    execution_date: WorldwideDay,
    run_today: usize,
    tribute_address: Addr,
    nod_address: Addr,
    vector_address: Addr,
    exchange_rate: Decimal,
) -> Result<Response, ContractError> {
    let info = DAILY_RUNS_INFO.load(deps.storage, execution_date)?;

    if run_today > info.number_of_runs {
        return Err(ContractError::BadRunConfiguration {});
    }

    let run_info = info.data[run_today - 1].clone();

    let response = match run_info.run_type {
        RunType::Lysis => {
            // use already distributed tokens
            let mut tributes_in_first_tier: Vec<String> = vec![];
            let mut j: usize = 0;
            loop {
                let key = format!("{}_{}_{}", execution_date, run_today, j);
                let tribute_id = TRIBUTES_DISTRIBUTION.may_load(deps.storage, &key)?;
                match tribute_id {
                    None => {
                        break;
                    }
                    Some(id) => tributes_in_first_tier.push(id),
                }
                j += 1;
            }

            do_lysis_tier(
                deps,
                tributes_in_first_tier,
                run_today,
                tribute_address.clone(),
                vector_address,
                nod_address,
                exchange_rate,
            )
        }
        RunType::Touch => {
            // TODO add tributes query that were not selected for metadosis
            let all_tributes: tribute::query::DailyTributesResponse =
                deps.querier.query_wasm_smart(
                    &tribute_address,
                    &tribute::query::QueryMsg::DailyTributes {
                        date: execution_date,
                    },
                )?;
            let tributes_for_touch = all_tributes.tributes;

            execute_touch(
                tributes_for_touch,
                run_info.pool_allocation,
                run_today,
                nod_address,
                exchange_rate,
            )
        }
    };

    if run_today == info.number_of_runs {
        Ok(response?.add_submessage(SubMsg::new(WasmMsg::Execute {
            contract_addr: tribute_address.to_string(),
            msg: to_json_binary(&tribute::msg::ExecuteMsg::BurnAll {})?,
            funds: vec![],
        })))
    } else {
        response
    }
}

#[allow(clippy::too_many_arguments)]
fn do_lysis_tier(
    deps: DepsMut,
    tributes_in_current_raffle: Vec<String>,
    run_today: usize,
    tribute_address: Addr,
    vector_address: Addr,
    nod_address: Addr,
    exchange_rate: Decimal,
) -> Result<Response, ContractError> {
    // mint nod
    let mut messages: Vec<SubMsg> = vec![];
    let tributes_count = tributes_in_current_raffle.len();
    println!("Tributes in current run {}: {}", run_today, tributes_count);

    let tribute_info: tribute::query::TributeContractInfoResponse = deps
        .querier
        .query_wasm_smart(&tribute_address, &tribute::query::QueryMsg::ContractInfo {})?;
    let tribute_info = tribute_info.collection_config;

    let vector_info: vector::query::AllVectorsResponse = deps
        .querier
        .query_wasm_smart(&vector_address, &vector::query::QueryMsg::Vectors {})?;
    let vector = vector_info
        .vectors
        .iter()
        .find(|v| usize::from(v.vector_id) == run_today)
        .ok_or(ContractError::BadRunConfiguration {})?;

    // TODO shuffle here like the following
    // let seed: randao::query::QuerySeedResponse = deps
    //     .querier
    //     .query_wasm_smart(&rand_address, &randao::query::QueryMsg::Seed {})?;
    //
    // let mut rnd = ChaCha8Rng::seed_from_u64(123);
    // // Simulate 20 candidates (IDs 1 to 20)
    // let mut candidates: Vec<u32> = (1..=20).collect();
    //
    // // Shuffle and pick 10 winners
    // candidates.shuffle(&mut rnd);
    // let winners = &candidates[..10];

    for tribute_id in tributes_in_current_raffle {
        let tribute: tribute::query::TributeInfoResponse = deps.querier.query_wasm_smart(
            &tribute_address,
            &tribute::query::QueryMsg::NftInfo {
                token_id: tribute_id.clone(),
            },
        )?;
        let nod_id = format!("{}_{}", tribute_id, run_today);
        let floor_price = exchange_rate
            * (Decimal::one() + Decimal::from_atomics(vector.vector_rate, 3).unwrap());
        println!("Nod id creation = {}", nod_id);
        let nod_mint = WasmMsg::Execute {
            contract_addr: nod_address.to_string(),
            msg: to_json_binary(&nod::msg::ExecuteMsg::Submit {
                token_id: nod_id.clone(),
                owner: tribute.owner.to_string(),
                extension: Box::new(nod::msg::SubmitExtension {
                    entity: nod::msg::NodEntity {
                        nod_id,
                        settlement_currency: tribute.extension.settlement_currency.clone(),
                        symbolic_rate: tribute_info.symbolic_rate,
                        floor_rate: vector.vector_rate,
                        nominal_price_minor: tribute.extension.tribute_price_minor,
                        issuance_price_minor: exchange_rate,
                        gratis_load_minor: tribute.extension.symbolic_load,
                        floor_price_minor: floor_price,
                        state: nod::types::State::Issued,
                        owner: tribute.owner.to_string(),
                        qualified_at: None,
                    },
                    created_at: None,
                }),
            })?,
            funds: vec![],
        };
        messages.push(SubMsg::new(nod_mint));
    }

    Ok(Response::new()
        .add_attribute("action", "metadosis::lysis")
        .add_event(
            Event::new("metadosis::lysis")
                .add_attribute("run", run_today.to_string())
                .add_attribute("tributes_count", format!("{}", tributes_count)),
        )
        .add_submessages(messages))
}

fn schedule_executions(
    deps: DepsMut,
    tribute_address: Addr,
    token_allocator_address: Addr,
    execution_date: u64,
    deficit: Decimal,
    day_type: DayType,
) -> Result<(), ContractError> {
    let (total_allocation, allocation_per_tier) =
        calc_allocation(deps.as_ref(), token_allocator_address)?;
    println!(
        "total_allocation = {}, allocation_per_pool = {}, ",
        total_allocation, allocation_per_tier
    );

    let all_tributes: tribute::query::DailyTributesResponse = deps.querier.query_wasm_smart(
        &tribute_address,
        &tribute::query::QueryMsg::DailyTributes {
            date: execution_date,
        },
    )?;
    let all_tributes = all_tributes.tributes;
    let all_tributes_len = all_tributes.len();
    println!(
        "Metadosis {} tributes distribution for date ",
        all_tributes_len
    );

    let mut run_data: Vec<RunInfo> = Vec::with_capacity(24);

    let total_interest = all_tributes
        .iter()
        .fold(Uint128::zero(), |acc, t| acc + t.data.symbolic_load);

    if day_type == DayType::Green {
        // distribute tokens

        let total_deficit = calc_total_deficit(total_allocation, total_interest, deficit);

        // let total_lysis_limit = total_interest - total_deficit;

        // TODO calc deficit per pool
        // let lysis_limit = total_lysis_limit / Uint128::new(23);
        let lysis_deficit = total_deficit / Uint128::new(23);
        let lysis_capacity = allocation_per_tier + lysis_deficit;

        println!("total_allocation = {}", total_allocation);
        println!("total_interest = {}", total_interest);
        println!("allocation_per_pool = {}", allocation_per_tier);
        println!("total_deficit = {}", total_deficit);
        println!("pool_deficit = {}", lysis_deficit);
        println!("pool_capacity = {}", lysis_capacity);

        let mut distributed_tributes: HashSet<String> = HashSet::new();

        let mut lysis_pools: Vec<Vec<String>> = Vec::with_capacity(23);
        let mut pool_index: u16 = 23;
        while pool_index > 0 {
            let mut pool_tributes: Vec<String> = vec![];
            let mut allocated_in_pool = Uint128::zero();
            for tribute in all_tributes.clone() {
                if allocated_in_pool >= lysis_capacity {
                    break;
                }
                if !distributed_tributes.contains(&tribute.token_id) {
                    if allocated_in_pool + tribute.data.symbolic_load > lysis_capacity {
                        continue;
                    }
                    allocated_in_pool += tribute.data.symbolic_load;
                    pool_tributes.push(tribute.token_id.clone());
                    distributed_tributes.insert(tribute.token_id.clone());
                }
            }
            println!(
                "Distributed in pool {:?}: {:?} tributes",
                pool_index,
                pool_tributes.len()
            );

            run_data.push(RunInfo {
                vector_index: pool_index,
                run_type: RunType::Lysis,
                total_allocation,
                pool_allocation: allocation_per_tier,
                total_deficit,
                pool_deficit: lysis_deficit,
                pool_capacity: lysis_capacity,
                assigned_tributes: pool_tributes.len(),
                assigned_tributes_sum: allocated_in_pool,
            });

            lysis_pools.push(pool_tributes);
            pool_index -= 1;
        }

        for (i, pool) in lysis_pools.iter().enumerate() {
            for (j, tribute_id) in pool.iter().enumerate() {
                // todo define map key for such struct
                // NB: i starts from 1 because first run starts from 1
                let key = format!("{}_{}_{}", execution_date, i + 1, j);
                TRIBUTES_DISTRIBUTION.save(deps.storage, &key, tribute_id)?;
                println!("added tribute {} in pool {}", tribute_id, key);
            }
        }
    }

    run_data.push(RunInfo {
        vector_index: 0,
        run_type: RunType::Touch,
        total_allocation,
        pool_allocation: allocation_per_tier,
        total_deficit: Uint128::zero(),
        pool_deficit: Uint128::zero(),
        pool_capacity: allocation_per_tier,
        assigned_tributes: all_tributes_len,
        assigned_tributes_sum: total_interest,
    });

    let runs_num = run_data.len();
    // save history of the last distribution
    DAILY_RUNS_INFO.save(
        deps.storage,
        execution_date,
        &DailyRunInfo {
            data: run_data,
            number_of_runs: runs_num,
        },
    )?;

    Ok(())
}

fn calc_total_deficit(
    total_allocation: Uint128,
    total_interest: Uint128,
    deficit_percent: Decimal,
) -> Uint128 {
    let mut total_deficit =
        (deficit_percent * Decimal::from_atomics(total_allocation, 0).unwrap()).to_uint_floor();

    if total_interest > total_allocation && total_interest - total_allocation > total_deficit {
        total_deficit = total_interest - total_allocation;
    }
    total_deficit
}

fn execute_touch(
    tributes: Vec<FullTributeData>,
    allocation: Uint128,
    run_today: usize,
    nod_address: Addr,
    exchange_rate: Decimal,
) -> Result<Response, ContractError> {
    if tributes.is_empty() {
        return Ok(Response::new()
            .add_attribute("action", "metadosis::touch")
            .add_event(Event::new("metadosis::touch").add_attribute("touch", "no-data")));
    }

    // todo implement random. Now first tribute will win.
    // todo implement query gold price
    let winner = tributes.last().unwrap();

    let mut messages: Vec<SubMsg> = vec![];
    let nod_id = format!("{}_{}", winner.token_id, run_today);
    println!("Nod id creation = {}", nod_id);
    // todo create Gratis instead of Node
    let nod_mint = WasmMsg::Execute {
        contract_addr: nod_address.to_string(),
        msg: to_json_binary(&nod::msg::ExecuteMsg::Submit {
            token_id: nod_id.clone(),
            owner: winner.owner.to_string(),
            extension: Box::new(nod::msg::SubmitExtension {
                entity: nod::msg::NodEntity {
                    nod_id,
                    settlement_currency: winner.data.settlement_currency.clone(),
                    symbolic_rate: winner.data.tribute_price_minor,
                    floor_rate: Uint128::zero(),
                    nominal_price_minor: winner.data.tribute_price_minor,
                    issuance_price_minor: exchange_rate,
                    gratis_load_minor: allocation,
                    floor_price_minor: exchange_rate,
                    state: nod::types::State::Issued,
                    owner: winner.owner.to_string(),
                    qualified_at: None,
                },
                created_at: None,
            }),
        })?,
        funds: vec![],
    };
    messages.push(SubMsg::new(nod_mint));

    Ok(Response::new()
        .add_attribute("action", "metadosis::touch")
        .add_event(Event::new("metadosis::touch").add_attribute("touch", run_today.to_string()))
        .add_submessages(messages))
}

fn execute_burn_all(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // TODO verify ownership
    // let token = config.nft_info.load(deps.storage, &token_id)?;
    // check_can_send(deps.as_ref(), env, info.sender.as_str(), &token)?;

    TRIBUTES_DISTRIBUTION.clear(deps.storage);
    DAILY_RUNS_INFO.clear(deps.storage);
    DAILY_RUNS.clear(deps.storage);

    Ok(Response::new()
        .add_attribute("action", "metadosis::burn_all")
        .add_event(
            Event::new("metadosis::burn_all").add_attribute("sender", info.sender.to_string()),
        ))
}

pub(crate) fn calc_allocation(
    deps: Deps,
    token_allocator_address: Addr,
) -> StdResult<(Uint128, Uint128)> {
    let allocation_per_block: token_allocator::types::TokenAllocatorData =
        deps.querier.query_wasm_smart(
            &token_allocator_address,
            &token_allocator::query::QueryMsg::GetData {},
        )?;

    // todo calc total_allocation based on blocks with exact values
    let total_allocation = Uint128::from(allocation_per_block.amount) * Uint128::new(24 * 60 * 12);
    let allocation_per_tier = total_allocation / Uint128::new(24);

    Ok((total_allocation, allocation_per_tier))
}
