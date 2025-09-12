# Agent Registry Smart Contract

## Overview

The Agent Registry smart contract is a governance and management component of the Q-Contracts ecosystem that maintains a decentralized registry of authorized agents within the network. It serves as the central authority for agent registration, approval processes, and account management, implementing a threshold-based voting mechanism to ensure democratic governance of agent participation.

The contract manages different types of agents including Network Registry Agents (NRA), Consumption Registry Agents (CRA), Regulatory Filing Agents (RFA), Independent Banking Agents (IBA), and Consumer Credit Agents (CCA), each serving specific roles in the broader financial ecosystem.

## Key Concepts

### Agent Types

- **NRA (Network Registry Agent)**: Responsible for network-level registry operations
- **CRA (Consumption Registry Agent)**: Manages consumption data and validation
- **RFA (Regulatory Filing Agent)**: Handles regulatory compliance and filing
- **IBA (Independent Banking Agent)**: Provides independent banking services
- **CCA (Consumer Credit Agent)**: Manages consumer credit operations

### Governance Mechanisms

- **Threshold Voting**: Configurable voting threshold for agent approval decisions
- **Democratic Process**: Community-driven agent registration and validation
- **Status Management**: Comprehensive agent lifecycle from pending to approved/rejected
- **Account Integration**: Automatic account creation for approved agents

### Agent Lifecycle

- **Registration**: Agents submit applications with comprehensive metadata
- **Voting Period**: Community members vote on agent applications
- **Status Determination**: Based on voting threshold and community consensus
- **Account Management**: Approved agents receive operational accounts

## Business Logic

### Agent Registration Process

1. **Application Submission**: Potential agents submit detailed applications including:
   - Agent type and specialization
   - Contact information and jurisdictions
   - Technical endpoints and documentation
   - Average consumption units (CU) capacity
   - Supporting documentation and metadata

2. **Community Review**: Registered community members review applications
3. **Voting Process**: Stakeholders vote to approve or reject applications
4. **Threshold Evaluation**: Applications are approved based on configured voting thresholds
5. **Account Creation**: Approved agents automatically receive operational accounts

### Voting Mechanism

The contract implements a sophisticated voting system:
- **Eligible Voters**: Only registered community members can vote
- **Vote Recording**: All votes are permanently recorded with timestamps and reasons
- **Threshold Logic**: Configurable approval thresholds determine outcomes
- **Transparency**: Complete voting history is publicly queryable
- **Anti-Gaming**: Agents cannot vote on their own applications

### Account Management

Upon agent approval, the system automatically:
- Creates corresponding operational accounts
- Inherits agent metadata and configuration
- Sets appropriate account status and permissions
- Maintains linkage between agents and accounts for operational purposes

### Status Transitions

**Agent Status Flow**:
```
Pending → Approved/Rejected/Recalled
```

**Account Status Options**:
- **Approved**: Full operational access
- **Blacklisted**: Restricted access due to violations
- **OnHold**: Temporary suspension pending review

## Technical Architecture

### State Management

- **AGENTS**: Map storing agent records by unique ID
- **AGENT_VOTES**: Composite map tracking votes by agent ID and voter address
- **ACCOUNTS**: Map linking addresses to operational accounts
- **CONFIG**: Global configuration including voting thresholds and pause state

### Core Data Structures

#### Agent Structure
```rust
pub struct Agent {
    pub id: u32,                        // Unique agent identifier
    pub agent_type: AgentType,          // Type of agent (NRA, CRA, etc.)
    pub wallet: Addr,                   // Associated wallet address
    pub name: String,                   // Agent display name
    pub email: String,                  // Contact email
    pub jurisdictions: Vec<String>,     // Operating jurisdictions
    pub endpoint: Option<String>,       // API endpoint URL
    pub metadata_json: Option<String>,  // Additional metadata
    pub docs_uri: Vec<String>,          // Documentation URLs
    pub discord: Option<String>,        // Discord contact
    pub status: AgentStatus,            // Current status
    pub avg_cu: Uint128,               // Average consumption units
    pub submitted_at: Timestamp,        // Application timestamp
    pub updated_at: Timestamp,          // Last update timestamp
}
```

#### Vote Structure
```rust
pub struct Vote {
    pub address: String,           // Voter address
    pub agent_id: String,          // Target agent ID
    pub approve: bool,             // Vote decision
    pub reason: Option<String>,    // Optional justification
    pub at: Timestamp,            // Vote timestamp
}
```

### Governance Configuration

```rust
pub struct Config {
    pub owner: Addr,           // Contract owner
    pub threshold: u8,         // Approval threshold percentage
    pub paused: bool,          // Emergency pause state
    pub last_token_id: u32,    // ID counter for agents
}
```

## API Reference

### Messages

#### InstantiateMsg
```rust
pub struct InstantiateMsg {
    pub threshold: Option<u8>,    // Voting threshold (default: configurable)
    pub paused: Option<bool>,     // Initial pause state (default: false)
}
```

#### ExecuteMsg
```rust
pub enum ExecuteMsg {
    CreateAgent {
        agent: AgentInput,
    },
    UpdateAgent {
        id: String,
        agent: AgentInput,
    },
    VoteAgent {
        id: String,
        approve: bool,
        reason: Option<String>,
    },
    UpdateAccount {
        account: AccountInput,
    },
    ChangeAccountStatus {
        address: Addr,
        status: AccountStatus,
        reason: Option<String>,
    },
}
```

#### QueryMsg
```rust
pub enum QueryMsg {
    ListAll {
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
    GetById { 
        id: String 
    },
    QueryByAddress {
        address: String,
        start_after: Option<String>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
    QueryVotesByAgent { 
        id: String 
    },
    QueryVotesByAddress { 
        address: Addr 
    },
    GetAccountByAddress { 
        address: Addr 
    },
    ListAllAccounts {
        start_after: Option<Addr>,
        limit: Option<u32>,
        query_order: Option<Order>,
    },
}
```

### Response Types

#### Agent Queries
```rust
pub struct ListAllResponse {
    pub agents: Vec<Agent>,
}

pub struct AgentResponse {
    pub agent: Agent,
}
```

#### Voting Queries
```rust
pub struct AgentVotesResponse {
    pub votes: Vec<Vote>,
}
```

#### Account Queries
```rust
pub struct AccountResponse {
    pub account: Account,
}

pub struct ListAllAccountsResponse {
    pub accounts: Vec<Account>,
}
```

## Usage Examples

### Contract Instantiation
```rust
let msg = InstantiateMsg {
    threshold: Some(60),  // 60% approval threshold
    paused: Some(false),  // Start in active state
};
```

### Agent Registration
```rust
let create_msg = ExecuteMsg::CreateAgent {
    agent: AgentInput {
        agent_type: AgentType::Cra,
        name: "Example Consumption Agent".to_string(),
        email: "agent@example.com".to_string(),
        jurisdictions: vec!["EU".to_string(), "US".to_string()],
        endpoint: Some("https://api.agent.example.com".to_string()),
        metadata_json: Some("{\"category\":\"financial\"}".to_string()),
        docs_uri: vec!["https://docs.agent.example.com".to_string()],
        discord: Some("agent_support".to_string()),
        status: AgentStatus::Pending,
        avg_cu: Uint128::new(1000000),
    },
};
```

### Voting on Agent Applications
```rust
let vote_msg = ExecuteMsg::VoteAgent {
    id: "1".to_string(),
    approve: true,
    reason: Some("Well-qualified agent with strong documentation".to_string()),
};
```

### Account Status Management
```rust
let status_msg = ExecuteMsg::ChangeAccountStatus {
    address: Addr::unchecked("agent_wallet_address"),
    status: AccountStatus::OnHold,
    reason: Some("Pending compliance review".to_string()),
};
```

### Querying Agents
```rust
// List all agents with pagination
let query_msg = QueryMsg::ListAll {
    start_after: None,
    limit: Some(50),
    query_order: Some(Order::Ascending),
};

// Get specific agent by ID
let query_msg = QueryMsg::GetById {
    id: "1".to_string(),
};

// Query voting history for an agent
let query_msg = QueryMsg::QueryVotesByAgent {
    id: "1".to_string(),
};
```

## Deployment

### Prerequisites

- CosmWasm-compatible blockchain environment
- Governance framework for community voting
- Agent onboarding process and documentation
- Compliance and regulatory framework

### Configuration Steps

1. **Deploy Contract**: Set appropriate voting threshold and initial state
2. **Configure Governance**: Establish community voting procedures
3. **Agent Onboarding**: Create registration process and documentation
4. **Integration Setup**: Connect with other ecosystem contracts
5. **Monitoring Tools**: Implement agent performance and compliance tracking

### Security Considerations

1. **Voting Integrity**: Prevent vote manipulation and gaming
2. **Agent Validation**: Thorough vetting of agent applications
3. **Access Control**: Proper permission management for sensitive operations
4. **Audit Requirements**: Regular reviews of agent performance and compliance
5. **Emergency Procedures**: Pause functionality for critical situations

## Governance Model

### Voting Threshold Configuration

The contract supports configurable voting thresholds to balance:
- **Democratic Participation**: Lower thresholds encourage broader participation
- **Quality Control**: Higher thresholds ensure rigorous agent selection
- **Network Security**: Appropriate thresholds prevent malicious agent approval

### Community Participation

- **Stakeholder Engagement**: Active community involvement in agent selection
- **Transparency**: Complete voting history and reasoning publicly available
- **Accountability**: Traceable decisions with timestamp and reasoning
- **Continuous Improvement**: Threshold and process refinement based on outcomes

## Testing

The contract includes comprehensive tests covering:

- Agent registration and update workflows
- Voting mechanism accuracy and threshold enforcement
- Account creation and status management
- Query functionality across all data types
- Edge cases and error conditions

Run tests using:
```bash
cargo test
```

### Key Test Scenarios

```rust
#[test]
fn test_agent_registration_workflow() {
    // Test complete agent onboarding process
}

#[test]
fn test_voting_threshold_enforcement() {
    // Verify threshold-based approval logic
}

#[test]
fn test_account_auto_creation() {
    // Test automatic account generation for approved agents
}

#[test]
fn test_anti_self_voting() {
    // Verify agents cannot vote on their own applications
}
```

## Integration Notes

### Ecosystem Integration

The Agent Registry integrates with other Q-Contracts components:

- **Tribute System**: Approved agents participate in tribute validation
- **Oracle Networks**: Agents provide external data feeds
- **Governance Contracts**: Democratic decision-making for network parameters
- **Compliance Systems**: Regulatory reporting and audit trails

### External Dependencies

- **Identity Verification**: Integration with KYC/AML systems
- **Performance Monitoring**: Agent activity and reliability tracking
- **Compliance Reporting**: Regulatory filing and audit support
- **Communication Systems**: Discord and email integration for notifications

### Best Practices

- **Regular Reviews**: Periodic assessment of agent performance
- **Community Engagement**: Active participation in voting processes
- **Documentation Maintenance**: Keep agent information current and accurate
- **Security Monitoring**: Continuous surveillance for suspicious activity
- **Compliance Adherence**: Maintain regulatory compliance across jurisdictions

## Error Handling

### Common Errors

- `OnlyWalletOwnerCanCreateAgent`: Prevents unauthorized agent creation
- `AgentNotFound`: Invalid agent ID references
- `CannotVoteForSelf`: Prevents self-voting on applications
- Standard CosmWasm errors for storage and validation issues

### Error Prevention

- Validate agent input data thoroughly
- Implement proper access controls for sensitive operations
- Provide clear error messages for troubleshooting
- Handle edge cases gracefully in voting and status transitions

## Security Considerations

### Voting Security

- **Anti-Manipulation**: Prevent coordinated voting attacks
- **Identity Verification**: Ensure legitimate voter participation
- **Audit Trails**: Maintain complete voting history for transparency
- **Threshold Protection**: Set appropriate approval thresholds

### Agent Security

- **Background Verification**: Thorough vetting of agent applications
- **Ongoing Monitoring**: Continuous performance and compliance assessment
- **Revocation Mechanisms**: Ability to recall or suspend problematic agents
- **Data Protection**: Secure handling of agent sensitive information

### Contract Security

- **Access Control**: Proper permission management for administrative functions
- **State Validation**: Comprehensive input validation and state consistency
- **Emergency Controls**: Pause functionality for critical situations
- **Upgrade Safety**: Secure migration procedures for contract updates

## License

This contract is part of the Q-Contracts ecosystem. See the main repository for licensing information.
