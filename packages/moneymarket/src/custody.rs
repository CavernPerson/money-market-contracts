use cosmwasm_std::StdResult;
use cosmwasm_std::{to_binary, WasmMsg};

use crate::astroport_router::AssetInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cosmwasm_std::{Addr, BankMsg};
use cosmwasm_std::{Coin, CosmosMsg};

use cosmwasm_std::Uint256;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

#[cw_serde]
pub struct InstantiateMsg {
    /// owner address
    pub owner: String,
    /// bAsset token address
    pub collateral_token: String,
    /// overseer contract address
    pub overseer_contract: String,
    /// market contract address
    pub market_contract: String,
    /// bAsset rewrad contract
    pub reward_contract: String,
    /// liquidation contract address
    pub liquidation_contract: String,
    /// Expected reward denom. If bAsset reward is not same with
    /// it, we try to convert the reward to the `stable_denom`.
    pub stable_denom: String,
    pub basset_info: BAssetInfo,

    // New mecanism specific values
    pub astroport_addr: String,
    pub phoenix_addr: String,
    pub terraswap_addr: String,
}

#[cw_serde]
pub struct LSDInstantiateMsg {
    /// owner address
    pub owner: String,
    /// bAsset token address
    pub collateral_token: String,
    /// overseer contract address
    pub overseer_contract: String,
    /// market contract address
    pub market_contract: String,
    /// bAsset rewrad contract
    pub reward_contract: String,
    /// liquidation contract address
    pub liquidation_contract: String,
    /// Expected reward denom. If bAsset reward is not same with
    /// it, we try to convert the reward to the `stable_token`.
    pub stable_token: AssetInfo,
    pub basset_info: BAssetInfo,

    // New mecanism specific values
    pub astroport_addr: String,
    pub phoenix_addr: String,
    pub terraswap_addr: String,

    // Known tokens to swap from to the stable_token
    pub known_tokens: Vec<String>,
}

#[cw_serde]
pub struct Asset {
    pub amount: Uint128,
    pub asset_info: AssetInfo,
}

impl AssetInfo {
    pub fn is_same_asset(&self, c: &Coin) -> bool {
        match self {
            AssetInfo::Token { contract_addr: _ } => false,
            AssetInfo::NativeToken { denom } => *denom == c.denom,
        }
    }
}

impl Asset {
    pub fn to_msg(&self, to: Addr) -> StdResult<CosmosMsg> {
        let send_message = match self.asset_info.clone() {
            AssetInfo::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
                to_address: to.to_string(),
                amount: vec![Coin {
                    denom,
                    amount: self.amount,
                }],
            }),
            AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: to.to_string(),
                    amount: self.amount,
                })?,
            }),
        };
        Ok(send_message)
    }
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// CW20 token receiver
    Receive(Cw20ReceiveMsg),

    ////////////////////
    /// Overseer operations
    ////////////////////

    /// Update config
    UpdateConfig {
        owner: Option<String>,
        liquidation_contract: Option<String>,
        known_tokens: Option<Vec<String>>,
    },
    /// Make specified amount of tokens unspendable
    LockCollateral { borrower: String, amount: Uint256 },
    /// Make specified amount of collateral tokens spendable
    UnlockCollateral { borrower: String, amount: Uint256 },
    /// Claim bAsset rewards and distribute claimed rewards
    /// to market and overseer contracts
    DistributeRewards {},

    /// Liquidate collateral and send liquidated collateral to `to` address
    LiquidateCollateral {
        liquidator: String,
        borrower: String,
        amount: Uint256,
    },

    ////////////////////
    /// User operations
    ////////////////////

    /// Withdraw spendable collateral token.
    /// If the amount is not given,
    /// return all spendable collateral
    WithdrawCollateral { amount: Option<Uint256> },
}

#[cw_serde]
pub enum Cw20HookMsg {
    /// Deposit collateral token
    DepositCollateral {},
}

#[cw_serde]

pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(BorrowerResponse)]
    Borrower { address: String },
    #[returns(BorrowersResponse)]
    Borrowers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub collateral_token: String,
    pub overseer_contract: String,
    pub market_contract: String,
    pub reward_contract: String,
    pub liquidation_contract: String,
    pub stable_denom: String,
    pub basset_info: BAssetInfo,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct LSDConfigResponse {
    pub owner: String,
    pub collateral_token: String,
    pub overseer_contract: String,
    pub market_contract: String,
    pub reward_contract: String,
    pub liquidation_contract: String,
    pub stable_token: AssetInfo,
    pub basset_info: BAssetInfo,

    pub known_tokens: Vec<String>,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct BorrowerResponse {
    pub borrower: String,
    pub balance: Uint256,
    pub spendable: Uint256,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct BorrowersResponse {
    pub borrowers: Vec<BorrowerResponse>,
}

#[cw_serde]
pub struct BAssetInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}
