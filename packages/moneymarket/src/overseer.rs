use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::tokens::TokensHuman;
use cosmwasm_std::{Decimal256, Uint256};

#[cw_serde]
pub struct PlatformFeeInstantiateMsg {
    pub rate: Decimal256,
    pub receiver: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Initial owner address
    pub owner_addr: String,
    /// Oracle contract address for collateral tokens
    pub oracle_contract: String,
    /// Market contract address to receive missing interest buffer
    pub market_contract: String,
    /// Liquidation model contract address to compute liquidation amount
    pub liquidation_contract: String,
    /// Bucket contract address which saves the borrower incentives
    pub borrow_reserves_bucket_contract: String,
    /// The base denomination used when fetching oracle price,
    /// reward distribution, and borrow
    pub stable_denom: String,
    /// # of blocks per epoch period
    pub epoch_period: u64,
    /// Distribute interest buffer to market contract,
    /// when deposit_rate < threshold_deposit_rate
    pub threshold_deposit_rate: Decimal256,
    /// Target deposit rate.
    /// When current deposit rate is bigger than this,
    /// Custody contracts send rewards to interest buffer
    pub target_deposit_rate: Decimal256,
    /// Ratio to be distributed from the interest buffer
    pub buffer_distribution_factor: Decimal256,
    /// Ratio to be used for purchasing ANC token from the interest buffer
    // pub anc_purchase_factor: Decimal256,
    /// Valid oracle price timeframe
    pub price_timeframe: u64,
    /// # of blocks per each dynamic rate change period
    pub dyn_rate_epoch: u64,
    /// maximum rate change during update
    pub dyn_rate_maxchange: Decimal256,
    /// amount of slack in yr change to trigger rate update
    pub dyn_rate_yr_increase_expectation: Decimal256,
    /// clamps for dyn rate
    pub dyn_rate_min: Decimal256,
    pub dyn_rate_max: Decimal256,
    pub platform_fee: PlatformFeeInstantiateMsg,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    ////////////////////
    /// Owner operations
    ////////////////////

    /// Update Configs
    UpdateConfig {
        owner_addr: Option<String>,
        oracle_contract: Option<String>,
        liquidation_contract: Option<String>,
        threshold_deposit_rate: Option<Decimal256>,
        target_deposit_rate: Option<Decimal256>,
        buffer_distribution_factor: Option<Decimal256>,
        //anc_purchase_factor: Option<Decimal256>,
        epoch_period: Option<u64>,
        price_timeframe: Option<u64>,
        dyn_rate_epoch: Option<u64>,
        dyn_rate_maxchange: Option<Decimal256>,
        dyn_rate_yr_increase_expectation: Option<Decimal256>,
        dyn_rate_min: Option<Decimal256>,
        dyn_rate_max: Option<Decimal256>,
        platform_fee: Option<PlatformFeeMsg>,
    },
    /// Create new custody contract for the given collateral token
    Whitelist {
        name: String,             // bAsset name
        symbol: String,           // bAsset symbol
        collateral_token: String, // bAsset token contract
        custody_contract: String, // bAsset custody contract
        max_ltv: Decimal256,      // Loan To Value ratio
    },
    /// Update registered whitelist info
    UpdateWhitelist {
        collateral_token: String,         // bAsset token contract
        custody_contract: Option<String>, // bAsset custody contract
        max_ltv: Option<Decimal256>,      // Loan To Value ratio
    },

    /// Claims all staking rewards from the bAsset contracts
    /// and also do a epoch basis updates
    /// 1. Distribute interest buffers to depositors
    /// 2. Invoke [Custody] DistributeRewards
    /// 3. Update epoch state
    ExecuteEpochOperations {},
    UpdateEpochState {
        interest_buffer: Uint256,
        distributed_interest: Uint256,
    },

    ////////////////////
    /// User operations
    ////////////////////
    LockCollateral {
        collaterals: TokensHuman, // <(Collateral Token, Amount)>
    },
    UnlockCollateral {
        collaterals: TokensHuman, // <(Collateral Token, Amount)>
    },

    /////////////////////////////
    /// Permissionless operations
    /////////////////////////////
    LiquidateCollateral {
        borrower: String,
    },

    FundReserve {},
}

#[cw_serde]
pub struct PlatformFeeMsg {
    pub rate: Option<Decimal256>,
    pub receiver: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(EpochState)]
    EpochState {},
    #[returns(DynrateState)]
    DynrateState {},
    #[returns(WhitelistResponse)]
    Whitelist {
        collateral_token: Option<String>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(CollateralsResponse)]
    Collaterals { borrower: String },
    #[returns(AllCollateralsResponse)]
    AllCollaterals {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(BorrowLimitResponse)]
    BorrowLimit {
        borrower: String,
        block_time: Option<u64>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub oracle_contract: String,
    pub market_contract: String,
    pub liquidation_contract: String,
    pub borrow_reserves_bucket_contract: String,
    pub threshold_deposit_rate: Decimal256,
    pub target_deposit_rate: Decimal256,
    pub buffer_distribution_factor: Decimal256,
    //pub anc_purchase_factor: Decimal256,
    pub stable_denom: String,
    pub epoch_period: u64,
    pub price_timeframe: u64,
    pub dyn_rate_epoch: u64,
    pub dyn_rate_maxchange: Decimal256,
    pub dyn_rate_yr_increase_expectation: Decimal256,
    pub dyn_rate_min: Decimal256,
    pub dyn_rate_max: Decimal256,
}

#[cw_serde]
pub struct EpochState {
    pub deposit_rate: Decimal256,
    pub prev_aterra_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
    pub prev_interest_buffer: Uint256,
    pub last_executed_height: u64,
}

#[cw_serde]
pub struct DynrateState {
    pub last_executed_height: u64,
    pub prev_yield_reserve: Decimal256,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct WhitelistResponseElem {
    pub name: String,
    pub symbol: String,
    pub max_ltv: Decimal256,
    pub custody_contract: String,
    pub collateral_token: String,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct WhitelistResponse {
    pub elems: Vec<WhitelistResponseElem>,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct CollateralsResponse {
    pub borrower: String,
    pub collaterals: TokensHuman, // <(Collateral Token, Amount)>
}

// We define a custom struct for each query response
#[cw_serde]
pub struct AllCollateralsResponse {
    pub all_collaterals: Vec<CollateralsResponse>,
}

#[cw_serde]
pub struct BorrowLimitResponse {
    pub borrower: String,
    pub borrow_limit: Uint256,
}
