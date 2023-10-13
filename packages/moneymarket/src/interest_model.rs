use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Decimal256, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub base_rate: Decimal256,
    pub first_interest_multiplier: Decimal256,
    pub target_utilization_rate: Decimal256,
    pub second_interest_multiplier: Decimal256,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        base_rate: Option<Decimal256>,
        first_interest_multiplier: Option<Decimal256>,
        target_utilization_rate: Option<Decimal256>,
        second_interest_multiplier: Option<Decimal256>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(BorrowRateResponse)]
    BorrowRate {
        market_balance: Uint256,
        total_liabilities: Decimal256,
        total_reserves: Decimal256,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub base_rate: Decimal256,
    pub first_interest_multiplier: Decimal256,
    pub target_utilization_rate: Decimal256,
    pub second_interest_multiplier: Decimal256,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct BorrowRateResponse {
    pub rate: Decimal256,
}

#[cw_serde]

pub struct MigrateMsg {
    pub owner: String,
    pub base_rate: Decimal256,
    pub first_interest_multiplier: Decimal256,
    pub target_utilization_rate: Decimal256,
    pub second_interest_multiplier: Decimal256,
}
