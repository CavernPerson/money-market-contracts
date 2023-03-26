use cosmwasm_std::{
    attr, to_binary, Addr, Api, BankMsg, Coin, CosmosMsg, Decimal256, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint256, WasmMsg,
};
use moneymarket::interest_model::BorrowRateResponse;
use moneymarket::market::{BorrowerInfoResponse, BorrowerInfosResponse};
use moneymarket::overseer::BorrowLimitResponse;
use moneymarket::querier::{query_balance, query_supply};
use std::convert::TryInto;

use crate::deposit::compute_exchange_rate_raw;
use crate::error::ContractError;
use crate::querier::{
    query_borrow_limit, query_borrow_rate, query_overseer_config, query_target_deposit_rate,
};
use crate::state::{
    read_borrower_info, read_borrower_infos, read_config, read_state, store_borrower_info,
    store_state, BorrowerInfo, Config, State,
};
use moneymarket::bucket::ExecuteMsg as BucketExecuteMsg;

pub fn borrow_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    borrow_amount: Uint256,
    to: Option<Addr>,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    let mut state: State = read_state(deps.storage)?;

    let borrower = info.sender;
    let borrower_raw = deps.api.addr_canonicalize(borrower.as_str())?;
    let mut liability: BorrowerInfo = read_borrower_info(deps.storage, &borrower_raw);

    // Compute interest
    let borrow_incentives_messages =
        compute_interest(deps.as_ref(), &config, &mut state, env.block.height, None)?;
    compute_borrower_interest(&state, &mut liability);

    compute_borrower_reward(&state, &mut liability);

    let overseer = deps.api.addr_humanize(&config.overseer_contract)?;
    let borrow_limit_res: BorrowLimitResponse = query_borrow_limit(
        deps.as_ref(),
        overseer,
        borrower.clone(),
        Some(env.block.time.seconds()),
    )?;

    if borrow_limit_res.borrow_limit < borrow_amount + liability.loan_amount {
        return Err(ContractError::BorrowExceedsLimit(
            borrow_limit_res.borrow_limit.try_into()?,
        ));
    }

    let current_balance = query_balance(
        deps.as_ref(),
        env.contract.address,
        config.stable_denom.to_string(),
    )?;

    // Assert borrow amount
    assert_max_borrow_factor(&config, &state, current_balance, borrow_amount)?;

    liability.loan_amount += borrow_amount;
    state.total_liabilities += Decimal256::from_ratio(borrow_amount, 1u128);
    store_state(deps.storage, &state)?;
    store_borrower_info(deps.storage, &borrower_raw, &liability)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: to.unwrap_or_else(|| borrower.clone()).to_string(),
            amount: vec![Coin {
                denom: config.stable_denom,
                amount: borrow_amount.try_into()?,
            }],
        }))
        .add_messages(borrow_incentives_messages)
        .add_attributes(vec![
            attr("action", "borrow_stable"),
            attr("borrower", borrower),
            attr("borrow_amount", borrow_amount),
        ]))
}

pub fn repay_stable_from_liquidation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    borrower: Addr,
    prev_balance: Uint256,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if config.overseer_contract != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    let cur_balance: Uint256 = query_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.stable_denom.to_string(),
    )?;

    // override env
    let mut info = info;

    info.sender = borrower;
    info.funds = vec![Coin {
        denom: config.stable_denom,
        amount: (cur_balance - prev_balance).try_into()?,
    }];

    repay_stable(deps, env, info)
}

pub fn repay_stable(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // Check stable denom deposit
    let amount: Uint256 = info
        .funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    // Cannot deposit zero amount
    if amount.is_zero() {
        return Err(ContractError::ZeroRepay(config.stable_denom));
    }

    let mut state: State = read_state(deps.storage)?;

    let borrower = info.sender;
    let borrower_raw = deps.api.addr_canonicalize(borrower.as_str())?;
    let mut liability: BorrowerInfo = read_borrower_info(deps.storage, &borrower_raw);

    // Compute interest
    let borrow_incentives_messages = compute_interest(
        deps.as_ref(),
        &config,
        &mut state,
        env.block.height,
        Some(amount),
    )?;
    compute_borrower_interest(&state, &mut liability);

    compute_borrower_reward(&state, &mut liability);

    let repay_amount: Uint256;
    let mut messages: Vec<CosmosMsg> = vec![];
    if liability.loan_amount < amount {
        repay_amount = liability.loan_amount;
        liability.loan_amount = Uint256::zero();

        // Payback left repay amount to sender
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: borrower.to_string(),
            amount: vec![Coin {
                denom: config.stable_denom,
                amount: (amount - repay_amount).try_into()?,
            }],
        }));
    } else {
        repay_amount = amount;
        liability.loan_amount -= repay_amount;
    }

    state.total_liabilities -= Decimal256::from_ratio(repay_amount, 1u128);

    store_borrower_info(deps.storage, &borrower_raw, &liability)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_messages(borrow_incentives_messages)
        .add_attributes(vec![
            attr("action", "repay_stable"),
            attr("borrower", borrower),
            attr("repay_amount", repay_amount),
        ]))
}

/// Compute interest and update state
/// total liabilities and total reserves
pub fn compute_interest(
    deps: Deps,
    config: &Config,
    state: &mut State,
    block_height: u64,
    deposit_amount: Option<Uint256>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    if state.last_interest_updated >= block_height {
        return Ok(vec![]);
    }

    let aterra_supply = query_supply(deps, deps.api.addr_humanize(&config.aterra_contract)?)?;
    let balance: Uint256 = query_balance(
        deps,
        deps.api.addr_humanize(&config.contract_addr)?,
        config.stable_denom.to_string(),
    )? - deposit_amount.unwrap_or_else(Uint256::zero);

    let borrow_rate_res: BorrowRateResponse = query_borrow_rate(
        deps,
        deps.api.addr_humanize(&config.interest_model)?,
        balance,
        state.total_liabilities,
        state.total_reserves,
    )?;

    let target_deposit_rate: Decimal256 =
        query_target_deposit_rate(deps, deps.api.addr_humanize(&config.overseer_contract)?)?;

    compute_interest_raw(
        deps,
        config,
        state,
        block_height,
        balance,
        aterra_supply,
        borrow_rate_res.rate,
        target_deposit_rate,
    )
}

// We want to take into account the borrower incentives that the oversser gives out based on its revenues.
// new_available_borrower_incentives corresponds to the part of revenues that the overseer wants to dedicate to borrower incentives
// Those incentives are reserved for the market contract only during this operations so that's a bit risky, but should be okay
//TODO
pub fn get_actual_interest_factor(
    api: &dyn Api,
    config: &Config,
    state: &mut State,
    available_borrower_incentives: Uint256,
    interest_factor_borrow: Decimal256,
    passed_blocks: Decimal256,
) -> StdResult<(Decimal256, Vec<CosmosMsg>)> {
    let max_epoch_borrow_subsidy = config.max_borrow_subsidy_rate * passed_blocks;

    let (actual_incentives, interest_factor_borrow) = if state.total_liabilities
        == Decimal256::zero()
        || interest_factor_borrow < max_epoch_borrow_subsidy
    {
        // If there is no liability or the interest factor is already low enough, we don't give out incentives
        (Uint256::zero(), interest_factor_borrow)
    } else if interest_factor_borrow
        < Decimal256::from_ratio(available_borrower_incentives, 1u128) / state.total_liabilities
            + max_epoch_borrow_subsidy
    {
        // Here, we can give some incentives
        // But giving all the available funds would make the borrow factor go too low
        (
            (interest_factor_borrow - max_epoch_borrow_subsidy)
                * state.total_liabilities
                * Uint256::one(),
            max_epoch_borrow_subsidy,
        )
    } else {
        // Here we give all the available incentives
        (
            available_borrower_incentives,
            interest_factor_borrow
                - Decimal256::from_ratio(available_borrower_incentives, 1u128)
                    / state.total_liabilities,
        )
    };
    state.prev_borrower_incentives = actual_incentives;

    let incentives_messages = if !actual_incentives.is_zero() {
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: api
                .addr_humanize(&config.borrow_reserves_bucket_contract)?
                .to_string(),
            funds: vec![],
            msg: to_binary(&BucketExecuteMsg::Send {
                denom: config.stable_denom.clone(),
                amount: actual_incentives.try_into()?,
            })?,
        })]
    } else {
        vec![]
    };

    Ok((interest_factor_borrow, incentives_messages))
}

// CONTRACT: to use this function as state update purpose,
// executor must update following three state after execution
// * state.prev_aterra_supply
// * state.prev_exchange_rate
// * state.last_interest_updated
#[allow(clippy::too_many_arguments)]
pub fn compute_interest_raw(
    deps: Deps,
    config: &Config,
    state: &mut State,
    block_height: u64,
    balance: Uint256,
    aterra_supply: Uint256,
    borrow_rate: Decimal256,
    target_deposit_rate: Decimal256,
) -> Result<Vec<CosmosMsg>, ContractError> {
    if state.last_interest_updated >= block_height {
        return Ok(vec![]);
    }

    let passed_blocks = Decimal256::from_ratio(block_height - state.last_interest_updated, 1u128);

    let interest_factor_borrow = passed_blocks * borrow_rate;

    let mut available_borrower_incentives = query_balance(
        deps,
        deps.api
            .addr_humanize(&config.borrow_reserves_bucket_contract)?,
        config.stable_denom.to_string(),
    )?;

    let overseer_config =
        query_overseer_config(deps, deps.api.addr_humanize(&config.overseer_contract)?)?;

    // We limit the maximum borrow incentives to a percentage of the total balance available to spread the reserves over more blocks
    // This makes the borrow distribution APY more stable over time and is best for users
    available_borrower_incentives =
        available_borrower_incentives * overseer_config.buffer_distribution_factor;

    let (interest_factor, interest_factor_messages) = get_actual_interest_factor(
        deps.api,
        config,
        state,
        available_borrower_incentives,
        interest_factor_borrow,
        passed_blocks,
    )?;

    let interest_accrued = state.total_liabilities * interest_factor;

    // We also subtract the borrower_subsidies to the liabilites.
    // The actual borrow_rate is slightly lower that what is predicted by the borrow_rate affine function

    state.global_interest_index *= Decimal256::one() + interest_factor;
    state.total_liabilities += interest_accrued;

    // We update the reward index as well here
    let borrow_amount = state.total_liabilities;
    if !state.prev_borrower_incentives.is_zero() && !borrow_amount.is_zero() {
        state.global_reward_index +=
            Decimal256::from_ratio(state.prev_borrower_incentives, 1u128) / borrow_amount;
    }

    let mut exchange_rate = compute_exchange_rate_raw(state, aterra_supply, balance);
    let effective_deposit_rate = exchange_rate / state.prev_exchange_rate;
    let deposit_rate = (effective_deposit_rate - Decimal256::one()) / passed_blocks;

    if deposit_rate > target_deposit_rate {
        // excess_deposit_rate(_per_block)
        let excess_deposit_rate = deposit_rate - target_deposit_rate;
        let prev_deposits =
            Decimal256::from_ratio(state.prev_aterra_supply * state.prev_exchange_rate, 1u128);

        // excess_yield = prev_deposits * excess_deposit_rate(_per_block) * blocks
        let excess_yield = prev_deposits * passed_blocks * excess_deposit_rate;

        state.total_reserves += excess_yield;
        exchange_rate = compute_exchange_rate_raw(state, aterra_supply, balance);
    }

    state.prev_aterra_supply = aterra_supply;
    state.prev_exchange_rate = exchange_rate;
    state.last_interest_updated = block_height;
    state.last_reward_updated = block_height;
    Ok(interest_factor_messages)
}

/// Compute new interest and apply to liability
pub(crate) fn compute_borrower_interest(state: &State, liability: &mut BorrowerInfo) {
    liability.loan_amount =
        Decimal256::from_ratio(liability.loan_amount * state.global_interest_index, 1u128)
            / liability.interest_index
            * Uint256::one();
    liability.interest_index = state.global_interest_index;
}

/// Compute reward amount a borrower received
pub(crate) fn compute_borrower_reward(state: &State, liability: &mut BorrowerInfo) {
    liability.pending_rewards += Decimal256::from_ratio(liability.loan_amount, 1u128)
        / state.global_interest_index
        * (state.global_reward_index - liability.reward_index);
    liability.reward_index = state.global_reward_index;
}

pub fn query_borrower_info(
    deps: Deps,
    env: Env,
    borrower: Addr,
    block_height: Option<u64>,
) -> Result<BorrowerInfoResponse, ContractError> {
    let mut borrower_info: BorrowerInfo = read_borrower_info(
        deps.storage,
        &deps.api.addr_canonicalize(borrower.as_str())?,
    );

    let block_height = if let Some(block_height) = block_height {
        block_height
    } else {
        env.block.height
    };

    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;

    compute_interest(deps, &config, &mut state, block_height, None)?;
    compute_borrower_interest(&state, &mut borrower_info);

    compute_borrower_reward(&state, &mut borrower_info);

    Ok(BorrowerInfoResponse {
        borrower: borrower.to_string(),
        interest_index: borrower_info.interest_index,
        reward_index: borrower_info.reward_index,
        loan_amount: borrower_info.loan_amount,
        pending_rewards: borrower_info.pending_rewards,
    })
}

pub fn query_borrower_infos(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u32>,
) -> StdResult<BorrowerInfosResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.addr_canonicalize(start_after.as_str())?)
    } else {
        None
    };

    let borrower_infos: Vec<BorrowerInfoResponse> = read_borrower_infos(deps, start_after, limit)?;
    Ok(BorrowerInfosResponse { borrower_infos })
}

fn assert_max_borrow_factor(
    config: &Config,
    state: &State,
    current_balance: Uint256,
    borrow_amount: Uint256,
) -> Result<(), ContractError> {
    let current_balance = Decimal256::from_ratio(current_balance, 1u128);
    let borrow_amount = Decimal256::from_ratio(borrow_amount, 1u128);

    // Assert max borrow factor
    if state.total_liabilities + borrow_amount
        > (current_balance + state.total_liabilities - state.total_reserves)
            * config.max_borrow_factor
    {
        return Err(ContractError::MaxBorrowFactorReached(
            config.stable_denom.clone(),
        ));
    }

    // Assert available balance
    if borrow_amount + state.total_reserves > current_balance {
        return Err(ContractError::NoStableAvailable(
            config.stable_denom.clone(),
        ));
    }

    Ok(())
}
