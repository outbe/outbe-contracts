use crate::contract::{
    exec_vote_agent, execute_add_agent, execute_change_account_status, execute_update_account,
    execute_update_agent, instantiate,
};
use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::{ACCOUNTS, AGENTS, AGENT_VOTES, CONFIG};
use crate::types::{AccountInput, AccountStatus, AgentInput, AgentStatus, AgentType};
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

fn create_test_account_input() -> AccountInput {
    AccountInput {
        name: "Updated Account Name".to_string(),
        email: "updated@example.com".to_string(),
        jurisdictions: vec!["US".to_string(), "CA".to_string()],
        endpoint: Some("https://updated-api.example.com".to_string()),
        metadata_json: Some(r#"{"updated": "metadata"}"#.to_string()),
        docs_uri: vec!["https://updated-docs.example.com".to_string()],
        discord: Some("updateduser#5678".to_string()),
        avg_cu: Uint128::new(2000),
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

    // Check agent-registry is saved correctly
    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.agent_type, AgentType::Nra);
    assert_eq!(agent.wallet.to_string(), USER1);
    assert_eq!(agent.name, "Test Agent");
    assert_eq!(agent.email, "test@example.com");
    assert_eq!(agent.jurisdictions, vec!["US", "EU"]);
    assert_eq!(agent.status, AgentStatus::Pending);

    // Check token ID is incremented
    let config = CONFIG.load(&deps.storage).unwrap();
    println!("config = {:?}", agent);
    assert_eq!(config.last_token_id, 2u32);
}

#[test]
fn test_update_agent_success() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent-registry first
    let agent_input = create_test_agent_input();
    let info = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info.clone(), agent_input).unwrap();

    // Update agent-registry
    let mut updated_input = create_test_agent_input();
    updated_input.name = "Updated Agent".to_string();
    updated_input.status = AgentStatus::Recalled;

    execute_update_agent(
        deps.as_mut(),
        mock_env(),
        info,
        "1".to_string(),
        updated_input,
    )
    .unwrap();

    // Check agent-registry is updated
    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.name, "Updated Agent");
    assert_eq!(agent.status, AgentStatus::Recalled);
    assert_eq!(agent.wallet.to_string(), USER1); // Should remain the same
}

#[test]
fn test_vote_agent() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent-registry
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
        Some("Good agent-registry".to_string()),
    )
    .unwrap();

    // Check vote is saved
    let vote = AGENT_VOTES
        .load(&deps.storage, ("1", &Addr::unchecked(USER2)))
        .unwrap();
    assert_eq!(vote.address, USER2);
    assert!(vote.approve);
    assert_eq!(vote.reason, Some("Good agent-registry".to_string()));
}

#[test]
fn test_vote_agent_self_vote() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent-registry
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1.clone(), agent_input).unwrap();

    // Try to vote for own agent-registry
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

    // Create agent-registry
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

    // Create agent-registry
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1, agent_input).unwrap();

    // Vote 1 - should not approve yet (a threshold is 3)
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

    // Create agent-registry
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1, agent_input).unwrap();

    // Single reject vote should reject the agent-registry
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

#[test]
fn test_account_created_on_agent_approval() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create agent
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1, agent_input).unwrap();

    // Vote 3 times to approve agent (threshold is 3)
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

    // Check that agent is approved
    let agent = AGENTS.load(&deps.storage, "1".to_string()).unwrap();
    assert_eq!(agent.status, AgentStatus::Approved);

    // Check that account was created automatically
    let account = ACCOUNTS
        .load(&deps.storage, Addr::unchecked(USER1))
        .unwrap();
    assert_eq!(account.name, "Test Agent");
    assert_eq!(account.email, "test@example.com");
    assert_eq!(account.status, AccountStatus::Approved);
    assert_eq!(account.agent_type, AgentType::Nra);
}

#[test]
fn test_update_account_success() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create and approve agent first (to create account)
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1.clone(), agent_input).unwrap();

    // Approve agent (create account)
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

    // Update account
    let account_input = create_test_account_input();
    execute_update_account(deps.as_mut(), mock_env(), info1, account_input).unwrap();

    // Check account is updated
    let account = ACCOUNTS
        .load(&deps.storage, Addr::unchecked(USER1))
        .unwrap();
    assert_eq!(account.name, "Updated Account Name");
    assert_eq!(account.email, "updated@example.com");
    assert_eq!(account.jurisdictions, vec!["US", "CA"]);
    assert_eq!(account.avg_cu, Uint128::new(2000));
    assert_eq!(account.status, AccountStatus::Approved); // Status should remain unchanged
}

#[test]
fn test_change_account_status_success() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Create and approve agent first (to create account)
    let agent_input = create_test_agent_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);
    execute_add_agent(deps.as_mut(), mock_env(), info1, agent_input).unwrap();

    // Approve agent (create account)
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

    // Check initial status
    let account_before = ACCOUNTS
        .load(&deps.storage, Addr::unchecked(USER1))
        .unwrap();
    assert_eq!(account_before.status, AccountStatus::Approved);

    // Change account status to Blacklisted
    execute_change_account_status(
        deps.as_mut(),
        mock_env(),
        Addr::unchecked(USER1),
        AccountStatus::Blacklisted,
        Some("Violation of terms".to_string()),
    )
    .unwrap();

    // Check account status is updated
    let account_after = ACCOUNTS
        .load(&deps.storage, Addr::unchecked(USER1))
        .unwrap();
    assert_eq!(account_after.status, AccountStatus::Blacklisted);
}

#[test]
fn test_update_account_not_found() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut()).unwrap();

    // Try to update account that doesn't exist
    let account_input = create_test_account_input();
    let info1 = message_info(&Addr::unchecked(USER1), &[]);

    let err = execute_update_account(deps.as_mut(), mock_env(), info1, account_input).unwrap_err();
    assert!(matches!(err, ContractError::AccountNotFound {}));
}
