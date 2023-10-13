use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;

use crate::interest_model::BorrowRateResponse;

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner address for config update
    pub owner_addr: String,
    /// stable coin denom used to borrow & repay
    pub stable_denom: String,
    /// Anchor token code ID used to instantiate
    pub aterra_code_id: u64,
    /// Anchor token distribution speed
    // pub anc_emission_rate: Decimal256,
    /// Maximum allowed borrow rate over deposited stable balance
    pub max_borrow_factor: Decimal256,

    /// Maximum rate under which borrowers are not incentivised with reserves
    pub max_borrow_subsidy_rate: Decimal256,

    /// Initial rate of rewards used for borrowers
    pub initial_borrower_incentives: Decimal256,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),

    ////////////////////
    /// Owner operations
    ////////////////////
    /// Register Contracts contract address
    RegisterContracts {
        overseer_contract: String,
        /// The contract has the logics for
        /// Anchor borrow interest rate
        interest_model: String,
        /// The contract has the logics for
        /// borrower incentives distribution speed
        distribution_model: String,
        /// Collector contract to send all the reserve
        collector_contract: String,
        /// Bucket to save borrow reserves
        borrow_reserves_bucket_contract: String,
        /// Faucet contract to drip ANC token to users
        distributor_contract: String,
    },

    /// Update config values
    UpdateConfig {
        owner_addr: Option<String>,
        max_borrow_factor: Option<Decimal256>,
        interest_model: Option<String>,
        max_borrow_subsidy_rate: Option<Decimal256>,
        distribution_model: Option<String>,
    },

    ////////////////////
    /// Overseer operations
    ////////////////////
    /// Repay stable with liquidated collaterals
    RepayStableFromLiquidation {
        borrower: String,
        prev_balance: Uint256,
    },

    /// Execute epoch operations
    /// 1. send reserve to collector contract
    /// 2. update anc_emission_rate state
    ExecuteEpochOperations {
        deposit_rate: Decimal256,
        target_deposit_rate: Decimal256,
        threshold_deposit_rate: Decimal256,
        distributed_interest: Uint256,
    },

    ////////////////////
    /// User operations
    ////////////////////
    /// Deposit stable asset to get interest
    DepositStable {},

    /// Borrow stable asset with collaterals in overseer contract
    BorrowStable {
        borrow_amount: Uint256,
        to: Option<String>,
    },

    /// Repay stable asset to decrease liability
    RepayStable {},
}

#[cw_serde]

pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemStable {},
}

#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State { block_height: Option<u64> },
    #[returns(BorrowRateResponse)]
    BorrowerIncentives { block_height: Option<u64> },
    #[returns(EpochStateResponse)]
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
    #[returns(BorrowerInfoResponse)]
    BorrowerInfo {
        borrower: String,
        block_height: Option<u64>,
    },
    #[returns(BorrowerInfosResponse)]
    BorrowerInfos {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub aterra_contract: String,
    pub interest_model: String,
    pub distribution_model: String,
    pub overseer_contract: String,
    pub collector_contract: String,
    pub distributor_contract: String,
    pub borrow_reserves_bucket_contract: String,
    pub stable_denom: String,
    pub max_borrow_factor: Decimal256,
    pub max_borrow_subsidy_rate: Decimal256,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct StateResponse {
    pub total_liabilities: Decimal256,
    pub total_reserves: Decimal256,
    pub last_interest_updated: u64,
    pub last_reward_updated: u64,
    pub global_interest_index: Decimal256,
    pub global_reward_index: Decimal256,
    pub reserves_rate_used_for_borrowers: Decimal256,
    pub prev_aterra_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
    pub prev_borrower_incentives: Uint256,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
    pub reserves_rate_used_for_borrowers: Decimal256,
    pub prev_borrower_incentives: Uint256,
    pub last_interest_updated: u64,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct BorrowerInfoResponse {
    pub borrower: String,
    pub interest_index: Decimal256,
    pub reward_index: Decimal256,
    pub loan_amount: Uint256,
    pub pending_rewards: Decimal256,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct BorrowerInfosResponse {
    pub borrower_infos: Vec<BorrowerInfoResponse>,
}

#[cw_serde]

pub struct MigrateMsg {}
