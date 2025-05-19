use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{CONFIG, CREATOR, DAILY_RAFFLE};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, DepsMut, Env, Event, MessageInfo, Response, SubMsg, Timestamp, WasmMsg,
};
use cw20::Denom;

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

    // query tribute
    let tributes: tribute::query::DailyTributesResponse = deps.querier.query_wasm_smart(
        &tribute_address,
        &tribute::query::QueryMsg::DailyTributes {
            date: date_time,
            status: Some(tribute::types::Status::Accepted),
        },
    )?;

    // todo implement logic with vectors and raffle itself

    // mint nod
    let nod_address = config.nod.ok_or(ContractError::NotInitialized {})?;

    let mut messages: Vec<SubMsg> = vec![];
    for tribute in tributes.tributes {
        let nod_id = format!("{}_{}", tribute.token_id, raffle_run_today);
        let nod_mint = WasmMsg::Execute {
            contract_addr: nod_address.to_string(),
            msg: to_json_binary(&nod::msg::ExecuteMsg::Submit {
                token_id: nod_id.clone(),
                owner: tribute.owner.clone(),
                extension: Box::new(nod::msg::SubmitExtension {
                    entity: nod::msg::NodEntity {
                        nod_id,
                        settlement_token: Denom::Native("gem".to_string()), // todo define fields
                        symbolic_rate: Default::default(),
                        vector_rate: Default::default(),
                        nominal_minor_rate: tribute.data.nominal_minor_qty,
                        issuance_minor_rate: Default::default(),
                        symbolic_minor_load: tribute.data.symbolic_load,
                        vector_minor_rate: Default::default(),
                        floor_minor_price: Default::default(),
                        state: nod::types::State::Issued,
                        address: tribute.owner,
                    },
                    created_at: None,
                }),
            })?,
            funds: vec![],
        };
        messages.push(SubMsg::new(nod_mint));
    }

    DAILY_RAFFLE.save(deps.storage, date, &raffle_run_today)?;

    Ok(
        Response::new()
            .add_attribute("action", "raffle::raffle")
            .add_event(
                Event::new("raffle::raffle").add_attribute("run", raffle_run_today.to_string()),
            ), // .add_submessages(messages)
    )
}

/// Normalize any timestamp to midnight UTC of that day.
fn normalize_to_date(timestamp: Timestamp) -> Timestamp {
    // 86400 seconds in a day
    let seconds = timestamp.seconds();
    let days = seconds / 86400;
    Timestamp::from_seconds(days * 86400)
}
