use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Env, Timestamp, Uint128};
use cw20::Denom;
use outbe_nft::state::NftInfo;
use outbe_nft::traits::Cw721CollectionConfig;

/// ConsumptionUnit contract config
#[cw_serde]
pub struct CUConfig {
    pub settlement_token: Denom,
    pub native_token: Denom,
    pub price_oracle: Addr,
}

impl Cw721CollectionConfig for CUConfig {}

/// ConsumptionUnit public data
#[cw_serde]
pub struct ConsumptionUnitData {
    /// The value of Consumption Unit in Settlement Tokens
    pub consumption_value: Uint128,
    /// Sum of Nominal Qty from Consumption Records
    pub nominal_quantity: Uint128,
    /// Nominal currency from Consumption Records
    pub nominal_currency: String,
    /// Where the CU is allocated by the User.
    /// A user can change commitment Pool at any time prior to CU NFT selection in raffle
    pub commitment_tier: u16,
    /// State of the record
    pub state: ConsumptionUnitState,
    /// Calculated according to initial Native Coin Price, PGT, and allocated Commitment Pool.
    /// FloorPrice is to be re-calculated each time out of the update of the Commitment Pool
    pub floor_price: Decimal,
    /// Hashes identifying consumption records batch
    pub hashes: Vec<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cw_serde]
pub enum ConsumptionUnitState {
    /// Created on the Network
    Reflected,
    /// Participating in Raffle (Commitment pool and consequently floorPrice can be changed)
    Nominated,
    /// Was selected as a winner in Raffle
    /// (Commitment pool and consequently floorPrice cannot be changed)
    Selected,
}

pub type ConsumptionUnitNft = NftInfo<ConsumptionUnitData>;

impl outbe_nft::traits::Cw721State for ConsumptionUnitData {}
impl outbe_nft::traits::Cw721CustomMsg for ConsumptionUnitData {}

impl ConsumptionUnitData {
    pub fn update_tier(mut self, new_tier_id: u16, env: &Env) -> Self {
        self.commitment_tier = new_tier_id;
        self.updated_at = env.block.time;
        self
    }
}
