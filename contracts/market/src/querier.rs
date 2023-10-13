use crate::borrow::get_actual_interest_factor;
use cosmwasm_std::{
    to_binary, Addr, Decimal256, Deps, QueryRequest, StdResult, Uint256, WasmQuery,
};
use cosmwasm_std::{Env, StdError};
use moneymarket::querier::query_balance;

use moneymarket::distribution_model::{
    BorrowerIncentivesRateResponse, QueryMsg as DistributionQueryMsg,
};
use moneymarket::interest_model::{BorrowRateResponse, QueryMsg as InterestQueryMsg};
use moneymarket::overseer::{BorrowLimitResponse, ConfigResponse, QueryMsg as OverseerQueryMsg};

use crate::state::{read_config, read_state};

pub fn query_borrow_rate(
    deps: Deps,
    interest_addr: Addr,
    market_balance: Uint256,
    total_liabilities: Decimal256,
    total_reserves: Decimal256,
) -> StdResult<BorrowRateResponse> {
    let borrow_rate: BorrowRateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: interest_addr.to_string(),
            msg: to_binary(&InterestQueryMsg::BorrowRate {
                market_balance,
                total_liabilities,
                total_reserves,
            })?,
        }))?;

    Ok(borrow_rate)
}

pub fn query_next_borrower_incentives(
    deps: Deps,
    env: Env,
    block_height: Option<u64>,
) -> StdResult<BorrowRateResponse> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;
    let block_height = block_height.unwrap_or(env.block.height);
    if state.last_interest_updated >= block_height {
        return Err(StdError::generic_err("Can't query borrow rate in the past"));
    }
    let passed_blocks = Decimal256::from_ratio(block_height - state.last_interest_updated, 1u128);
    let mut available_borrower_incentives = query_balance(
        deps,
        deps.api
            .addr_humanize(&config.borrow_reserves_bucket_contract)?,
        config.stable_denom.to_string(),
    )?;

    let balance = query_balance(
        deps,
        deps.api.addr_humanize(&config.contract_addr)?,
        config.stable_denom.to_string(),
    )?;

    let borrow_rate_res: BorrowRateResponse = query_borrow_rate(
        deps,
        deps.api.addr_humanize(&config.interest_model)?,
        balance,
        state.total_liabilities,
        state.total_reserves,
    )?;

    let overseer_config =
        query_overseer_config(deps, deps.api.addr_humanize(&config.overseer_contract)?)?;

    // We limit the maximum borrow incentives to a percentage of the total balance available to spread the reserves over more blocks
    // This makes the borrow distribution APY more stable over time and is best for users
    available_borrower_incentives =
        available_borrower_incentives * overseer_config.buffer_distribution_factor;

    get_actual_interest_factor(
        deps.api,
        &config,
        &mut state,
        available_borrower_incentives,
        borrow_rate_res.rate * passed_blocks,
        passed_blocks,
    )?;

    Ok(BorrowRateResponse {
        rate: Decimal256::from_ratio(state.prev_borrower_incentives, 1u128)
            / state.total_liabilities
            / passed_blocks,
    })
}

pub fn query_borrow_limit(
    deps: Deps,
    overseer_addr: Addr,
    borrower: Addr,
    block_time: Option<u64>,
) -> StdResult<BorrowLimitResponse> {
    let borrow_limit: BorrowLimitResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: overseer_addr.to_string(),
            msg: to_binary(&OverseerQueryMsg::BorrowLimit {
                borrower: borrower.to_string(),
                block_time,
            })?,
        }))?;

    Ok(borrow_limit)
}

pub fn query_borrow_reserves_incentives_rate(
    deps: Deps,
    distribution_model: Addr,
    deposit_rate: Decimal256,
    target_deposit_rate: Decimal256,
    threshold_deposit_rate: Decimal256,
    current_incentives_rate: Decimal256,
) -> StdResult<BorrowerIncentivesRateResponse> {
    let incentives_rate: BorrowerIncentivesRateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: distribution_model.to_string(),
            msg: to_binary(&DistributionQueryMsg::BorrowerIncentivesRate {
                deposit_rate,
                target_deposit_rate,
                threshold_deposit_rate,
                current_incentives_rate,
            })?,
        }))?;

    Ok(incentives_rate)
}

pub fn query_target_deposit_rate(deps: Deps, overseer_contract: Addr) -> StdResult<Decimal256> {
    let overseer_config = query_overseer_config(deps, overseer_contract)?;

    Ok(overseer_config.target_deposit_rate)
}

pub fn query_overseer_config(deps: Deps, overseer_contract: Addr) -> StdResult<ConfigResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer_contract.to_string(),
        msg: to_binary(&OverseerQueryMsg::Config {})?,
    }))
}
