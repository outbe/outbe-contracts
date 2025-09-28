use crate::contract::{execute, instantiate};
use crate::msg::ExecuteMsg;
use agent_common::msg::InstantiateMsg;
use agent_common::state::AGENTS;
use agent_common::types::{Agent, AgentExt, AgentStatus, AgentType, ExternalWallet, WalletType};
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{Addr, DepsMut, Env, Uint128};

const OWNER: &str = "owner";
const IBA_AGENT: &str = "iba_agent";
const REGISTRY: &str = "registry";

fn setup_contract(deps: DepsMut) {
    let msg = InstantiateMsg {
        application_registry_addr: Addr::unchecked(REGISTRY),
        paused: Some(false),
    };
    let info = message_info(&Addr::unchecked(OWNER), &[]);
    let env = mock_env();
    instantiate(deps, env, info, msg).unwrap();
}

fn create_iba_agent(deps: DepsMut, env: &Env, wallet: &str) {
    let agent = Agent {
        wallet: Addr::unchecked(wallet),
        agent_type: AgentType::Iba,
        name: "Test IBA Agent".to_string(),
        email: None,
        jurisdictions: vec!["US".to_string()],
        endpoint: Some("https://test-iba.com".to_string()),
        metadata_json: Some(r#"{"type": "iba"}"#.to_string()),
        docs_uri: vec!["https://docs.test-iba.com".to_string()],
        discord: Some("test_iba#1234".to_string()),
        status: AgentStatus::Active,
        avg_cu: Some(Uint128::new(1500)),
        submitted_at: env.block.time,
        updated_at: env.block.time,
        ext: AgentExt::Iba {
            preferred_nra: Some(vec![Addr::unchecked("nra1")]),
            additional_wallets: Some(vec![
                ExternalWallet {
                    wallet_type: WalletType::Cosmos,
                    address: "cosmos1abc123".to_string(),
                },
            ]),
            license_number: Some("IBA123456".to_string()),
            license_uri: Some("https://license.iba.com".to_string()),
        },
    };

    AGENTS.save(deps.storage, Addr::unchecked(wallet), &agent).unwrap();
}

#[test]
fn test_edit_additional_wallets() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    setup_contract(deps.as_mut());
    create_iba_agent(deps.as_mut(), &env, IBA_AGENT);

    let new_wallets = vec![
        ExternalWallet {
            wallet_type: WalletType::Evm,
            address: "0x9876543210".to_string(),
        },
    ];

    let msg = ExecuteMsg::EditAdditionalWallets {
        additional_wallets: Some(new_wallets.clone()),
    };
    let info = message_info(&Addr::unchecked(IBA_AGENT), &[]);

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(res.attributes[0].value, "edit_iba_additional_wallets");
    assert_eq!(res.attributes[3].value, "1");

    // Verify the agent was updated
    let updated_agent = AGENTS.load(&deps.storage, Addr::unchecked(IBA_AGENT)).unwrap();
    match updated_agent.ext {
        AgentExt::Iba { additional_wallets, .. } => {
            assert_eq!(additional_wallets, Some(new_wallets));
        }
        _ => panic!("Expected IBA agent ext"),
    }
}