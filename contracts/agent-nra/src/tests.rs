use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::msg::{ApplicationMsg, ExecuteMsg, InstantiateMsg, OwnerMsg};
use crate::query::query;
use crate::state::CONFIG;
use crate::types::ApplicationInput;
use agent_common::state::AGENTS;
use agent_common::types::{Agent, AgentExt, AgentInput, AgentStatus, AgentType};
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{Addr, DepsMut, Response, Uint128};
use cw_multi_test::{App, ContractWrapper, Executor};

const CREATOR: &str = "owner";
const USER1: &str = "user1";
const USER2: &str = "user2";
const USER3: &str = "user3";
const USER4: &str = "user4";
const USER5: &str = "user5";

fn sample_application_input() -> Box<ApplicationInput> {
    Box::from(ApplicationInput {
        application_type: AgentType::Nra,
        name: "Test NRA".to_string(),
        email: Some("test@nra.com".to_string()),
        jurisdictions: vec!["US".to_string(), "EU".to_string()],
        endpoint: Some("https://test-nra.com".to_string()),
        metadata_json: Some(r#"{"key": "value"}"#.to_string()),
        docs_uri: vec!["https://docs.test-nra.com".to_string()],
        discord: Some("test_nra#1234".to_string()),
        avg_cu: Option::from(Uint128::new(1000)),
        ext: Some(AgentExt::Nra {}),
    })
}

fn sample_agent_input() -> AgentInput {
    AgentInput {
        name: "Test Agent".to_string(),
        email: Some("agent@test.com".to_string()),
        jurisdictions: vec!["US".to_string()],
        endpoint: Some("https://agent.test.com".to_string()),
        metadata_json: Some(r#"{"agent": "data"}"#.to_string()),
        docs_uri: vec!["https://agent-docs.com".to_string()],
        discord: Some("agent#5678".to_string()),
        avg_cu: Option::from(Uint128::new(2000)),
        ext: AgentExt::Nra {},
    }
}

fn create_mock_agent(deps: DepsMut, env: &cosmwasm_std::Env, wallet: &str) {
    let agent = Agent {
        wallet: Addr::unchecked(wallet),
        agent_type: AgentType::Nra,
        name: "Test Agent".to_string(),
        email: Some("test@example.com".to_string()),
        jurisdictions: vec!["US".to_string()],
        endpoint: Some("https://test.com".to_string()),
        metadata_json: None,
        docs_uri: vec![], // Changed from None to empty Vec<String>
        discord: None,
        status: AgentStatus::Active,
        avg_cu: Option::from(Uint128::new(100)), // Changed from u64 to Uint128
        submitted_at: env.block.time,
        updated_at: env.block.time,
        ext: AgentExt::Nra {}, // Changed from None to ApplicationExt::Nra {}
    };

    AGENTS
        .save(deps.storage, Addr::unchecked(wallet), &agent)
        .unwrap();
}

fn instantiate_contract(deps: DepsMut) -> Result<Response, ContractError> {
    let env = mock_env();
    let info = message_info(&Addr::unchecked(CREATOR), &[]);

    let msg = InstantiateMsg {
        bootstrap_voters: Some(vec![
            USER1.to_string(),
            USER2.to_string(),
            USER3.to_string(),
            USER4.to_string(),
        ]),
        thresholds: None,
        paused: None,
    };

    instantiate(deps, env, info, msg)
}

#[test]

fn test_instantiate_with_defaults() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    let config = CONFIG.load(&deps.storage).unwrap();
    assert!(!config.paused); // Default paused
}

#[test]
fn test_create_application_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Create application
    let msg = ExecuteMsg::Application(ApplicationMsg::CreateApplication {
        application: sample_application_input(),
    });

    let info = message_info(&Addr::unchecked(USER1), &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "application::add");
}

#[test]
fn test_edit_application_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Create application
    let create_msg = ExecuteMsg::Application(ApplicationMsg::CreateApplication {
        application: sample_application_input(),
    });
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1.clone(), create_msg).unwrap();

    // Edit application
    let mut updated_app = sample_application_input();
    updated_app.name = "Updated NRA Name".to_string();

    let edit_msg = ExecuteMsg::Application(ApplicationMsg::EditApplication {
        id: "0".to_string(),
        application: updated_app,
    });

    let res = execute(deps.as_mut(), env, info_user1, edit_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "application::update");
}

#[test]
fn test_edit_application_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Create application as USER1
    let create_msg = ExecuteMsg::Application(ApplicationMsg::CreateApplication {
        application: sample_application_input(),
    });
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1, create_msg).unwrap();

    // Try to edit application as USER2 (unauthorized)
    let edit_msg = ExecuteMsg::Application(ApplicationMsg::EditApplication {
        id: "1".to_string(),
        application: sample_application_input(),
    });
    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);
    let res = execute(deps.as_mut(), env, info_user2, edit_msg);
    assert!(res.is_err());
}

#[test]
fn test_vote_application_approve() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    let create_msg = ExecuteMsg::Application(ApplicationMsg::CreateApplication {
        application: sample_application_input(),
    });
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1, create_msg).unwrap();

    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);

    // Vote on application
    let vote_msg = ExecuteMsg::Application(ApplicationMsg::VoteApplication {
        id: "0".to_string(),
        approve: true,
        reason: Some("Good application".to_string()),
    });

    let res = execute(deps.as_mut(), env, info_user2, vote_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "application::vote_application");
}

#[test]
fn test_vote_application_reject() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    let create_msg = ExecuteMsg::Application(ApplicationMsg::CreateApplication {
        application: sample_application_input(),
    });
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1, create_msg).unwrap();

    // Submit agent
    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);

    // Vote reject
    let vote_msg = ExecuteMsg::Application(ApplicationMsg::VoteApplication {
        id: "0".to_string(),
        approve: false,
        reason: Some("Insufficient documentation".to_string()),
    });

    let res = execute(deps.as_mut(), env, info_user2, vote_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "application::vote_application");
}

#[test]
fn test_vote_application_non_exist() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    let create_msg = ExecuteMsg::Application(ApplicationMsg::CreateApplication {
        application: sample_application_input(),
    });
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1.clone(), create_msg).unwrap();

    // Vote on nonexistent application
    let vote_msg = ExecuteMsg::Application(ApplicationMsg::VoteApplication {
        id: "999".to_string(),
        approve: true,
        reason: None,
    });

    let res = execute(deps.as_mut(), env, info_user1, vote_msg);

    assert!(res.is_err());
    match res.unwrap_err() {
        ContractError::ApplicationNotFound {} => {}
        other => panic!("Expected ApplicationNotFound error, got: {:?}", other),
    }
}

#[test]
fn test_hold_application() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    let create_msg = ExecuteMsg::Application(ApplicationMsg::CreateApplication {
        application: sample_application_input(),
    });
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1, create_msg).unwrap();

    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);

    // Hold application
    let hold_msg = ExecuteMsg::Application(ApplicationMsg::HoldApplication {
        id: "0".to_string(),
    });

    let res = execute(deps.as_mut(), env, info_user2, hold_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "application::hold");
}

#[test]
fn test_submit_agent_flow() {
    // Spin up a mock chain
    let mut app = App::default();
    let owner = app.api().addr_make("owner");
    let user1 = app.api().addr_make("user1");
    let user2 = app.api().addr_make("user2");
    let user3 = app.api().addr_make("user3");
    let user4 = app.api().addr_make("user4");

    // Upload our contract code
    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id = app.store_code(Box::new(code));

    // Instantiate NRA with bootstrap voters
    let init_msg = InstantiateMsg {
        bootstrap_voters: Some(vec![
            user1.to_string(),
            user2.to_string(),
            user3.to_string(),
            user4.to_string(),
        ]),
        thresholds: None,
        paused: None,
    };
    let nra_addr = app
        .instantiate_contract(code_id, owner.clone(), &init_msg, &[], "agent-nra", None)
        .unwrap();

    // 1) Create application from user1
    let create_res = app
        .execute_contract(
            user1.clone(),
            nra_addr.clone(),
            &ExecuteMsg::Application(ApplicationMsg::CreateApplication {
                application: sample_application_input(), // <-- твой helper
            }),
            &[],
        )
        .unwrap();

    // Extract application_id from events
    let app_id = create_res
        .events
        .iter()
        .flat_map(|e| e.attributes.iter())
        .find(|a| a.key == "application_id")
        .expect("application_id not found")
        .value
        .clone();

    // 2) Vote 3 times to reach Approved status
    for voter in [&user2, &user3, &user4] {
        app.execute_contract(
            voter.clone(),
            nra_addr.clone(),
            &ExecuteMsg::Application(ApplicationMsg::VoteApplication {
                id: app_id.clone(),
                approve: true,
                reason: Some("good".to_string()),
            }),
            &[],
        )
        .unwrap();
    }

    // 3) Submit agent
    let submit_res = app
        .execute_contract(
            user1.clone(),
            nra_addr.clone(),
            &ExecuteMsg::Agent(agent_common::msg::ExecuteMsg::SubmitAgent { id: app_id.clone() }),
            &[],
        )
        .unwrap();

    // Verify response attributes (from events)
    let attrs: Vec<_> = submit_res
        .events
        .iter()
        .flat_map(|e| &e.attributes)
        .collect();
    let get = |k: &str| {
        attrs
            .iter()
            .find(|a| a.key == k)
            .unwrap_or_else(|| panic!("missing attribute '{k}'"))
            .value
            .clone()
    };
    assert_eq!(get("action"), "agent-nra::submit");
    assert_eq!(get("application_id"), app_id);
    assert_eq!(get("wallet"), user1.to_string());
}
#[test]
fn test_edit_agent() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Create mock agent directly in storage instead of going through full process
    create_mock_agent(deps.as_mut(), &env, USER1);

    // Edit agent
    let edit_msg = ExecuteMsg::Agent(agent_common::msg::ExecuteMsg::EditAgent {
        agent: Box::new(sample_agent_input()),
    });
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);

    let res = execute(deps.as_mut(), env, info_user1, edit_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "agent-nra::edit");
}

#[test]
fn test_unauthorized_vote() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Create mock agents - USER1 and USER2 are active NRA agents
    create_mock_agent(deps.as_mut(), &env, USER1);
    create_mock_agent(deps.as_mut(), &env, USER2);

    // USER5 is not an NRA agent and not in bootstrap voters
    let unauthorized_info = message_info(&Addr::unchecked(USER5), &[]);

    // Test: Unauthorized user tries to vote on application
    // First create an application
    let create_msg = ExecuteMsg::Application(ApplicationMsg::CreateApplication {
        application: sample_application_input(),
    });
    let res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(USER1), &[]),
        create_msg,
    )
    .unwrap();

    // Get application ID from response
    let app_id = res
        .attributes
        .iter()
        .find(|attr| attr.key == "application_id")
        .unwrap()
        .value
        .clone();

    // Try to vote with unauthorized user
    let vote_msg = ExecuteMsg::Application(ApplicationMsg::VoteApplication {
        id: app_id,
        approve: true,
        reason: Some("Unauthorized vote".to_string()),
    });
    let res = execute(deps.as_mut(), env, unauthorized_info, vote_msg);

    assert!(res.is_err());
    match res.unwrap_err() {
        ContractError::OnlyActiveNra {} => {
            // Correct error - test passes
        }
        other => panic!("Expected OnlyActiveNra error, got: {:?}", other),
    }
}

#[test]
fn test_preferred_nra_restrict() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    create_mock_agent(deps.as_mut(), &env, USER2);
    create_mock_agent(deps.as_mut(), &env, USER3);

    let mut app = sample_application_input();
    app.application_type = AgentType::Cra;
    app.ext = Some(AgentExt::Cra {
        preferred_nra: Some(vec![Addr::unchecked(USER2)]),
        additional_wallets: None,
    });

    let create_res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(USER1), &[]),
        ExecuteMsg::Application(ApplicationMsg::CreateApplication { application: app }),
    )
    .unwrap();

    let app_id = create_res
        .attributes
        .iter()
        .find(|a| a.key == "application_id")
        .map(|a| a.value.clone())
        .unwrap_or_else(|| "1".to_string());

    let res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(USER3), &[]),
        ExecuteMsg::Application(ApplicationMsg::VoteApplication {
            id: app_id.clone(),
            approve: true,
            reason: None,
        }),
    );

    println!("{:?}", res);
    assert!(res.is_err());
    match res.unwrap_err() {
        ContractError::OnlyPreferredNra {} => {}
        other => panic!("Expected OnlyPreferredNra error, got: {:?}", other),
    }

    let res_ok = execute(
        deps.as_mut(),
        env,
        message_info(&Addr::unchecked(USER2), &[]),
        ExecuteMsg::Application(ApplicationMsg::VoteApplication {
            id: app_id,
            approve: true,
            reason: Some("ok".into()),
        }),
    )
    .unwrap();

    assert_eq!(res_ok.attributes[0].key, "action");
    assert_eq!(res_ok.attributes[0].value, "application::vote_application");
}

#[test]
fn test_add_agent_directly_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Owner adds agent directly
    let agent_address = USER1.to_string();
    let agent_input = sample_agent_input();

    let msg = ExecuteMsg::Owner(OwnerMsg::AddNraDirectly {
        address: agent_address.clone(),
        agent: Box::new(agent_input.clone()),
    });

    let info_owner = message_info(&Addr::unchecked(CREATOR), &[]);
    let res = execute(deps.as_mut(), env.clone(), info_owner, msg).unwrap();

    // Check response attributes
    assert_eq!(res.attributes.len(), 5);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "agent-nra::add_agent_directly");
    assert_eq!(res.attributes[1].key, "agent_address");
    assert_eq!(res.attributes[1].value, agent_address);
    assert_eq!(res.attributes[2].key, "agent_type");
    assert_eq!(res.attributes[2].value, "NRA");
    assert_eq!(res.attributes[3].key, "status");
    assert_eq!(res.attributes[3].value, "Active");

    // Verify agent was created in storage
    let agent_addr = Addr::unchecked(&agent_address);
    let agent = AGENTS.load(&deps.storage, agent_addr.clone()).unwrap();
    assert_eq!(agent.wallet, agent_addr);
    assert_eq!(agent.name, agent_input.name);
    assert_eq!(agent.email, agent_input.email);
    assert_eq!(agent.agent_type, AgentType::Nra);
    assert_eq!(agent.status, AgentStatus::Active);
    assert_eq!(agent.jurisdictions, agent_input.jurisdictions);
    assert_eq!(agent.endpoint, agent_input.endpoint);
    assert_eq!(agent.avg_cu, agent_input.avg_cu);
}

#[test]
fn test_add_agent_directly_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Non-owner tries to add agent directly
    let agent_address = USER1.to_string();
    let agent_input = sample_agent_input();

    let msg = ExecuteMsg::Owner(OwnerMsg::AddNraDirectly {
        address: agent_address.clone(),
        agent: Box::new(agent_input),
    });

    let info_user = message_info(&Addr::unchecked(USER2), &[]);
    let res = execute(deps.as_mut(), env, info_user, msg);

    // Should fail with Unauthorized error
    assert!(res.is_err());
    match res.unwrap_err() {
        ContractError::Unauthorized => {}
        other => panic!("Expected Unauthorized error, got: {:?}", other),
    }

    // Verify no agent was created
    let agent_addr = Addr::unchecked(&agent_address);
    let agent = AGENTS.may_load(&deps.storage, agent_addr).unwrap();
    assert!(agent.is_none());
}
