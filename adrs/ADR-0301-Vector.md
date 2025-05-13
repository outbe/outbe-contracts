# [ARD.0301] Vector

## Status

PROPOSED

## Version

0001 — Draft, Initial specification

Change Log:

## Context

Vector is an entity within the Tribute Issuance Protocol that determines
how the Minting Pool is distributed among Users.
Users join a Vector by selecting a Performance Rate, defining the minimum percentage price increase
above Tribute Price Threshold (TPT) they commit to waiting for before claiming Tribute.
The selected Performance Rate is used to calculate the Tribute Price Floor (TPF).
If the Minting Pool is limited, Users in pools with higher Performance Rates receive priority,
rewarding commitments to long-term value growth.

### Glossary

* **Consumption Unit** – is NFT; an aggregate record that represents user's consumption by a given time range.
* **Tribute** – is a cw721-like Smart Contract to hold a locked "gain" value.
* **Tribute Price Floor** – (TPF) is an adjusted price value of the native token at the current time.
* **Tribute Price Threshold** – (TPT) is a kind of target price of the native token.
* **Vector** – is a value chosen by a user that reflects the trade-off between waiting time of the gain and the
  probability of winning.
* **Raffle** – a process of releasing Consumption Unit into Tribute.

## Scope

In the scope of this document are the following points:

* Define a high-level overview of the Vector.
* Define Vector structure and smart contract.

## Decision

To implement the above smart contracts to fulfill the requirements.
The proposed solution is to have a smart contract that encapsulates vector options and
info required to make a raffle.

## Solution

### Vector Smart Contract

#### Vector Entity Data

```rust
/// Vector
pub struct VectorData {
    /// Identifier
    pub vector_id: u16,
    /// Vector name
    pub name: String,
    /// TPT+%
    pub tpt_percent_increase: Uint128,
    /// Winning Probability weight
    pub weight: Uint128,
}
```

#### Vector Write API

Write API is not needed for this contract such as tiers are a kind of static dictionary.
In case of need they can be updated using the migration mechanism by the contract's owner.

#### Vector Read API

```rust
pub enum QueryMsg {
    /// Returns all vector tiers
    #[returns(AllVectorsResponse)]
    Vectors {},
}

pub struct AllVectorResponse {
    pub vectors: Vec<VectorData>
}
```
