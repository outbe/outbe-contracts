# Token Allocator Contract

This contract is designed to allocate tokens over time based on an exponential decay rate. It provides a mechanism for distributing tokens gradually, potentially rewarding users or contributors over a defined period.

## Features

*   **Configurable Emission Rate:** The initial token emission rate can be adjusted.
*   **Exponential Decay:**  The token allocation follows an exponential decay curve, reducing the amount of tokens allocated per block as time progresses.
*   **Ownership Management:** Uses `cw-ownable` for contract ownership and administrative control.

## Contract Structure

The project is structured with the following key components:

*   `src/contract.rs`: Contains the core contract logic, including instantiation, execution, and migration functions.
*   `src/error.rs`: Defines custom error types for the contract.
*   `src/msg.rs`:  Defines message structures for instantiating, executing, and migrating the contract.
*   `src/query.rs`: Implements query functionality to retrieve contract data (e.g., current token allocation amount).
*   `src/state.rs`: Manages the contract's state variables, including the creator address.
*   `src/types.rs`: Defines custom data structures used within the contract.

## Usage

### Instantiation

The contract is instantiated with an optional `creator` address. If no creator is provided during instantiation, the sender of the transaction will be set as the owner.

```toml
{
    "creator": "outbe1..." // Optional: Address of the contract creator
}
```

### Querying

You can query the contract to retrieve information such as:

*   **Current Token Allocation Amount:**  `GetData {}` - Returns the amount of tokens allocated based on the current block height and emission rate.
*   **Creator Ownership:** `GetCreatorOwnership {}` - Returns the address of the contract owner.
*   **Total Token Allocation for Block Range:** `GetRangeData { from_block, to_block }` - Returns the sum of token allocation amounts for blocks in the specified range (inclusive).

#### Example JSON Query
```json
{
  "get_range_data": {
    "from_block": "1000",
    "to_block": "2000"
  }
}
```

#### Example JSON Response
```json
{
  "amount": "<total tokens emitted from block 1000 through 2000>"
}
```


## Development & Testing

The project utilizes `cw-multi-test` for unit testing. You can run tests using:

```bash
cargo test
```
