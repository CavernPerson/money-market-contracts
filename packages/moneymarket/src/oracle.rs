use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal256;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub base_asset: String,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
    },
    RegisterFeeder {
        asset: String,
        feeder: String,
    },
    UpdateFeeder {
        asset: String,
        feeder: String,
    },
    FeedPrice {
        prices: Vec<(String, Decimal256)>, // (asset, price)
    },
}

#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(FeederResponse)]
    Feeder { asset: String },
    #[returns(PriceResponse)]
    Price { base: String, quote: String },
    #[returns(PricesResponse)]
    Prices {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub base_asset: String,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct FeederResponse {
    pub asset: String,
    pub feeder: String,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct PriceResponse {
    pub rate: Decimal256,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct PricesResponseElem {
    pub asset: String,
    pub price: Decimal256,
    pub last_updated_time: u64,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct PricesResponse {
    pub prices: Vec<PricesResponseElem>,
}
