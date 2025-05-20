use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, CONFIG, CREATOR, DAILY_RAFFLE, TRIBUTES_DISTRIBUTION};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, DepsMut, Env, Event, MessageInfo, Response, SubMsg, Timestamp, Uint128,
    WasmMsg,
};
use cw20::Denom;
use std::collections::HashSet;

const CONTRACT_NAME: &str = "outbe.net:raffle";
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
        },
    )?;

    Ok(Response::default()
        .add_attribute("action", "raffle::instantiate")
        .add_event(Event::new("raffle::instantiate").add_attribute("creator", creator)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Raffle { raffle_date } => execute_raffle(deps, env, info, raffle_date),
    }
}

fn execute_raffle(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    raffle_date: Option<Timestamp>,
) -> Result<Response, ContractError> {
    let date_time = raffle_date.unwrap_or(env.block.time);
    let date = normalize_to_date(date_time).seconds();

    let raffle_run_today = DAILY_RAFFLE.may_load(deps.storage, date)?;
    let raffle_run_today = raffle_run_today.unwrap_or_default();
    let raffle_run_today = raffle_run_today + 1;

    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config.tribute.ok_or(ContractError::NotInitialized {})?;
    let token_allocator_address = config
        .token_allocator
        .ok_or(ContractError::NotInitialized {})?;

    println!("Raffle dates = {} {} ", date_time, date);
    let nod_address = config.nod.ok_or(ContractError::NotInitialized {})?;

    let tributes_in_current_raffle: Vec<String> = if raffle_run_today == 1 {
        // distribute tokens
        let tributes: tribute::query::DailyTributesResponse = deps.querier.query_wasm_smart(
            &tribute_address,
            &tribute::query::QueryMsg::DailyTributes {
                date: date_time,
                status: Some(tribute::types::Status::Accepted),
            },
        )?;
        println!("Raffle tributes = {}", tributes.tributes.len());

        // query total to distribute
        // todo rely on block numbers to calc how many we need to distribute?
        let allocation_per_block: token_allocator::types::TokenAllocatorData =
            deps.querier.query_wasm_smart(
                &token_allocator_address,
                &token_allocator::query::QueryMsg::GetData {},
            )?;
        //
        let total_allocation =
            Uint128::from(allocation_per_block.amount) * Uint128::new(24 * 60 * 12);
        let allocation_per_pool = total_allocation / Uint128::new(24);

        let mut distributed_tributes: HashSet<String> = HashSet::new();
        let mut pools: Vec<Vec<String>> = Vec::with_capacity(24);
        for pool_id in [0usize; 23] {
            let mut pool_tributes: Vec<String> = vec![];
            let mut allocated_in_pool = Uint128::zero();
            for tribute in tributes.tributes.clone() {
                if allocated_in_pool >= allocation_per_pool {
                    break;
                }
                if !distributed_tributes.contains(&tribute.token_id) {
                    if allocated_in_pool + tribute.data.minor_value_settlement > allocation_per_pool
                    {
                        continue;
                    }
                    allocated_in_pool += tribute.data.minor_value_settlement;
                    pool_tributes.push(tribute.token_id.clone());
                    distributed_tributes.insert(tribute.token_id.clone());
                }
            }
            println!(
                "Distributed in pool {:?}: {:?} tributes",
                pool_id,
                pool_tributes.len()
            );
            pools.push(pool_tributes);
        }

        for (i, pool) in pools.iter().enumerate() {
            for (j, tribute_id) in pool.iter().enumerate() {
                // todo define public key for such struct
                // NB: i starts from 1 because first vector starts from 1
                let key = format!("{}_{}_{}", date, i + 1, j);
                TRIBUTES_DISTRIBUTION.save(deps.storage, &key, tribute_id)?;
            }
        }
        pools.first().unwrap_or(&vec![]).clone()
    } else {
        // use already distributed tokens
        let mut result: Vec<String> = vec![];
        let mut j: usize = 0;
        loop {
            let key = format!("{}_{}_{}", date, raffle_run_today, j);
            let tribute_id = TRIBUTES_DISTRIBUTION.may_load(deps.storage, &key)?;
            match tribute_id {
                None => {
                    break;
                }
                Some(id) => result.push(id),
            }
            j += 1;
        }
        result
    };

    // mint nod
    let mut messages: Vec<SubMsg> = vec![];
    let tributes_count = tributes_in_current_raffle.len();
    println!(
        "Tributes in current run {}: {}",
        raffle_run_today, tributes_count
    );
    for tribute_id in tributes_in_current_raffle {
        let tribute: tribute::query::TributeInfoResponse = deps.querier.query_wasm_smart(
            &tribute_address,
            &tribute::query::QueryMsg::NftInfo {
                token_id: tribute_id.clone(),
            },
        )?;
        let nod_id = format!("{}_{}", tribute_id, raffle_run_today);
        let nod_mint = WasmMsg::Execute {
            contract_addr: nod_address.to_string(),
            msg: to_json_binary(&nod::msg::ExecuteMsg::Submit {
                token_id: nod_id.clone(),
                owner: tribute.owner.to_string(),
                extension: Box::new(nod::msg::SubmitExtension {
                    entity: nod::msg::NodEntity {
                        nod_id,
                        settlement_token: Denom::Cw20(Addr::unchecked(
                            "gem1aaf9r6s7nxhysuegqrxv0wpm27ypyv4886medd3mrkrw6t4yfcnsvkn2zr",
                        )), // todo query from tribute?
                        symbolic_rate: Default::default(),
                        vector_rate: Default::default(),
                        nominal_minor_rate: tribute.extension.nominal_minor_qty,
                        issuance_minor_rate: Default::default(),
                        symbolic_minor_load: tribute.extension.symbolic_load,
                        vector_minor_rate: Default::default(),
                        floor_minor_price: Default::default(),
                        state: nod::types::State::Issued,
                        address: tribute.owner.to_string(),
                    },
                    created_at: None,
                }),
            })?,
            funds: vec![],
        };
        messages.push(SubMsg::new(nod_mint));
    }

    DAILY_RAFFLE.save(deps.storage, date, &raffle_run_today)?;

    Ok(Response::new()
        .add_attribute("action", "raffle::raffle")
        .add_event(
            Event::new("raffle::raffle")
                .add_attribute("run", raffle_run_today.to_string())
                .add_attribute("tributes_count", format!("{}", tributes_count)),
        )
        .add_submessages(messages))
}

/// Normalize any timestamp to midnight UTC of that day.
fn normalize_to_date(timestamp: Timestamp) -> Timestamp {
    // 86400 seconds in a day
    let seconds = timestamp.seconds();
    let days = seconds / 86400;
    Timestamp::from_seconds(days * 86400)
}
