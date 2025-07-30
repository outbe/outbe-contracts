use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
// use crate::prepare;
use crate::state::{
    Config, DailyRunHistory, DailyRunState, LysisInfo, MetadosisInfo, RunHistoryInfo, RunType,
    TouchInfo, CONFIG, CREATOR, DAILY_RUNS_HISTORY, DAILY_RUN_STATE, METADOSIS_INFO, WINNERS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Decimal, DepsMut, Env, Event, MessageInfo, Reply, Response, SubMsg, Uint128,
    WasmMsg,
};
// use cw_utils::ParseReplyError::SubMsgFailure;
// use cw_utils::{parse_execute_response_data, MsgExecuteContractResponse};
use outbe_utils::date;
use outbe_utils::date::WorldwideDay;
use rand::prelude::SliceRandom;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
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
        ExecuteMsg::Prepare { run_date } => execute_prepare(deps, env, info, run_date),
        ExecuteMsg::Execute { run_date } => execute_run(deps, env, info, run_date),
        ExecuteMsg::BurnAll {} => execute_burn_all(deps, &env, &info),
    }
}

/// A unique ID for tokens allocation callback
const ALLOCATE_NATIVE_TOKENS_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // Match on the ID of the reply to handle the correct one
    // match msg.id {
    //     ALLOCATE_NATIVE_TOKENS_REPLY_ID => handle_token_allocation_reply(deps, msg),
    //     _ => Err(ContractError::UnrecognizedReplyId { id: msg.id }),
    // }
    Ok(Response::default())
}

fn execute_prepare(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    run_date: Option<WorldwideDay>,
) -> Result<Response, ContractError> {
    // todo verify ownership to run metadosis

    let execution_date = get_execution_date(run_date, env)?;

    let config = CONFIG.load(deps.storage)?;
    let token_allocator_address = config
        .token_allocator
        .ok_or(ContractError::NotInitialized {})?;

    let wasm_msg = WasmMsg::Execute {
        contract_addr: token_allocator_address.to_string(),
        msg: to_json_binary(&token_allocator::msg::ExecuteMsg::AllocateTokens {
            date: execution_date,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(
            wasm_msg,
            ALLOCATE_NATIVE_TOKENS_REPLY_ID,
        ))
        .add_attribute("action", "metadosis::prepare")
        .add_event(
            Event::new("metadosis::prepare").add_attribute("date", execution_date.to_string()),
        ))
}

// fn handle_token_allocation_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
//     //todo verify caller is token_allocator_address
//
//     println!("handle_token_allocation_reply {:?}", msg);
//
//     // 1. Check the result of the submessage
//     let subcall_result = msg.result.into_result().map_err(SubMsgFailure)?;
//
//     // 2. Get the data from the successful reply
//     let data = subcall_result
//         .msg_responses
//         .first()
//         .ok_or(ContractError::NoDataInReply {})?;
//
//     // 3. Deserialize the data into your expected struct
//     let allocation_result: MsgExecuteContractResponse =
//         parse_execute_response_data(data.value.as_slice())?;
//
//     let allocation_result = allocation_result
//         .data
//         .ok_or(ContractError::NoDataInReply {})?;
//
//     let allocation_result: token_allocator::contract::AllocationResult =
//         from_json(allocation_result.as_slice())?;
//
//     prepare::prepare_executions(deps, allocation_result.allocation, allocation_result.day)?;
//
//     Ok(Response::new()
//         .add_attribute("action", "metadosis::handle_allocation_reply")
//         .add_event(
//             Event::new("metadosis::handle_allocation_reply")
//                 .add_attribute("date", allocation_result.day.to_string())
//                 .add_attribute("allocation", allocation_result.allocation.to_string()),
//         ))
// }

fn execute_run(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    run_date: Option<WorldwideDay>,
) -> Result<Response, ContractError> {
    // todo verify ownership to run metadosis

    let execution_date = get_execution_date(run_date, env)?;
    let config = CONFIG.load(deps.storage)?;

    let run_today = DAILY_RUN_STATE.may_load(deps.storage, execution_date)?;
    let mut run_today = run_today.unwrap_or(DailyRunState {
        number_of_runs: 0,
        last_tribute_id: None,
    });
    run_today.number_of_runs += 1;

    let info = METADOSIS_INFO
        .load(deps.storage, execution_date)
        .map_err(|_| ContractError::NotPrepared {})?;

    let mut clean_tributes = false;
    let result: Result<Response, ContractError> = match info {
        MetadosisInfo::LysisAndTouch {
            lysis_info,
            touch_info,
        } => {
            match run_today.number_of_runs {
                1..=23 => do_execute_lysis(deps, execution_date, lysis_info, run_today),
                24 => {
                    // execute touch
                    clean_tributes = true;
                    do_execute_touch(deps, execution_date, touch_info, run_today)
                }
                _ => return Err(ContractError::BadRunConfiguration {}),
            }
        }
        MetadosisInfo::Touch { touch_info } => {
            if run_today.number_of_runs > 1 {
                return Err(ContractError::BadRunConfiguration {});
            }
            clean_tributes = true;
            do_execute_touch(deps, execution_date, touch_info, run_today)
        }
    };

    let mut submessages: Vec<SubMsg> = vec![];
    if clean_tributes {
        let tribute_address = config.tribute.ok_or(ContractError::NotInitialized {})?;
        let submsg = SubMsg::new(WasmMsg::Execute {
            contract_addr: tribute_address.to_string(),
            msg: to_json_binary(&tribute::msg::ExecuteMsg::BurnForDay {
                date: execution_date,
            })?,
            funds: vec![],
        });
        submessages.push(submsg);
    }

    Ok(result?.add_submessages(submessages))
}

fn do_execute_lysis(
    deps: DepsMut,
    execution_date: WorldwideDay,
    lysis_info: LysisInfo,
    run_today: DailyRunState,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config.tribute.ok_or(ContractError::NotInitialized {})?;
    let nod_address = config.nod.ok_or(ContractError::NotInitialized {})?;
    let random_oracle_address = config
        .random_oracle
        .ok_or(ContractError::NotInitialized {})?;
    let price_oracle_address = config
        .price_oracle
        .ok_or(ContractError::NotInitialized {})?;
    let exchange_rate: price_oracle::types::TokenPairPrice = deps.querier.query_wasm_smart(
        &price_oracle_address,
        &price_oracle::query::QueryMsg::GetPrice {},
    )?;

    let tributes: tribute::query::DailyTributesResponse = deps.querier.query_wasm_smart(
        &tribute_address,
        &tribute::query::QueryMsg::DailyTributes {
            date: execution_date,
            query_order: None,
            limit: None,
            start_after: run_today.last_tribute_id,
        },
    )?;

    let all_tributes_count = tributes.tributes.len();
    println!(
        "All Fetched Tributes in current run {}: count = {}",
        run_today.number_of_runs, all_tributes_count
    );

    // fast exit when no tributes
    if all_tributes_count == 0 {
        return Ok(Response::new()
            .add_attribute("action", "metadosis::lysis")
            .add_event(
                Event::new("metadosis::lysis")
                    .add_attribute("run", run_today.number_of_runs.to_string())
                    .add_attribute("tributes_count", "0"),
            ));
    }

    let lysis_deficit = lysis_info
        .lysis_deficits
        .get(run_today.number_of_runs - 1)
        .ok_or(ContractError::BadRunConfiguration {})?;
    let lysis_capacity = lysis_info.lysis_limit + lysis_deficit;

    let mut allocated_tributes_sum = Uint128::zero();
    let mut allocated_tributes: Vec<FullTributeData> = vec![];
    for tribute in tributes.tributes {
        if allocated_tributes_sum + tribute.data.symbolic_load > lysis_capacity {
            break;
        }
        allocated_tributes_sum += tribute.data.symbolic_load;
        allocated_tributes.push(tribute);
    }

    // update state
    let last_tribute_id = allocated_tributes.last().map(|t| t.token_id.clone());
    DAILY_RUN_STATE.save(
        deps.storage,
        execution_date,
        &DailyRunState {
            number_of_runs: run_today.number_of_runs,
            last_tribute_id,
        },
    )?;

    let allocated_tributes_count = allocated_tributes.len();
    println!(
        "Tributes in current run {}: count = {}, sum = {}",
        run_today.number_of_runs, allocated_tributes_count, allocated_tributes_sum
    );

    let tribute_info: tribute::query::TributeContractInfoResponse = deps
        .querier
        .query_wasm_smart(&tribute_address, &tribute::query::QueryMsg::ContractInfo {})?;
    let tribute_info = tribute_info.collection_config;

    // Get vector rate for this run
    let vector_rate = lysis_info
        .vector_rates
        .get(run_today.number_of_runs - 1)
        .ok_or(ContractError::BadRunConfiguration {})?;
    let vector_rate_dec = Decimal::from_atomics(*vector_rate, 3).unwrap();

    // shuffle here like the following
    let seed: random_oracle::msg::SeedResponse = deps.querier.query_wasm_smart(
        &random_oracle_address,
        &random_oracle::msg::QueryMsg::RandomSeed {},
    )?;

    let mut rnd = ChaCha8Rng::seed_from_u64(seed.seed);

    // Shuffle and pick winners
    allocated_tributes.shuffle(&mut rnd);

    let mut winners: Vec<FullTributeData> = vec![];
    let mut winners_sum = Uint128::zero();
    for tribute in allocated_tributes {
        // todo here we need to give full win for last tribute in this lysis
        //  and take some allocation limit from the next lysis run (if any)
        if winners_sum + tribute.data.symbolic_load > lysis_info.lysis_limit {
            break;
        }
        WINNERS.save(deps.storage, tribute.token_id.clone(), &())?;
        winners_sum += tribute.data.symbolic_load;
        winners.push(tribute);
    }
    let winners_len = winners.len();

    let mut messages: Vec<SubMsg> = vec![];
    for tribute in winners {
        let nod_id = format!("{}_{}", tribute.token_id, run_today.number_of_runs);
        // todo check if we need to calc floor price at the moment of lysis or take from tribute
        let floor_price = exchange_rate.price * (Decimal::one() + vector_rate_dec);

        let nod_mint = WasmMsg::Execute {
            contract_addr: nod_address.to_string(),
            msg: to_json_binary(&nod::msg::ExecuteMsg::Submit {
                token_id: nod_id.clone(),
                owner: tribute.owner.to_string(),
                extension: Box::new(nod::msg::SubmitExtension {
                    entity: nod::msg::NodEntity {
                        nod_id,
                        settlement_currency: tribute.data.settlement_currency.clone(),
                        symbolic_rate: tribute_info.symbolic_rate,
                        floor_rate: *vector_rate,
                        nominal_price_minor: tribute.data.tribute_price_minor,
                        issuance_price_minor: exchange_rate.price,
                        gratis_load_minor: tribute.data.symbolic_load,
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

    let mut history = DAILY_RUNS_HISTORY
        .may_load(deps.storage, execution_date)?
        .unwrap_or(DailyRunHistory { data: vec![] });

    history.data.push(RunHistoryInfo {
        run_type: RunType::Lysis,
        vector_rate: Some(vector_rate_dec),
        pool_allocation: lysis_info.lysis_limit,
        pool_deficit: *lysis_deficit,
        pool_capacity: lysis_capacity,
        assigned_tributes: allocated_tributes_count,
        assigned_tributes_sum: allocated_tributes_sum,
        winner_tributes: winners_len,
        winner_tributes_sum: winners_sum,
    });
    DAILY_RUNS_HISTORY.save(deps.storage, execution_date, &history)?;

    Ok(Response::new()
        .add_attribute("action", "metadosis::lysis")
        .add_event(
            Event::new("metadosis::lysis")
                .add_attribute("run", run_today.number_of_runs.to_string())
                .add_attribute("tributes_count", format!("{}", allocated_tributes_count)),
        )
        .add_submessages(messages))
}

fn do_execute_touch(
    deps: DepsMut,
    execution_date: WorldwideDay,
    touch_info: TouchInfo,
    run_today: DailyRunState,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config.tribute.ok_or(ContractError::NotInitialized {})?;
    let nod_address = config.nod.ok_or(ContractError::NotInitialized {})?;
    let random_oracle_address = config
        .random_oracle
        .ok_or(ContractError::NotInitialized {})?;
    let price_oracle_address = config
        .price_oracle
        .ok_or(ContractError::NotInitialized {})?;
    let exchange_rate: price_oracle::types::TokenPairPrice = deps.querier.query_wasm_smart(
        &price_oracle_address,
        &price_oracle::query::QueryMsg::GetPrice {},
    )?;

    let tributes: tribute::query::DailyTributesResponse = deps.querier.query_wasm_smart(
        &tribute_address,
        &tribute::query::QueryMsg::DailyTributes {
            date: execution_date,
            query_order: None,
            limit: None,
            start_after: None,
        },
    )?;

    let mut allocated_tributes = tributes.tributes;
    let allocated_tributes_count = allocated_tributes.len();
    println!(
        "Tributes in current run {}: count = {}",
        run_today.number_of_runs, allocated_tributes_count
    );

    // fast exit when no tributes
    if allocated_tributes.is_empty() {
        return Ok(Response::new()
            .add_attribute("action", "metadosis::lysis")
            .add_event(
                Event::new("metadosis::lysis")
                    .add_attribute("run", run_today.number_of_runs.to_string())
                    .add_attribute("tributes_count", "0"),
            ));
    }

    // update state
    DAILY_RUN_STATE.save(deps.storage, execution_date, &run_today)?;

    // shuffle here like the following
    let seed: random_oracle::msg::SeedResponse = deps.querier.query_wasm_smart(
        &random_oracle_address,
        &random_oracle::msg::QueryMsg::RandomSeed {},
    )?;

    let mut rnd = ChaCha8Rng::seed_from_u64(seed.seed);

    // Shuffle and pick winners
    allocated_tributes.shuffle(&mut rnd);

    let mut winners: Vec<FullTributeData> = vec![];
    for tribute in allocated_tributes {
        if WINNERS.has(deps.storage, tribute.token_id.clone()) {
            continue;
        }
        WINNERS.save(deps.storage, tribute.token_id.clone(), &())?;
        winners.push(tribute);
        // todo track ignot price here
        break;
    }
    let winners_len = winners.len();

    let mut messages: Vec<SubMsg> = vec![];
    for tribute in winners {
        let nod_id = format!("{}_{}", tribute.token_id, run_today.number_of_runs);
        let nod_mint = WasmMsg::Execute {
            contract_addr: nod_address.to_string(),
            msg: to_json_binary(&nod::msg::ExecuteMsg::Submit {
                token_id: nod_id.clone(),
                owner: tribute.owner.to_string(),
                extension: Box::new(nod::msg::SubmitExtension {
                    entity: nod::msg::NodEntity {
                        nod_id,
                        settlement_currency: tribute.data.settlement_currency.clone(),
                        symbolic_rate: tribute.data.tribute_price_minor,
                        floor_rate: Uint128::zero(),
                        nominal_price_minor: tribute.data.tribute_price_minor,
                        issuance_price_minor: exchange_rate.price,
                        gratis_load_minor: touch_info.touch_limit, // todo mb 1 ignot ?
                        floor_price_minor: exchange_rate.price,
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

    let mut history = DAILY_RUNS_HISTORY
        .may_load(deps.storage, execution_date)?
        .unwrap_or(DailyRunHistory { data: vec![] });

    history.data.push(RunHistoryInfo {
        run_type: RunType::Lysis,
        vector_rate: None,
        pool_allocation: touch_info.touch_limit,
        pool_deficit: Uint128::zero(),
        pool_capacity: touch_info.touch_limit,
        assigned_tributes: allocated_tributes_count,
        assigned_tributes_sum: touch_info.touch_limit,
        winner_tributes: winners_len,
        winner_tributes_sum: touch_info.touch_limit,
    });
    DAILY_RUNS_HISTORY.save(deps.storage, execution_date, &history)?;

    Ok(Response::new()
        .add_attribute("action", "metadosis::touch")
        .add_event(
            Event::new("metadosis::touch")
                .add_attribute("run", run_today.number_of_runs.to_string()),
        )
        .add_submessages(messages))
}

fn get_execution_date(
    run_date: Option<WorldwideDay>,
    env: Env,
) -> Result<WorldwideDay, ContractError> {
    let execution_date = run_date.unwrap_or(date::normalize_to_date(&env.block.time));
    date::is_valid(&execution_date)?;
    println!("execution date = {}", execution_date);
    Ok(execution_date)
}

fn execute_burn_all(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // TODO verify ownership

    METADOSIS_INFO.clear(deps.storage);
    DAILY_RUN_STATE.clear(deps.storage);
    DAILY_RUNS_HISTORY.clear(deps.storage);
    WINNERS.clear(deps.storage);

    Ok(Response::new()
        .add_attribute("action", "metadosis::burn_all")
        .add_event(
            Event::new("metadosis::burn_all").add_attribute("sender", info.sender.to_string()),
        ))
}
