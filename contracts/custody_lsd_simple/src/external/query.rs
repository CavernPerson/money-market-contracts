use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum LSDQueryMsg {
    State {},
}

#[cw_serde]
pub struct LSDStateResponse {
    /// Total supply to the Steak token
    pub total_usteak: Uint128,
    /// Total amount of uluna staked
    pub total_uluna: Uint128,
    /// The exchange rate between usteak and uluna, in terms of uluna per usteak
    pub exchange_rate: Decimal,
    /// Staking rewards currently held by the contract that are ready to be reinvested
    pub unlocked_coins: Vec<Coin>,
}
