use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint256,
};
use moneymarket::common::optional_addr_validate;
use moneymarket::interest_model::{
    BorrowRateResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
            base_rate: msg.base_rate,
            first_interest_multiplier: msg.first_interest_multiplier,
            second_interest_multiplier: msg.second_interest_multiplier,
            target_utilization_rate: msg.target_utilization_rate,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            base_rate,
            first_interest_multiplier,
            target_utilization_rate,
            second_interest_multiplier,
        } => {
            let api = deps.api;
            update_config(
                deps,
                info,
                optional_addr_validate(api, owner)?,
                base_rate,
                first_interest_multiplier,
                target_utilization_rate,
                second_interest_multiplier,
            )
        }
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<Addr>,
    base_rate: Option<Decimal256>,
    first_interest_multiplier: Option<Decimal256>,
    target_utilization_rate: Option<Decimal256>,
    second_interest_multiplier: Option<Decimal256>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(owner.as_str())?;
    }

    if let Some(base_rate) = base_rate {
        config.base_rate = base_rate;
    }

    if let Some(first_interest_multiplier) = first_interest_multiplier {
        config.first_interest_multiplier = first_interest_multiplier;
    }

    if let Some(second_interest_multiplier) = second_interest_multiplier {
        config.second_interest_multiplier = second_interest_multiplier;
    }

    if let Some(target_utilization_rate) = target_utilization_rate {
        config.target_utilization_rate = target_utilization_rate;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::BorrowRate {
            market_balance,
            total_liabilities,
            total_reserves,
        } => to_binary(&query_borrow_rate(
            deps,
            market_balance,
            total_liabilities,
            total_reserves,
        )?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&state.owner)?.to_string(),
        base_rate: state.base_rate,
        first_interest_multiplier: state.first_interest_multiplier,
        target_utilization_rate: state.target_utilization_rate,
        second_interest_multiplier: state.second_interest_multiplier,
    };

    Ok(resp)
}

fn first_interest_rate(config: &Config, utilization_ratio: Decimal256) -> StdResult<Decimal256> {
    if utilization_ratio > config.target_utilization_rate {
        Err(StdError::generic_err(
            "This function only receives values under target UR",
        ))
    } else {
        Ok(utilization_ratio * config.first_interest_multiplier + config.base_rate)
    }
}

fn second_interest_rate(config: &Config, utilization_ratio: Decimal256) -> StdResult<Decimal256> {
    if utilization_ratio <= config.target_utilization_rate {
        Err(StdError::generic_err(
            "This function only receives values under over UR",
        ))
    } else {
        let target_ir = first_interest_rate(config, config.target_utilization_rate)?;
        Ok(target_ir
            + (utilization_ratio - config.target_utilization_rate)
                * config.second_interest_multiplier)
    }
}

fn query_borrow_rate(
    deps: Deps,
    market_balance: Uint256,
    total_liabilities: Decimal256,
    total_reserves: Decimal256,
) -> StdResult<BorrowRateResponse> {
    let config: Config = read_config(deps.storage)?;

    // ignore decimal parts
    let total_value_in_market =
        Decimal256::from_ratio(market_balance, 1u128) + total_liabilities - total_reserves;

    let utilization_ratio = if total_value_in_market.is_zero() {
        Decimal256::zero()
    } else {
        total_liabilities / total_value_in_market
    };

    // We want 2 slopes
    let rate = if utilization_ratio <= config.target_utilization_rate {
        first_interest_rate(&config, utilization_ratio)
    } else {
        second_interest_rate(&config, utilization_ratio)
    }?;

    Ok(BorrowRateResponse { rate })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    let new_config = Config {
        owner: deps.api.addr_canonicalize(&msg.owner)?,
        base_rate: msg.base_rate,
        first_interest_multiplier: msg.first_interest_multiplier,
        target_utilization_rate: msg.target_utilization_rate,
        second_interest_multiplier: msg.second_interest_multiplier,
    };

    store_config(deps.storage, &new_config)?;
    Ok(Response::default())
}
