use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal256, Uint256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub base_rate: Decimal256,
    pub first_interest_multiplier: Decimal256,
    pub target_utilization_rate: Decimal256,
    pub second_interest_multiplier: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        base_rate: Option<Decimal256>,
        first_interest_multiplier: Option<Decimal256>,
        target_utilization_rate: Option<Decimal256>,
        second_interest_multiplier: Option<Decimal256>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BorrowRate {
        market_balance: Uint256,
        total_liabilities: Decimal256,
        total_reserves: Decimal256,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub base_rate: Decimal256,
    pub first_interest_multiplier: Decimal256,
    pub target_utilization_rate: Decimal256,
    pub second_interest_multiplier: Decimal256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowRateResponse {
    pub rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {
    pub base_rate: Decimal256,
    pub first_interest_multiplier: Decimal256,
    pub target_utilization_rate: Decimal256,
    pub second_interest_multiplier: Decimal256,
}