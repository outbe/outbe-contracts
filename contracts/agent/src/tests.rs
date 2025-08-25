use crate::contract::{exec_vote_agent, execute_add_agent, execute_update_agent, instantiate};
use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::{AGENTS, AGENT_VOTES, CONFIG};
use crate::types::{AgentInput, AgentStatus, AgentType};
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{Addr, DepsMut, Response, Uint128};

const OWNER: &str = "owner";
const USER1: &str = "user1";
const USER2: &str = "user2";
const USER3: &str = "user3";
const USER4: &str = "user4";

fn create_test_agent_input() -> AgentInput {
    AgentInput {
        agent_type: AgentType::Nra,
        name: "Test Agent".to_string(),
        email: "test@example.com".to_string(),
        jurisdictions: vec!["US".to_string(), "EU".to_string()],
        endpoint: Some("https://api.example.com".to_string()),
        metadata_json: Some(r#"{"key": "value"}"#.to_string()),
        docs_uri: vec!["https://docs.example.com".to_string()],
        discord: Some("testuser#1234".to_string()),
        status: AgentStatus::Pending,
        avg_cu: Uint128::new(1000),
    }
}

fn instantiate_contract(deps: DepsMut) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        threshold: Some(3),
        paused: Some(false),
    };
    let info = message_info(&Addr::unchecked(OWNER), &[]);
    instantiate(deps, mock_env(), info, msg)
}

#[test]
fn test_instantiate_with_defaults() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.threshold, 3); // Default threshold
    assert!(!config.paused); // Default paused
}

#[test]
fn test_create_agent_success() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    let agent_input = create_test_agent_input();
    let info = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info, agent_input.clone()).unwrap();

    // Check agent is saved correctly
    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.agent_type, AgentType::Nra);
    assert_eq!(agent.wallet, USER1);
    assert_eq!(agent.name, "Test Agent");
    assert_eq!(agent.email, "test@example.com");
    assert_eq!(agent.jurisdictions, vec!["US", "EU"]);
    assert_eq!(agent.status, AgentStatus::Pending);

    // Check token ID is incremented
    let config = CONFIG.load(&deps.storage).unwrap();
    println!("config = {:?}", config);
    assert_eq!(config.last_token_id, Uint128::new(1));
}

#[test]
fn test_update_agent_success() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent first
    let agent_input = create_test_agent_input();
    let info = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info.clone(), agent_input).unwrap();

    // Update agent
    let mut updated_input = create_test_agent_input();
    updated_input.name = "Updated Agent".to_string();
    updated_input.status = AgentStatus::OnHold;

    execute_update_agent(
        deps.as_mut(),
        mock_env(),
        info,
        "1".to_string(),
        updated_input,
    )
    .unwrap();

    // Check agent is updated
    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.name, "Updated Agent");
    assert_eq!(agent.status, AgentStatus::OnHold);
    assert_eq!(agent.wallet, USER1); // Should remain the same
}

#[test]
fn test_vote_agent() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1, agent_input).unwrap();

    // Vote for approval
    let info2 = message_info(&Addr::unchecked(USER2), &[]);
    exec_vote_agent(
        deps.as_mut(),
        mock_env(),
        info2,
        "1".to_string(),
        true,
        Some("Good agent".to_string()),
    )
    .unwrap();

    // Check vote is saved
    let vote = AGENT_VOTES
        .load(&deps.storage, ("1", &Addr::unchecked(USER2)))
        .unwrap();
    assert_eq!(vote.address, USER2);
    assert!(vote.approve);
    assert_eq!(vote.reason, Some("Good agent".to_string()));
}

#[test]
fn test_vote_agent_self_vote() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1.clone(), agent_input).unwrap();

    // Try to vote for own agent
    let err = exec_vote_agent(
        deps.as_mut(),
        mock_env(),
        info1,
        "1".to_string(),
        true,
        None,
    )
    .unwrap_err();

    assert!(matches!(err, ContractError::SelfVote {}));
}

#[test]
fn test_vote_agent_already_voted() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1, agent_input).unwrap();

    // Vote first time
    let info2 = message_info(&Addr::unchecked(USER2), &[]);

    exec_vote_agent(
        deps.as_mut(),
        mock_env(),
        info2.clone(),
        "1".to_string(),
        true,
        None,
    )
    .unwrap();

    // Try to vote second time
    let err = exec_vote_agent(
        deps.as_mut(),
        mock_env(),
        info2,
        "1".to_string(),
        false,
        None,
    )
    .unwrap_err();

    assert!(matches!(err, ContractError::AlreadyVoted {}));
}

#[test]
fn test_agent_approval() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1, agent_input).unwrap();

    // Vote 1 - should not approve yet (threshold is 3)
    let info2 = message_info(&Addr::unchecked(USER2), &[]);
    exec_vote_agent(
        deps.as_mut(),
        mock_env(),
        info2,
        "1".to_string(),
        true,
        None,
    )
    .unwrap();

    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.status, AgentStatus::Pending);

    // Vote 2 - still pending
    let info3 = message_info(&Addr::unchecked(USER3), &[]);
    exec_vote_agent(
        deps.as_mut(),
        mock_env(),
        info3,
        "1".to_string(),
        true,
        None,
    )
    .unwrap();

    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.status, AgentStatus::Pending);

    // Vote 3 - should approve now
    let info4 = message_info(&Addr::unchecked(USER4), &[]);
    exec_vote_agent(
        deps.as_mut(),
        mock_env(),
        info4,
        "1".to_string(),
        true,
        None,
    )
    .unwrap();

    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.status, AgentStatus::Approved);
}

#[test]
fn test_agent_rejection() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1, agent_input).unwrap();

    // Single reject vote should reject the agent
    let info2 = message_info(&Addr::unchecked(USER2), &[]);
    exec_vote_agent(
        deps.as_mut(),
        mock_env(),
        info2,
        "1".to_string(),
        false,
        Some("Not qualified".to_string()),
    )
    .unwrap();

    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.status, AgentStatus::Rejected);
}
