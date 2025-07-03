# [ARD.0302] Tribute on L1

## Status

PROPOSED

## Version

0001 — Draft, Initial specification

## Context

Tribute is a non-fungible token that represents a user consumption reflection for a given worldwide day.
This document aims to describe logic and data fields required for successful management of the Tributes on
L1 outbe chain.

### Glossary

* **Consumption Units** – are bundles of transaction representation aggregated per User's
  Bank Account and Worldwide Day that persisted on L2.
* **Tribute Draft** – are units automatically created on the L2 Network by aggregating Consumption Units (CUs)
  per Account and Worldwide Day. They serve as the pre-stage of any Tribute offered on L1.
* **Tribute** – is NFT; an aggregate record that represents user's consumption by a given worldwide day.

## Decision

We will implement Tribute smart contract using cosmwasm to satisfy the needs to represent a Tribute NFTs
onchain.

The Tribute is published by an end user via Wallet App using Tribute Draft entity on L2 as well as
cryptographic proof of the data validity.

The Tribute itself is a non-fungible non-transferable token that represents act of consumption for the given date
and it's a responsibility of a user to submit (or not submit) the Tribute.

Tribute has no state and status and should be burned after Metadosis run. This functionality will be in-details
described in the following ARDs.

Tribute Draft structure:

```rust
pub struct TributeDraftPayload {
    /// ID of the draft tribute
    pub tribute_draft_id: String,
    /// Owner address
    pub owner: String,
    /// Timestamp value
    pub created_at: u64,
    /// ISO 8601
    pub worldwide_day: String,
    /// ISO 4217
    pub settlement_currency: String,
    pub settlement_base_amount: Uint128,
    pub settlement_atto_amount: Uint128,
    pub nominal_base_qty: Uint128,
    pub nominal_atto_qty: Uint128,
    pub cu_hashes: Vec<String>
}
```

Such a structure will cover all the fields required to represent consumption and help to create Tribute on L1.

Tribute structure:

```rust
pub struct TributePayload {
    /// ISO 8601
    pub worldwide_day: String,
    /// Owner address
    pub owner: String,
    /// Value of the Tribute in Settlement Tokens
    pub settlement_amount_minor: Uint128,
    /// Tribute settlement token
    pub settlement_currency: Denom,
    /// Quantity in native coins
    pub nominal_qty: Uint128,
    /// Price in Native coins with a rate on the moment of the transaction
    pub tribute_price_minor: Decimal,
    /// Divisor to calculate symbolic_load
    pub symbolic_divisor: Decimal,
    /// Signals an eligible interest to the network
    pub symbolic_load: Uint128,
    /// Time when the Tribute NFT was created on the network
    pub created_at: Timestamp,
}
```

### Data Integrity and Verification

To make sure that the provided data is correct the smart contract should have a verification mechanics to check that
the submitted Tribute is correct and not modified.

The target solution for this problem is to use zero knowledge proofs Plonk algorithm verification.

#### Tribute Data Hashing

The tribute hash is needed to make sure that the given Tribute is not modified by third party and verify that the given
Tribute hash is presented in the L2 state.

The proposed schema of the Tributes hashing:

```rust
let prefix = "TRIBUTE";
let tribute_hash = sha256::hash(prefix | tribute.owner | tribute.worldwide_day | tribute.settlement_amount_minor | tribute.settlement_currency);
```

Such as CosmWasm only supports natively `sha256` hash function it's a kind of limitation
to use this hash function to avoid a custom implementations of the cryptography onchain.
See more details at https://cosmwasm.cosmos.network/core/standard-library/cryptography#note-on-hash-functions

#### Tribute Data Verification

To verify that the Tribute is correct, the following data should be submitted together with the Tribute payload:

- zk proof – cryptographic proof of the data correctness on L2
- verification key – cryptographic public key to verify the proof
- tribute hash – sha256 hash that can be calculated on the flight based on the tribute payload.
- owner's signature – such as Tribute is published by the end user i.e., owner it's
  signature verified as a standard tx signature. I.e., it wouldn't be possible to submit tx with the wrong signature.

Zk Proof should be passed as Structured Reference String and based on PlonK alghoritm.

TBD: deep dive research and PoC is required to verify the proofs verification onchain using cosmwasm.
Because cosmwasm natively doesn't support ZK proofs verification at the moment, it can be a challenging task.

Helpful links:

Plonk paper: https://eprint.iacr.org/2019/953.pdf
Zksync umbrella repo for ZK proofs in Rust: https://github.com/matter-labs/zksync-crypto
ZK in cosmwasm examples: https://github.com/DoraFactory/zk-cosmwasm/tree/main

### Related Changes

Such as settlement amount can be any of the `ISO 4217` currency standards we need to extend `Denom` enum with
`Fiat` option that will contain the currency code.

Proposed structure:

```rust
pub enum Denom {
    Native(String),
    Cw20(Addr),
    Fiat(Iso4217)
}

pub enum Iso4217 {
    Usd,
    Eur,
... .
}
```

And the corresponding changes should be made to Price Oracle so it contains the pairs between settlement currency and
native token.

## Solution

Implement Tribute smart contract to cover described functionality. It should be compatible with CW721 standard
for NFTs, non-transferable. For such a purpose the API can be reused and remains the same from the previous
version of the tribute described in [ADR-0300](ADR-0300-Tribute.md).

## Open Questions

Q: Tribute Draft: Should it have `nominal_base_qty` i.e. amount in native coins?
A: Looks like this property should be calculated on L1 using Price Oracle when Tribute is minting.

Q: Tribute Draft: Should it have separate properties for `settlement_base_amount` and `settlement_atto_amount`?  
A: It looks like overcomplicated, and the amount can be stored in the smallest currency units,
for example, for 123.45 Euro -> 12345 integer value in euro cents.

Q: Tribute: should a mint operation be named `offer`?
A: TBD. It brakes compatibility with CW721 standard because creating a new tokens is `mint` operation.

