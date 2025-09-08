use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::CONFIG;
use crate::types::{AgentInput, ApplicationExt, ApplicationInput, ApplicationType};
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{Addr, DepsMut, Response, Uint128};

const CREATOR: &str = "owner";
const USER1: &str = "user1";
const USER2: &str = "user2";
const USER3: &str = "user3";
const USER4: &str = "user4";
const USER5: &str = "user5";

fn sample_application_input() -> ApplicationInput {
    ApplicationInput {
        application_type: ApplicationType::Nra,
        name: "Test NRA".to_string(),
        email: "test@nra.com".to_string(),
        jurisdictions: vec!["US".to_string(), "EU".to_string()],
        endpoint: Some("https://test-nra.com".to_string()),
        metadata_json: Some(r#"{"key": "value"}"#.to_string()),
        docs_uri: vec!["https://docs.test-nra.com".to_string()],
        discord: Some("test_nra#1234".to_string()),
        avg_cu: Option::from(Uint128::new(1000)),
        ext: Some(ApplicationExt::Nra {}),
    }
}

fn sample_agent_input() -> AgentInput {
    AgentInput {
        name: "Test Agent".to_string(),
        email: "agent@test.com".to_string(),
        jurisdictions: vec!["US".to_string()],
        endpoint: Some("https://agent.test.com".to_string()),
        metadata_json: Some(r#"{"agent": "data"}"#.to_string()),
        docs_uri: vec!["https://agent-docs.com".to_string()],
        discord: Some("agent#5678".to_string()),
        avg_cu: Option::from(Uint128::new(2000)),
        ext: ApplicationExt::Nra {},
    }
}

fn create_mock_agent(deps: DepsMut, env: &cosmwasm_std::Env, wallet: &str) {
    use crate::state::AGENTS;
    use crate::types::{Agent, AgentStatus, ApplicationExt};
    use cosmwasm_std::{Addr, Uint128};

    let agent = Agent {
        wallet: Addr::unchecked(wallet),
        name: "Test Agent".to_string(),
        email: "test@example.com".to_string(),
        jurisdictions: vec!["US".to_string()],
        endpoint: Some("https://test.com".to_string()),
        metadata_json: None,
        docs_uri: vec![], // Changed from None to empty Vec<String>
        discord: None,
        status: AgentStatus::Active,
        avg_cu: Option::from(Uint128::new(100)), // Changed from u64 to Uint128
        submitted_at: env.block.time,
        updated_at: env.block.time,
        ext: ApplicationExt::Nra {}, // Changed from None to ApplicationExt::Nra {}
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
    let msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };

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
    let create_msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1.clone(), create_msg).unwrap();

    // Edit application
    let mut updated_app = sample_application_input();
    updated_app.name = "Updated NRA Name".to_string();

    let edit_msg = ExecuteMsg::EditApplication {
        id: "1".to_string(),
        application: updated_app,
    };

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
    let create_msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1, create_msg).unwrap();

    // Try to edit application as USER2 (unauthorized)
    let edit_msg = ExecuteMsg::EditApplication {
        id: "1".to_string(),
        application: sample_application_input(),
    };
    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);
    let res = execute(deps.as_mut(), env, info_user2, edit_msg);
    assert!(res.is_err());
}

#[test]
fn test_vote_application_approve() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    let create_msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1, create_msg).unwrap();

    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);

    // Vote on application
    let vote_msg = ExecuteMsg::VoteApplication {
        id: "1".to_string(),
        approve: true,
        reason: Some("Good application".to_string()),
    };

    let res = execute(deps.as_mut(), env, info_user2, vote_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "application::vote_application");
}

#[test]
fn test_vote_application_reject() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    let create_msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1, create_msg).unwrap();

    // Submit agent
    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);

    // Vote reject
    let vote_msg = ExecuteMsg::VoteApplication {
        id: "1".to_string(),
        approve: false,
        reason: Some("Insufficient documentation".to_string()),
    };

    let res = execute(deps.as_mut(), env, info_user2, vote_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "application::vote_application");
}

#[test]
fn test_vote_application_non_exist() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    let create_msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1.clone(), create_msg).unwrap();

    // Vote on nonexistent application
    let vote_msg = ExecuteMsg::VoteApplication {
        id: "999".to_string(),
        approve: true,
        reason: None,
    };

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

    let create_msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    execute(deps.as_mut(), env.clone(), info_user1, create_msg).unwrap();

    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);

    // Hold application
    let hold_msg = ExecuteMsg::HoldApplication {
        id: "1".to_string(),
    };

    let res = execute(deps.as_mut(), env, info_user2, hold_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "application::hold");
}

#[test]
fn test_submit_agent() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Step 1: Create application
    let create_msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);
    let res = execute(deps.as_mut(), env.clone(), info_user1.clone(), create_msg).unwrap();

    // Get application ID from response
    let app_id = res
        .attributes
        .iter()
        .find(|attr| attr.key == "application_id")
        .unwrap()
        .value
        .clone();

    // Step 2: Vote to approve the application (need 3 votes for NRA threshold)
    // First vote
    let vote_msg = ExecuteMsg::VoteApplication {
        id: app_id.clone(),
        approve: true,
        reason: Some("Good candidate".to_string()),
    };
    let info_user2 = message_info(&Addr::unchecked(USER2), &[]);
    execute(deps.as_mut(), env.clone(), info_user2, vote_msg.clone()).unwrap();

    // Second vote
    let info_user3 = message_info(&Addr::unchecked(USER3), &[]);
    execute(deps.as_mut(), env.clone(), info_user3, vote_msg.clone()).unwrap();

    // Third vote (reaches threshold)
    let info_user4 = message_info(&Addr::unchecked(USER4), &[]);
    execute(deps.as_mut(), env.clone(), info_user4, vote_msg).unwrap();

    // Step 3: Submit agent (now application should be approved)
    let submit_msg = ExecuteMsg::SubmitAgent { id: app_id.clone() };

    let res = execute(deps.as_mut(), env, info_user1, submit_msg).unwrap();

    // Verify response attributes
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "agent::submit");
    assert_eq!(res.attributes[1].key, "application_id");
    assert_eq!(res.attributes[1].value, app_id);
    assert_eq!(res.attributes[2].key, "wallet");
    assert_eq!(res.attributes[2].value, USER1);
}

#[test]
fn test_edit_agent() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_contract(deps.as_mut()).unwrap();

    // Create mock agent directly in storage instead of going through full process
    create_mock_agent(deps.as_mut(), &env, USER1);

    // Edit agent
    let edit_msg = ExecuteMsg::EditAgent {
        agent: sample_agent_input(),
    };
    let info_user1 = message_info(&Addr::unchecked(USER1), &[]);

    let res = execute(deps.as_mut(), env, info_user1, edit_msg).unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "edit_agent");
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
    let create_msg = ExecuteMsg::CreateApplication {
        application: sample_application_input(),
    };
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
    let vote_msg = ExecuteMsg::VoteApplication {
        id: app_id,
        approve: true,
        reason: Some("Unauthorized vote".to_string()),
    };
    let res = execute(deps.as_mut(), env, unauthorized_info, vote_msg);

    assert!(res.is_err());
    match res.unwrap_err() {
        ContractError::OnlyActiveNra {} => {
            // Correct error - test passes
        }
        other => panic!("Expected OnlyActiveNra error, got: {:?}", other),
    }
}
