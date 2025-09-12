# Random Oracle Smart Contract

## Overview

The Random Oracle smart contract is a utility component of the outbe ecosystem that provides pseudo-random number
generation services for other smart contracts. It serves as a centralized source of randomness that can be used for
various purposes, including tribute selection, lottery systems, and any operations requiring unpredictable values.

The contract offers both deterministic randomness (for testing and debugging) and block-height-based pseudo-randomness (
for production use), making it suitable for both development and production environments.

## Key Concepts

### Randomness Sources

- **Stored Random Value**: Optionally set deterministic value for predictable behavior
- **Block Height Fallback**: Uses blockchain block height as an entropy source when no stored value exists
- **Range-Based Generation**: Generates random numbers within specified ranges
- **Multiple Values**: Can generate multiple random numbers in a single query

### Use Cases

- **Tribute Selection**: Used by Metadosis contract for random tribute processing
- **Lottery Systems**: Provides randomness for fair selection processes
- **Testing**: Deterministic values enable predictable test scenarios
- **Seeding**: Provides seed values for external random number generators

## Business Logic

### Random Value Generation

The contract generates pseudo-random numbers using a simple but effective algorithm:

1. **Seed Selection**: Uses either stored random value or current block height as base seed
2. **Range Calculation**: Computes the range size (to_range - from_range)
3. **Value Generation**: For each requested value:
    - Adds index to seed for uniqueness
    - Applies modulo operation to fit within range
    - Adds from_range offset to achieve desired range

### Deterministic Mode

When a random value is explicitly set via `SetRandom`:

- All subsequent queries use this fixed seed
- Enables reproducible test scenarios
- Useful for debugging and development
- Can be reset by calling `SetRandom` with `None`

### Block Height Mode

When no stored random value exists:

- Uses current blockchain block height as entropy
- Provides reasonable pseudo-randomness for production
- Changes with each new block
- Simple but effective for most use cases

## Technical Architecture

### State Management

The contract maintains minimal state:

- **RND**: Optional stored random seed value (u64)

### Core Functions

- **Instantiate**: Optionally sets initial random value
- **SetRandom**: Updates or removes the stored random seed
- **RandomValue**: Generates random numbers within specified range
- **RandomSeed**: Returns the current seed value

## API Reference

### Messages

#### InstantiateMsg

```rust
pub struct InstantiateMsg {
    pub random_value: Option<u64>,  // Optional initial seed
}
```

#### ExecuteMsg

```rust
pub enum ExecuteMsg {
    /// Sets a predictable value as "random" or removes if None
    SetRandom { random_value: Option<u64> },
}
```

#### QueryMsg

```rust
pub enum QueryMsg {
    /// Returns pseudo random values within specified range
    RandomValue {
        from_range: u64,      // Minimum value (inclusive)
        to_range: u64,        // Maximum value (exclusive) 
        count_values: u64,    // Number of values to generate
    },
    /// Returns the current random seed
    RandomSeed {},
}
```

### Response Types

#### RandomResponse

```rust
pub struct RandomResponse {
    pub random_values: Vec<u64>,  // Generated random values
}
```

#### SeedResponse

```rust
pub struct SeedResponse {
    pub seed: u64,  // Current seed value
}
```

## Mathematical Model

### Random Number Generation Algorithm

```rust
fn generate_random_values(seed: u64, from: u64, to: u64, count: u64) -> Vec<u64> {
    let range = to - from;
    let mut result = Vec::new();

    for i in 0..count {
        let value = (seed + i) % range + from;
        result.push(value);
    }

    result
}
```

### Properties

- **Deterministic**: Same seed and parameters always produce same results
- **Uniform Distribution**: Values are evenly distributed within range
- **Sequential**: Multiple values are generated using incrementing offsets
- **Range Flexible**: Supports any valid u64 range

## Usage Examples

### Contract Instantiation

```rust
// With initial seed
let msg = InstantiateMsg {
random_value: Some(12345),
};

// Without initial seed (uses block height)
let msg = InstantiateMsg {
random_value: None,
};
```

### Setting Random Seed

```rust
// Set deterministic seed for testing
let msg = ExecuteMsg::SetRandom {
    random_value: Some(98765),
};

// Remove seed (revert to block height)
let msg = ExecuteMsg::SetRandom {
    random_value: None,
};
```

### Querying Random Values

```rust
// Generate single random number between 1-100
let query = QueryMsg::RandomValue {
   from_range: 1,
   to_range: 101,
   count_values: 1,
};

// Generate 10 random numbers between 0-999
let query = QueryMsg::RandomValue {
   from_range: 0,
   to_range: 1000,
   count_values: 10,
};

// Get current seed
let query = QueryMsg::RandomSeed {};
```

### Integration Example (Metadosis)

```rust
// Query random oracle from another contract
let random_response: RandomResponse = deps.querier.query_wasm_smart(
   &random_oracle_addr,
   &QueryMsg::RandomValue {
      from_range: 0,
      to_range: tribute_count,
      count_values: 1,
   },
)?;

let selected_index = random_response.random_values[0];
```

## Deployment

### Prerequisites

- CosmWasm-compatible blockchain
- Sufficient gas for contract operations
- No external dependencies required

### Configuration Steps

1. Deploy the contract with optional initial seed
2. Configure consuming contracts with oracle address
3. Set deterministic seed for testing environments
4. Verify randomness behavior in development

### Environment Considerations

- **Development**: Use deterministic seeds for predictable testing
- **Staging**: Mix of deterministic and block-height randomness
- **Production**: Rely on block-height entropy or external entropy sources

## Testing

The contract includes comprehensive tests covering:

- Seed storage and retrieval functionality
- Random value generation within specified ranges
- Block height fallback behavior
- Multiple value generation accuracy
- Range validation and error handling

Run tests using:

```bash
cargo test
```

### Test Cases

```rust
#[test]
fn test_deterministic_randomness() {
    // Test that same seed produces same values
}

#[test]
fn test_block_height_fallback() {
    // Test fallback to block height when no seed stored
}

#[test]
fn test_range_validation() {
    // Test proper range validation and error handling
}
```

## Integration Notes

### Consumer Contracts

The Random Oracle is designed to be consumed by other contracts in the ecosystem:

- **Metadosis**: Uses for tribute selection randomness
- **Lysis Operations**: Provides randomness for recognition processes
- **Testing Frameworks**: Enables deterministic test scenarios

### Best Practices

- Use deterministic seeds only for testing
- Validate range parameters before querying
- Consider gas costs for multiple value generation
- Implement proper error handling for invalid ranges

### Limitations

- **Pseudo-Random**: Not cryptographically secure
- **Predictable**: Block height entropy is somewhat predictable
- **Simple Algorithm**: Uses basic modulo operation
- **No True Entropy**: Relies on blockchain state for randomness

## Security Considerations

### Randomness Quality

- Block height provides basic pseudo-randomness
- Not suitable for high-security applications requiring true randomness
- Predictable by validators who know future block heights
- Sufficient for most gaming and selection use cases

### Access Control

- No built-in access restrictions on SetRandom
- Consider implementing ownership controls in production
- Protect against malicious seed manipulation
- Monitor for suspicious randomness patterns

### Recommendations

- Use for non-critical randomness requirements only
- Implement additional entropy sources for security-sensitive applications
- Consider using commit-reveal schemes for sensitive operations
- Regular audits of randomness distribution patterns

## Error Handling

### Common Errors

- `WrongInput`: Invalid range parameters (from_range >= to_range)
- Standard CosmWasm errors for storage and serialization issues

### Error Prevention

- Always ensure `from_range < to_range`
- Validate count_values parameter to prevent excessive gas usage
- Handle query failures gracefully in consuming contracts
