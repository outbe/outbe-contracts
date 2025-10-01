# Metadosis Smart Contract

## Overview

The Metadosis smart contract is a core component of the Q-Contracts ecosystem that manages Network Limits and
orchestrates daily recognition processes. It serves as the central coordinator for calculating emission limits, managing
daily Lysis and Touch runs, and distributing network resources according to sophisticated economic models.

**Metadosis Day** begins 36 hours after the end of Worldwide Day at UTC 00:00:00 and manages the complex interplay
between tribute recognition, deficit calculations, and resource allocation.

## Key Concepts

### Network Limits Management

- **Emission Limit**: Total network emission calculated for each Worldwide Day
- **Total Gratis Limit**: Available resources after deducting fees (`Emission Limit - Total Fees`)
- **Total Lysis Limit**: 23/24 of Total Gratis Limit (distributed across 23 hourly Lysis runs)
- **Touch Limit**: 1/24 of Total Gratis Limit

### Day Types

- **Green Days**: Full operation with 23 hourly Lysis runs followed by Touch
- **Red Days**: Touch-only operation for all offered Tributes

### Recognition Process

- **Lysis**: Hourly tribute recognition runs (23 times per day on Green Days)
- **Touch**: Final recognition opportunity for unrecognized tributes

## Business Logic

### Preparation Phase (`Prepare`)

1. **Day Classification**: Determines if day is Green (Lysis + Touch) or Red (Touch only) via Price Oracle
2. **Limit Calculations**:
    - Total Gratis Limit = Emission Limit - Total Fees
    - Lysis Limit = Total Lysis Limit / 23
    - Touch Limit = Total Gratis Limit / 24
3. **Deficit Analysis**:
    - Total Tribute Interest = Sum of all tribute symbolic loads
    - Total Lysis Deficit = max(Total Tribute Interest - Total Lysis Limit, 32% × Total Tribute Interest)
    - Recalculate Total Lysis Limit = Total Tribute Interest - Total Lysis Deficit
4. **Resource Distribution**:
    - Calculate progressive deficit distribution across 23 Lysis runs using exponential decay
    - Retrieve vector rates for each Lysis run
    - Query gold ingot price (400 troy ounces) for Touch calculations

### Execution Phase (`Execute`)

1. **Lysis Execution** (Green Days only):
    - Run 23 hourly Lysis sessions from index 23 down to 1
    - Apply tribute queuing based on Account Fidelity
    - Handle capacity constraints and rollover logic
    - Issue Nods with calculated floor prices
2. **Touch Execution** (Both day types):
    - Process unrecognized tributes through random selection
    - Calculate number of touches based on gold ingot price
    - Issue Qualified Nods with Touch Gratis Quantity
3. **Tribute Burning**: All tributes are burned after completion

## Technical Architecture

### State Management

- **Config**: Contract configuration including oracle addresses and deficit parameters
- **MetadosisInfo**: Daily run information (Lysis + Touch or Touch-only)
- **DailyRunState**: Tracks execution progress and undistributed limits
- **RunHistory**: Historical data for UI display and analytics
- **Winners**: Tracking to prevent duplicate recognition

### External Dependencies

- **Tribute Contract**: Queries total tribute interest and manages tribute lifecycle
- **Price Oracle**: Provides exchange rates and day type classification
- **Vector Contract**: Supplies vector rates for Lysis calculations
- **Token Allocator**: Handles resource allocation and distribution
- **Nod Contract**: Issues recognition tokens (Nods)

## API Reference

### Messages

#### InstantiateMsg

```rust
pub struct InstantiateMsg {
    pub creator: Option<String>,
    pub vector: Option<Addr>,
    pub tribute: Option<Addr>,
    pub nod: Option<Addr>,
    pub token_allocator: Option<Addr>,
    pub price_oracle: Option<Addr>,
    pub random_oracle: Option<Addr>,
    pub deficit: Decimal,  // Deficit percentage (1.0 = 100%)
}
```

#### ExecuteMsg

```rust
pub enum ExecuteMsg {
    Prepare {
        run_date: Option<WorldwideDay>,
    },
    Execute {
        run_date: Option<WorldwideDay>,
    },
    BurnAll {},  // Available only in demo feature
}
```

### Operations

#### Prepare Operation

Calculates and stores all parameters needed for daily runs:

- Queries emission limits and oracle data
- Computes deficit distribution using exponential formula
- Determines day type and appropriate run configuration
- Stores MetadosisInfo for execution phase

#### Execute Operation

Performs the actual tribute recognition:

- Executes Lysis runs (on Green Days) with progressive deficit application
- Runs Touch process for remaining tributes
- Issues Nods through external contracts
- Records execution history and winner tracking

## Mathematical Models

### Deficit Distribution Formula

The deficit is distributed across 23 Lysis runs using progressive scaling:

```
d_r = D × (e^(-0.2(r-1))) / (∑(j=1 to 23) e^(-0.2(j-1)))
```

Where:

- `d_r` = Deficit for Lysis run r
- `r` = Lysis run index (1 to 23)
- `D` = Total Lysis Deficit

### Touch Value Calculation

```rust
fn calc_touch_win_amount(touch_limit: Uint128, ignot_price: Decimal) -> (usize, Uint128) {
    let touches = max(1, touch_limit / ignot_price);
    let touch_value = touch_limit / touches;
    (touches, touch_value)
}
```

## Usage Examples

### Contract Instantiation

```rust
let msg = InstantiateMsg {
   creator: Some("creator_address".to_string()),
   vector: Some(Addr::unchecked("vector_contract")),
   tribute: Some(Addr::unchecked("tribute_contract")),
   nod: Some(Addr::unchecked("nod_contract")),
   token_allocator: Some(Addr::unchecked("allocator_contract")),
   price_oracle: Some(Addr::unchecked("oracle_contract")),
   random_oracle: Some(Addr::unchecked("random_contract")),
   deficit: Decimal::percent(32), // 32% minimum deficit
};
```

### Daily Operations

```rust
// 1. Prepare daily runs
let prepare_msg = ExecuteMsg::Prepare {
    run_date: Some(WorldwideDay::new(2025, 6, 10)),
};

// 2. Execute recognition process
let execute_msg = ExecuteMsg::Execute {
    run_date: Some(WorldwideDay::new(2025, 6, 10)),
};
```

## Deployment

### Prerequisites

- Tribute contract deployed and operational
- Price Oracle providing exchange rates and day types
- Vector contract with configured vector rates
- Token Allocator for resource management
- Nod contract for issuing recognition tokens

### Configuration Steps

1. Deploy the Metadosis contract with proper instantiation parameters
2. Configure all external contract addresses
3. Set appropriate deficit percentage (typically 32%)
4. Verify oracle connectivity and data feeds
5. Test with demo features before production use

## Testing

The contract includes comprehensive tests covering:

- Nod ID generation algorithms
- Touch amount calculations with various gold price scenarios
- Deficit distribution mathematical accuracy
- State management and storage operations

Run tests using:

```bash
cargo test
```

## Integration Notes

### Daily Workflow

1. **36 hours after Worldwide Day ends**: Metadosis Day begins
2. **UTC 00:00:00**: Prepare operation calculates daily parameters
3. **Green Days**: 23 hourly Lysis runs from 00:00 to 22:00
4. **UTC 23:00:00**: Touch operation processes remaining tributes
5. **End of day**: All tributes burned to reset for next cycle

### Cron Integration

```shell
SENDER="outbe1y4xt40rc8lhsz2lulkkkx555fde53n4x2hxgq5"
CONTRACT_ADDRESS="outbe1l3nvn4nc9ftahmr6zjd4frywzpw7ag87kkl93ms37al2zptst6cq9kus08"

# Create the prepare job
PREPARE_MSG=$(jq -n \
  --arg sender "$SENDER" \
  --arg contract "$CONTRACT_ADDRESS" \
  '{
    "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
    "sender": $sender,
    "contract": $contract,
    "msg": {"prepare": {}}
  }')

outbe-chaind tx cron create-job "metadosis-prepare" $TXFLAG \
  --start-time "2025-10-01T12:30:00Z" \
  --interval-seconds 86400 \
  --from ci \
  --message "$PREPARE_MSG"

# Create the execute job
EXECUTE_MSG=$(jq -n \
  --arg sender "$SENDER" \
  --arg contract "$CONTRACT_ADDRESS" \
  '{
    "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
    "sender": $sender,
    "contract": $contract,
    "msg": {"execute": {}}
  }')

outbe-chaind tx cron create-job "metadosis-execute" $TXFLAG \
  --start-time "2025-10-01T12:40:00Z" \
  --interval-seconds 86400 \
  --from ci \
  --message "$EXECUTE_MSG"
```

### Error Handling

- `AlreadyPrepared`: Prevents duplicate preparation for same day
- `NotInitialized`: Ensures all required contracts are configured
- Comprehensive validation of mathematical calculations and state transitions

## Security Considerations

- Access control through ownership management
- Immutable mathematical formulas prevent manipulation
- Oracle dependency requires trusted price feeds
- State validation prevents inconsistent execution phases
- Winner tracking prevents double recognition
