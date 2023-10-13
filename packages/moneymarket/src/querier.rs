use crate::custody::Asset;
use std::convert::TryInto;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, AllBalanceResponse, BalanceResponse, BankQuery, Coin, Deps, QueryRequest,
    StdError, StdResult, Uint128, Uint256, WasmQuery,
};
use cw20::{Cw20QueryMsg, TokenInfoResponse};

use crate::{
    astroport_router::AssetInfo,
    oracle::{PriceResponse, QueryMsg as OracleQueryMsg},
};

pub fn query_all_balances(deps: Deps, account_addr: Addr) -> StdResult<Vec<Coin>> {
    // load price form the oracle
    let all_balances: AllBalanceResponse =
        deps.querier
            .query(&QueryRequest::Bank(BankQuery::AllBalances {
                address: account_addr.to_string(),
            }))?;
    Ok(all_balances.amount)
}

pub fn query_all_cw20_balances(
    deps: Deps,
    contract_addr: Addr,
    tokens: &[Addr],
) -> StdResult<Vec<Asset>> {
    tokens
        .iter()
        .map(|token| {
            let result = query_token_balance(deps, token.clone(), contract_addr.clone());
            let asset_info = AssetInfo::Token {
                contract_addr: token.clone(),
            };
            result
                .map(|amount| Asset {
                    amount: amount.try_into().unwrap(),
                    asset_info: asset_info.clone(),
                })
                .or_else(|_| {
                    Ok(Asset {
                        amount: Uint128::zero(),
                        asset_info: asset_info.clone(),
                    })
                })
        })
        .collect()
}

pub fn query_all_token_types_balance(
    deps: Deps,
    account_addr: Addr,
    asset_info: AssetInfo,
) -> StdResult<Uint256> {
    match asset_info {
        AssetInfo::NativeToken { denom } => query_balance(deps, account_addr, denom),
        AssetInfo::Token { contract_addr } => {
            query_token_balance(deps, contract_addr, account_addr)
        }
    }
}

pub fn query_balance(deps: Deps, account_addr: Addr, denom: String) -> StdResult<Uint256> {
    // load price form the oracle
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: account_addr.to_string(),
        denom,
    }))?;
    Ok(balance.amount.amount.into())
}

pub fn query_token_balance(
    deps: Deps,
    contract_addr: Addr,
    account_addr: Addr,
) -> StdResult<Uint256> {
    // load balance form the token contract
    let balance: Uint128 = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: account_addr.to_string(),
            })?,
        }))
        .unwrap_or_else(|_| Uint128::zero());

    Ok(balance.into())
}

pub fn query_supply(deps: Deps, contract_addr: Addr) -> StdResult<Uint256> {
    // load price form the oracle
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    Ok(Uint256::from(token_info.total_supply))
}

#[cw_serde]
pub struct TimeConstraints {
    pub block_time: u64,
    pub valid_timeframe: u64,
}

pub fn query_price(
    deps: Deps,
    oracle_addr: Addr,
    base: String,
    quote: String,
    time_contraints: Option<TimeConstraints>,
) -> StdResult<PriceResponse> {
    let oracle_price: PriceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: oracle_addr.to_string(),
            msg: to_binary(&OracleQueryMsg::Price { base, quote })?,
        }))?;

    if let Some(time_contraints) = time_contraints {
        let valid_update_time = time_contraints.block_time - time_contraints.valid_timeframe;
        if oracle_price.last_updated_base < valid_update_time
            || oracle_price.last_updated_quote < valid_update_time
        {
            return Err(StdError::generic_err("Price is too old"));
        }
    }

    Ok(oracle_price)
}
