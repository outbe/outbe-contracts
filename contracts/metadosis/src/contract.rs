use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::prepare;
use crate::state::{
    Config, DailyRunState, Entry, LysisEntity, LysisInfo, MetadosisInfo, TouchEntity, TouchInfo,
    CONFIG, CREATOR, DAILY_RUN_STATE, ENTRY_STATE, METADOSIS_INFO, WINNERS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Decimal, DepsMut, Env, Event, HexBinary, MessageInfo, Reply,
    Response, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::ParseReplyError::SubMsgFailure;
use cw_utils::{parse_execute_response_data, MsgExecuteContractResponse};
use outbe_utils::consts::to_decimals_amount;
use outbe_utils::date::WorldwideDay;
use outbe_utils::{date, gen_compound_hash};
use rand::prelude::SliceRandom;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::str::FromStr;
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
            tribute: msg.tribute,
            nod: msg.nod,
            token_allocator: msg.token_allocator,
            price_oracle: msg.price_oracle,
            random_oracle: msg.random_oracle,
            lysis_limit_percent: msg.lysis_limit_percent,
        },
    )?;

    Ok(Response::default()
        .add_attribute("action", "metadosis::instantiate")
        .add_event(Event::new("metadosis::instantiate").add_attribute("creator", creator)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg {
        MigrateMsg::Migrate {} => Ok(Response::new()),
    }
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
        #[cfg(feature = "demo")]
        ExecuteMsg::BurnAll {} => execute_burn_all(deps, &env, &info),
    }
}

/// A unique ID for tokens allocation callback
const ALLOCATE_NATIVE_TOKENS_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    // Match on the ID of the reply to handle the correct one
    match msg.id {
        ALLOCATE_NATIVE_TOKENS_REPLY_ID => handle_token_allocation_reply(deps, msg),
        _ => Err(ContractError::UnrecognizedReplyId { id: msg.id }),
    }
}

fn execute_prepare(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    run_date: Option<WorldwideDay>,
) -> Result<Response, ContractError> {
    // todo verify ownership to run metadosis

    let execution_date = get_execution_date(run_date, &env)?;

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

fn handle_token_allocation_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    //todo verify caller is token_allocator_address

    println!("handle_token_allocation_reply {:?}", msg);

    // 1. Check the result of the submessage
    let subcall_result = msg.result.into_result().map_err(SubMsgFailure)?;

    // 2. Get the data from the successful reply
    #[allow(deprecated)] // NB: older version for SEI
    let data = subcall_result.data.ok_or(ContractError::NoDataInReply {})?;

    // 3. Deserialize the data into your expected struct
    let allocation_result: MsgExecuteContractResponse =
        parse_execute_response_data(data.as_slice())?;

    let allocation_result = allocation_result
        .data
        .ok_or(ContractError::NoDataInReply {})?;

    let allocation_result: token_allocator::contract::AllocationResult =
        from_json(allocation_result.as_slice())?;

    prepare::prepare_executions(deps, allocation_result.allocation, allocation_result.day)?;

    Ok(Response::new()
        .add_attribute("action", "metadosis::handle_allocation_reply")
        .add_event(
            Event::new("metadosis::handle_allocation_reply")
                .add_attribute("date", allocation_result.day.to_string())
                .add_attribute("allocation", allocation_result.allocation.to_string()),
        ))
}

fn execute_run(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    run_date: Option<WorldwideDay>,
) -> Result<Response, ContractError> {
    // todo verify ownership to run metadosis

    let execution_date = get_execution_date(run_date, &env)?;
    let config = CONFIG.load(deps.storage)?;

    let run_today = DAILY_RUN_STATE.may_load(deps.storage, execution_date)?;
    let mut run_today = run_today.unwrap_or(DailyRunState {
        number_of_runs: 0,
        last_tribute_id: None,
        undistributed_limit: Uint128::zero(),
    });
    run_today.number_of_runs += 1;

    let info = METADOSIS_INFO
        .load(deps.storage, execution_date)
        .map_err(|_| ContractError::NotPrepared {})?;

    let result: Result<Response, ContractError> = match info {
        MetadosisInfo::Lysis { lysis_info } => match run_today.number_of_runs {
            1 => do_execute_lysis(deps, env.block.time, execution_date, lysis_info, run_today),
            _ => return Err(ContractError::BadRunConfiguration {}),
        },
        MetadosisInfo::Touch { touch_info } => {
            if run_today.number_of_runs > 1 {
                return Err(ContractError::BadRunConfiguration {});
            }
            do_execute_touch(deps, env.block.time, execution_date, touch_info, run_today)
        }
    };

    let mut submessages: Vec<SubMsg> = vec![];
    let tribute_address = config.tribute.ok_or(ContractError::NotInitialized {})?;
    let submsg = SubMsg::new(WasmMsg::Execute {
        contract_addr: tribute_address.to_string(),
        msg: to_json_binary(&tribute::msg::ExecuteMsg::BurnForDay {
            date: execution_date,
        })?,
        funds: vec![],
    });
    submessages.push(submsg);

    Ok(result?.add_submessages(submessages))
}

fn do_execute_lysis(
    deps: DepsMut,
    block_time: Timestamp,
    execution_date: WorldwideDay,
    lysis_info: LysisInfo,
    run_today: DailyRunState,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let tribute_address = config.tribute.ok_or(ContractError::NotInitialized {})?;
    let nod_address = config.nod.ok_or(ContractError::NotInitialized {})?;
    let price_oracle_address = config
        .price_oracle
        .ok_or(ContractError::NotInitialized {})?;
    let exchange_rate: price_oracle::types::TokenPairPrice = deps.querier.query_wasm_smart(
        &price_oracle_address,
        &price_oracle::query::QueryMsg::GetPrice {},
    )?;

    let tributes: tribute::query::FullTributesResponse = deps.querier.query_wasm_smart(
        &tribute_address,
        &tribute::query::QueryMsg::DailyTributes {
            date: Some(execution_date),
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
            .add_event(Event::new("metadosis::lysis").add_attribute("tributes_count", "0")));
    }

    let mut allocated_tributes_sum = Uint128::zero();
    let mut allocated_tributes: Vec<(FullTributeData, Uint128)> = vec![];
    for tribute in tributes.tributes {
        let symbolic_load = (to_decimals_amount(tribute.data.nominal_qty_minor)
            * lysis_info.distribution_percent)
            .atomics();
        allocated_tributes_sum += symbolic_load;
        allocated_tributes.push((tribute, symbolic_load));
    }

    let allocated_tributes_count = allocated_tributes.len();
    println!(
        "Tributes in current run {}: count = {}, sum = {}",
        run_today.number_of_runs, allocated_tributes_count, allocated_tributes_sum
    );

    let mut messages: Vec<SubMsg> = vec![];
    for (tribute, symbolic_load) in allocated_tributes {
        let nod_token_id = generate_nod_id(&tribute.token_id, &tribute.owner);

        // todo impl floor price with S-Curve
        let floor_price =
            exchange_rate.price * (Decimal::one() + Decimal::from_str("0.08").unwrap());

        let mod_issuance_price = exchange_rate.price.max(tribute.data.nominal_price_minor);
        let nod_mint = WasmMsg::Execute {
            contract_addr: nod_address.to_string(),
            msg: to_json_binary(&nod::msg::ExecuteMsg::Submit {
                token_id: nod_token_id.to_hex(),
                owner: tribute.owner.to_string(),
                extension: Box::new(nod::msg::SubmitExtension {
                    entity: nod::msg::NodEntity {
                        nod_id: nod_token_id.to_hex(),
                        settlement_currency: tribute.data.settlement_currency.clone(),
                        symbolic_rate: config.lysis_limit_percent,
                        floor_rate: Decimal::zero(), // todo populate
                        nominal_price_minor: tribute.data.nominal_price_minor,
                        issuance_price_minor: mod_issuance_price,
                        gratis_load_minor: symbolic_load,
                        floor_price_minor: floor_price,
                        state: nod::types::State::Issued,
                        owner: tribute.owner.to_string(),
                        qualified_at: None,
                        is_touch: false,
                    },
                    created_at: None,
                }),
            })?,
            funds: vec![],
        };
        messages.push(SubMsg::new(nod_mint));
    }

    DAILY_RUN_STATE.save(
        deps.storage,
        execution_date,
        &DailyRunState {
            number_of_runs: run_today.number_of_runs,
            last_tribute_id: None,
            undistributed_limit: Uint128::zero(),
        },
    )?;

    ENTRY_STATE.save(
        deps.storage,
        execution_date,
        &Entry::Lysis(LysisEntity {
            id: format!("lysis_{}", execution_date),
            index: 1,
            limit: lysis_info.total_lysis_limit,
            deficit: lysis_info.total_lysis_deficit,
            capacity: lysis_info.total_tribute_interest,
            worldwide_day: execution_date,
            total_gratis_limit: lysis_info.total_gratis_limit,
            assigned_tributes: allocated_tributes_count,
            timestamp: block_time,
            assigned_tributes_sum: allocated_tributes_sum,
        }),
    )?;

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
    block_time: Timestamp,
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

    let tributes: tribute::query::FullTributesResponse = deps.querier.query_wasm_smart(
        &tribute_address,
        &tribute::query::QueryMsg::DailyTributes {
            date: Some(execution_date),
            query_order: None,
            limit: None,
            start_after: None,
        },
    )?;

    let mut allocated_tributes: Vec<FullTributeData> = vec![];
    for tribute in tributes.tributes {
        if WINNERS.has(deps.storage, tribute.token_id.clone()) {
            continue;
        }
        allocated_tributes.push(tribute);
    }

    let assigned_tributes_count = allocated_tributes.len();
    println!(
        "Tributes in current run {}: count = {}",
        run_today.number_of_runs, assigned_tributes_count
    );

    // fast exit when no tributes
    if allocated_tributes.is_empty() {
        ENTRY_STATE.save(
            deps.storage,
            execution_date,
            &Entry::Touch(TouchEntity {
                id: format!("touch_{}", execution_date),
                worldwide_day: execution_date,
                total_gratis_limit: touch_info.total_gratis_limit,
                gold_ignot_price: touch_info.gold_ignot_price,
                touch_limit: touch_info.touch_limit,
                assigned_tributes: assigned_tributes_count,
                recognised_tributes: vec![],
                timestamp: block_time,
            }),
        )?;

        DAILY_RUN_STATE.save(deps.storage, execution_date, &run_today)?;

        return Ok(Response::new()
            .add_attribute("action", "metadosis::touch")
            .add_event(Event::new("metadosis::touch").add_attribute("tributes_count", "0")));
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

    let (expected_winners_count, win_amount) =
        calc_touch_win_amount(touch_info.touch_limit, touch_info.gold_ignot_price);

    let mut winners: Vec<FullTributeData> = vec![];
    for tribute in allocated_tributes {
        WINNERS.save(deps.storage, tribute.token_id.clone(), &())?;
        winners.push(tribute);
        if expected_winners_count == winners.len() {
            break;
        }
    }
    let winners_ids: Vec<String> = winners.iter().map(|t| t.token_id.clone()).collect();

    let mut messages: Vec<SubMsg> = vec![];
    for tribute in winners {
        let token_id = generate_nod_id(&tribute.token_id, &tribute.owner);

        let mod_issuance_price = exchange_rate.price.max(tribute.data.nominal_price_minor);
        let nod_mint = WasmMsg::Execute {
            contract_addr: nod_address.to_string(),
            msg: to_json_binary(&nod::msg::ExecuteMsg::Submit {
                token_id: token_id.to_hex(),
                owner: tribute.owner.to_string(),
                extension: Box::new(nod::msg::SubmitExtension {
                    entity: nod::msg::NodEntity {
                        nod_id: token_id.to_hex(),
                        settlement_currency: tribute.data.settlement_currency.clone(),
                        symbolic_rate: config.lysis_limit_percent,
                        floor_rate: Decimal::zero(),
                        nominal_price_minor: tribute.data.nominal_price_minor,
                        issuance_price_minor: mod_issuance_price,
                        gratis_load_minor: win_amount,
                        floor_price_minor: exchange_rate.price,
                        state: nod::types::State::Issued,
                        owner: tribute.owner.to_string(),
                        qualified_at: None,
                        is_touch: true,
                    },
                    created_at: None,
                }),
            })?,
            funds: vec![],
        };
        messages.push(SubMsg::new(nod_mint));
    }

    ENTRY_STATE.save(
        deps.storage,
        execution_date,
        &Entry::Touch(TouchEntity {
            id: format!("touch_{}", execution_date),
            worldwide_day: execution_date,
            total_gratis_limit: touch_info.total_gratis_limit,
            gold_ignot_price: touch_info.gold_ignot_price,
            touch_limit: touch_info.touch_limit,
            assigned_tributes: assigned_tributes_count,
            recognised_tributes: winners_ids,
            timestamp: block_time,
        }),
    )?;

    Ok(Response::new()
        .add_attribute("action", "metadosis::touch")
        .add_event(
            Event::new("metadosis::touch")
                .add_attribute("run", run_today.number_of_runs.to_string()),
        )
        .add_submessages(messages))
}

fn generate_nod_id(token_id: &String, owner: &String) -> HexBinary {
    gen_compound_hash(
        Some("metadosis:nod_id"),
        vec![token_id.as_bytes(), owner.as_bytes()],
    )
}

fn calc_touch_win_amount(touch_limit: Uint128, ignot_price: Decimal) -> (usize, Uint128) {
    let mut expected_winners_count = 1usize;
    let mut expected_win_amount = touch_limit;

    let ignot_price_ato = ignot_price.atomics();
    if ignot_price_ato < touch_limit {
        let winners_count = touch_limit / ignot_price_ato;
        expected_winners_count = winners_count.u128() as usize;
        expected_win_amount = touch_limit / winners_count;
    };

    (expected_winners_count, expected_win_amount)
}

fn get_execution_date(
    run_date: Option<WorldwideDay>,
    env: &Env,
) -> Result<WorldwideDay, ContractError> {
    let execution_date = run_date.unwrap_or(date::normalize_to_date(&env.block.time));
    date::is_valid(&execution_date)?;
    println!("execution date = {}", execution_date);
    Ok(execution_date)
}

#[cfg(feature = "demo")]
fn execute_burn_all(
    deps: DepsMut,
    _env: &Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    METADOSIS_INFO.clear(deps.storage);
    DAILY_RUN_STATE.clear(deps.storage);
    ENTRY_STATE.clear(deps.storage);
    WINNERS.clear(deps.storage);

    Ok(Response::new()
        .add_attribute("action", "metadosis::burn_all")
        .add_event(
            Event::new("metadosis::burn_all").add_attribute("sender", info.sender.to_string()),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use outbe_utils::consts::DECIMALS;
    use std::str::FromStr;

    #[test]
    fn ignot_price_higher_than_touch_limit() {
        let touch_limit = Uint128::new(30) * DECIMALS;
        let ignot_price = Decimal::from_str("45.3").unwrap();

        let (winners, amount) = calc_touch_win_amount(touch_limit, ignot_price);
        assert_eq!(winners, 1);
        assert_eq!(amount, touch_limit);
    }

    #[test]
    fn ignot_price_lower_than_touch_limit() {
        let touch_limit = Uint128::new(100) * DECIMALS;
        let ignot_price = Decimal::from_str("45.3").unwrap();

        let (winners, amount) = calc_touch_win_amount(touch_limit, ignot_price);
        assert_eq!(winners, 2);
        assert_eq!(amount, touch_limit / Uint128::new(2));
    }

    #[test]
    fn ignot_price_equal_to_touch_limit() {
        let touch_limit = Uint128::new(100) * DECIMALS;
        let ignot_price = Decimal::from_str("50").unwrap();

        let (winners, amount) = calc_touch_win_amount(touch_limit, ignot_price);
        assert_eq!(winners, 2);
        assert_eq!(amount, touch_limit / Uint128::new(2));
    }

    #[test]
    fn test_generate_nod_id_basic() {
        let token_id = "token123".to_string();
        let owner = "owner456".to_string();
        let nod_id = generate_nod_id(&token_id, &owner);

        // Check that output is a valid hex string
        assert!(nod_id.to_hex().chars().all(|c| c.is_ascii_hexdigit()));
        // Check length is 64 (32 bytes as hex)
        assert_eq!(nod_id.len(), 32);
    }

    #[test]
    fn test_generate_nod_id_deterministic() {
        let token_id = "token123".to_string();
        let owner = "owner456".to_string();

        let nod_id1 = generate_nod_id(&token_id, &owner);
        let nod_id2 = generate_nod_id(&token_id, &owner);

        // Same inputs should produce same output
        assert_eq!(nod_id1, nod_id2);
    }

    #[test]
    fn test_generate_nod_id_unique() {
        let token_id1 = "token123".to_string();
        let token_id2 = "token124".to_string();
        let owner = "owner456".to_string();

        let nod_id1 = generate_nod_id(&token_id1, &owner);
        let nod_id2 = generate_nod_id(&token_id2, &owner);

        // Different inputs should produce different outputs
        assert_ne!(nod_id1, nod_id2);
    }

    #[test]
    fn test_generate_nod_id_empty_inputs() {
        let empty_token = "".to_string();
        let empty_owner = "".to_string();

        let nod_id = generate_nod_id(&empty_token, &empty_owner);

        // Should still produce valid hex string of correct length
        assert!(nod_id.to_hex().chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(nod_id.len(), 32);
    }
}
