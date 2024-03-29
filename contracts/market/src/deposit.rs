use cosmwasm_std::{
    attr, to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal256, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, Uint256, WasmMsg,
};
use std::convert::TryInto;

use crate::borrow::compute_interest;
use crate::error::ContractError;
use crate::state::{read_config, read_state, store_state, Config, State};

use cw20::Cw20ExecuteMsg;
use moneymarket::querier::{query_balance, query_supply};

pub fn deposit_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // Check base denom deposit
    let deposit_amount: Uint256 = info
        .funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    // Cannot deposit zero amount
    if deposit_amount.is_zero() {
        return Err(ContractError::ZeroDeposit(config.stable_denom));
    }

    // Update interest related state
    let mut state: State = read_state(deps.storage)?;
    let borrow_incentives_messages = compute_interest(
        deps.as_ref(),
        &config,
        &mut state,
        env.block.height,
        Some(deposit_amount),
    )?;

    // Load anchor token exchange rate with updated state
    let exchange_rate =
        compute_exchange_rate(deps.as_ref(), &config, &state, Some(deposit_amount))?;
    let mint_amount =
        Decimal256::from_ratio(deposit_amount, 1u128) / exchange_rate * Uint256::one();

    state.prev_aterra_supply += mint_amount;
    store_state(deps.storage, &state)?;
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
            funds: vec![],
            msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                amount: mint_amount.try_into()?,
            })?,
        }))
        .add_messages(borrow_incentives_messages)
        .add_attributes(vec![
            attr("action", "deposit_stable"),
            attr("depositor", info.sender),
            attr("mint_amount", mint_amount),
            attr("deposit_amount", deposit_amount),
        ]))
}

pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    burn_amount: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // Update interest related state
    let mut state: State = read_state(deps.storage)?;
    let borrow_incentives_messages =
        compute_interest(deps.as_ref(), &config, &mut state, env.block.height, None)?;

    // Load anchor token exchange rate with updated state
    let exchange_rate = compute_exchange_rate(deps.as_ref(), &config, &state, None)?;
    let redeem_amount = Uint256::from(burn_amount) * exchange_rate;

    let current_balance = query_balance(
        deps.as_ref(),
        env.contract.address,
        config.stable_denom.to_string(),
    )?;

    // Assert redeem amount
    assert_redeem_amount(&config, &state, current_balance, redeem_amount)?;

    state.prev_aterra_supply -= Uint256::from(burn_amount);
    store_state(deps.storage, &state)?;
    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
                funds: vec![],
                msg: to_json_binary(&Cw20ExecuteMsg::Burn {
                    amount: burn_amount,
                })?,
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: sender.to_string(),
                amount: vec![Coin {
                    denom: config.stable_denom,
                    amount: redeem_amount.try_into()?,
                }],
            }),
        ])
        .add_messages(borrow_incentives_messages)
        .add_attributes(vec![
            attr("action", "redeem_stable"),
            attr("burn_amount", burn_amount),
            attr("redeem_amount", redeem_amount),
        ]))
}

fn assert_redeem_amount(
    config: &Config,
    state: &State,
    current_balance: Uint256,
    redeem_amount: Uint256,
) -> Result<(), ContractError> {
    let current_balance = Decimal256::from_ratio(current_balance, 1u128);
    let redeem_amount = Decimal256::from_ratio(redeem_amount, 1u128);
    if redeem_amount + state.total_reserves > current_balance {
        return Err(ContractError::NoStableAvailable(
            config.stable_denom.clone(),
        ));
    }

    Ok(())
}

pub(crate) fn compute_exchange_rate(
    deps: Deps,
    config: &Config,
    state: &State,
    deposit_amount: Option<Uint256>,
) -> StdResult<Decimal256> {
    let aterra_supply = query_supply(deps, deps.api.addr_humanize(&config.aterra_contract)?)?;
    let balance = query_balance(
        deps,
        deps.api.addr_humanize(&config.contract_addr)?,
        config.stable_denom.to_string(),
    )? - deposit_amount.unwrap_or_else(Uint256::zero);

    Ok(compute_exchange_rate_raw(state, aterra_supply, balance))
}

pub fn compute_exchange_rate_raw(
    state: &State,
    aterra_supply: Uint256,
    contract_balance: Uint256,
) -> Decimal256 {
    if aterra_supply.is_zero() {
        return Decimal256::one();
    }

    // (aterra / stable_denom)
    // exchange_rate = (balance + total_liabilities - total_reserves) / aterra_supply
    (Decimal256::from_ratio(contract_balance, 1u128) + state.total_liabilities
        - state.total_reserves)
        / aterra_supply
}
