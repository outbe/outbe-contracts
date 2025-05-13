# [ARD.0300] Consumption Unit

## Status

PROPOSED

## Version

0001 — Draft, Initial specification

Change Log:

## Context

This ADR defines the functionality required to implement Consumption Unit smart contract.

### Glossary

*   **Consumption Record** – is a user's transaction representation that 1-to-1 corresponds to a user tx.
*   **Consumption Unit** – is NFT; an aggregate record that represents user's consumption by a given time range.
*   **Consumption Unit Oracle** – is a smart contract responsible for creating Consumption Units.
*   **Open Banking Data Agent** (agent in short) – is a trusted counterparty responsible for.

## Scope

In the scope of this document are the following points:

* Define a high-level overview of Consumption Unit and sibling contracts.
* Define Consumption Unit structure.
* Provide technical implementation details of smart contracts.

Out of Scope:

* Consumption Unit validity verification. 
* Consumption Unit lifecycle. It'll be covered in further ADRs.

## Decision

To implement the above smart contracts to fulfill the requirements. 
The proposed solution will have two main smart contracts that are wired together: Consumption Unit and
Consumption Unit Oracle which will be responsible for creating (minting) Consumption Unit NFTs.

The target solution is to be implemented in 2 steps:

1. Implement Consumption Unit smart contract **without** mint restrictions i.e. anyone can create new units.
2. Implement Consumption Unit Oracle smart contract with user and agent multisig verification and link it to 
the Consumption Unit smart contract so only Oracle can mint such.

## Solution

### Consumption Unit Smart Contract

Consumption Unit smart contract should be compatible with the standard CW721 token and represented in a way of 
non-transferable NFT.

#### Config

```rust
/// ConsumptionUnit Smart Contract Config
pub  struct Config {
    /// Settlement Token
    pub settlement_token: String,
    /// Address of the price Oracle to query floor prices
    pub price_oracle: Addr,
}
```

#### Consumption Unit Entity Data

```rust
/// ConsumptionUnit public data
pub struct ConsumptionUnitData {
    /// The value of Consumption Unit in Settlement Tokens
    pub consumption_value: Uint128,
    /// Sum of Nominal Qty from Consumption Records
    pub nominal_quantity: Uint128,
    /// Nominal currency from Consumption Records
    pub nominal_currency: String,
    /// Where the CU is allocated by the User.
    /// A user can change vector at any time prior to CU NFT selection in raffle
    pub vector: u16,
    /// Calculated according to initial Native Coin Price, PGT and allocated vector.
    /// FloorPrice is to be re-calculated each time out of the update of the vector
    pub floor_price: Decimal,
    /// Hashes identifying consumption records batch
    pub hashes: Vec<String>,
    pub created_at: Timestamp,
}
```

#### Consumption Unit Write API

NB: Keep maximum compatibility with cw721. 

```rust
pub enum ExecuteMsg {
    /// Mint a new NFT, can only be called by the contract minter
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Any custom extension used by this contract
        extension: ConsumptionUnitData,
    },

    /// Burn an NFT the sender has access to
    Burn { token_id: String },

    /// Extension msg
    Extension { msg: ConsumptionUnitExtensionUpdate },
}

pub enum ConsumptionUnitExtensionUpdate {
    UpdateVector {
        token_id: String,
        new_vector_id: u16,
    }
}
```

#### Consumption Unit Read API

NB: Keep maximum compatibility with cw721.

```rust
pub enum QueryMsg<Q: ConsumptionUnitData> {
    /// Return the owner of the given token, error if token does not exist
    #[returns(cw721::OwnerOfResponse)]
    OwnerOf {
        token_id: String,
    },
    /// Total number of tokens issued
    #[returns(cw721::NumTokensResponse)]
    NumTokens {},

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract
    #[returns(cw721::ContractInfoResponse)]
    ContractInfo {},
    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract
    #[returns(cw721::NftInfoResponse<Q>)]
    NftInfo { token_id: String },
    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients
    #[returns(cw721::AllNftInfoResponse<Q>)]
    AllNftInfo {
        token_id: String,
    },

    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    #[returns(cw721::TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    #[returns(cw721::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Return the minter
    #[returns(MinterResponse)]
    Minter {},
}
```

### Consumption Unit Oracle

This section will cover implementation details of the Consumption Unit Oracle smart contract which will be 
responsible for creating Consumption Unit NFTs. The main reason to have a dedicated smart contract for
creating (minting) consumption units is security to make sure data validity and integrity are maintained.

The Oracle smart contract should be implemented as a modified (actually simplified) version of the 
[CW3 standard](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw3/README.md). 

#### Consumption Unit Creation Process

[![Consumption Unit Creation](https://tinyurl.com/2atgwzh5)](https://tinyurl.com/2atgwzh5)<!--![Consumption Unit Creation](puml/ADR-0300-consumption-unit-creation.puml)-->

1. Agent creates a proposal with a list of consumption units to be created.
2. User fetches that data and approves/rejects the proposal.
3. If a User accepts the proposal, then Consumption Unit NFTs are minted instantly by the Oracle on behalf of the user.

#### Entity

```rust
pub struct ProposalMessage {
    /// Owner of the consumption unit i.e. user
    pub owner: Addr,
    /// The value of Consumption Unit in Settlement Tokens
    pub consumption_value: Uint128,
    /// Sum of Nominal Qty from Consumption Records
    pub nominal_quantity: Uint128,
    /// Nominal currency from Consumption Records
    pub nominal_currency: String,
    /// Hash identifying the uniqueness of the consumption unit i.e. consumption records batch
    pub hash: String,
}
```

#### Config

```rust
/// Consumption Unit Oracle Smart Contract Config
pub struct Config {
    /// List of agents who is allowed to propose consumption units   
    pub agents: Vec<Addr>,
    /// Address of the consumption unit smart contract
    pub consumption_unit: Addr,
}
```

#### Consumption Unit Oracle Write API

NB: Keep maximum compatibility with CW3.

```rust
pub enum ExecuteMsg {
    Propose {
        title: String,
        description: String,
        msgs: Vec<ProposalMessage>,
        // note: we ignore API-spec'd earliest if passed, always opens immediately
        latest: Option<Expiration>,
    },
    Vote {
        proposal_id: u64,
        vote: Vote,
    },
    VoteAll {
        proposal_id: Vec<u64>,
        vote: Vote,
    },
    Execute {
        proposal_id: u64,
    },
    ExecuteAll {
        proposal_id: Vec<u64>,
    },
    Close {
        proposal_id: u64,
    },
    CloseAll {
        proposal_id: Vec<u64>,
    },
}
```

#### Consumption Unit Oracle Read API

NB: Keep maximum compatibility with CW3.

```rust
pub enum QueryMsg {
    #[returns(cw3::ProposalResponse)]
    Proposal { proposal_id: u64 },
    #[returns(cw3::ProposalListResponse)]
    ListProposals {
        owner: Addr,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(cw3::ProposalListResponse)]
    ReverseProposals {
        owner: Addr,
        start_before: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(cw3::ProposalListResponse)]
    ListProposalsAll {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(cw3::ProposalListResponse)]
    ReverseProposalsAll {
        start_before: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(cw3::VoterListResponse)]
    ListProposers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
```

## Consequences

- Positive:
    - Controlled, transparent minting
    - Transparency by CosmWasm smart contracts
- Negative:
    - Complexity in accurate implementation
    - Audit required

## Risks

- Smart contract vulnerabilities
