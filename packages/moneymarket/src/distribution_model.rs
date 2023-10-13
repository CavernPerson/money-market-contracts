use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Decimal256;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub distribution_cap: Decimal256,
    pub distribution_floor: Decimal256,
    pub increment_multiplier: Decimal256,
    pub decrement_multiplier: Decimal256,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        distribution_cap: Option<Decimal256>,
        distribution_floor: Option<Decimal256>,
        increment_multiplier: Option<Decimal256>,
        decrement_multiplier: Option<Decimal256>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(BorrowerIncentivesRateResponse)]
    BorrowerIncentivesRate {
        deposit_rate: Decimal256,
        target_deposit_rate: Decimal256,
        threshold_deposit_rate: Decimal256,
        current_incentives_rate: Decimal256,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub distribution_cap: Decimal256,
    pub distribution_floor: Decimal256,
    pub increment_multiplier: Decimal256,
    pub decrement_multiplier: Decimal256,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct BorrowerIncentivesRateResponse {
    pub incentives_rate: Decimal256,
}
