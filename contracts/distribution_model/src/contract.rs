use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use moneymarket::common::optional_addr_validate;
use moneymarket::distribution_model::{
    BorrowerIncentivesRateResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
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
            distribution_cap: msg.distribution_cap,
            distribution_floor: msg.distribution_floor,
            increment_multiplier: msg.increment_multiplier,
            decrement_multiplier: msg.decrement_multiplier,
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
            distribution_cap,
            distribution_floor,
            increment_multiplier,
            decrement_multiplier,
        } => {
            let api = deps.api;
            update_config(
                deps,
                info,
                optional_addr_validate(api, owner)?,
                distribution_cap,
                distribution_floor,
                increment_multiplier,
                decrement_multiplier,
            )
        }
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<Addr>,
    distribution_cap: Option<Decimal256>,
    distribution_floor: Option<Decimal256>,
    increment_multiplier: Option<Decimal256>,
    decrement_multiplier: Option<Decimal256>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(owner.as_str())?;
    }

    if let Some(distribution_cap) = distribution_cap {
        config.distribution_cap = distribution_cap;
    }

    if let Some(distribution_floor) = distribution_floor {
        config.distribution_floor = distribution_floor
    }

    if let Some(increment_multiplier) = increment_multiplier {
        config.increment_multiplier = increment_multiplier;
    }

    if let Some(decrement_multiplier) = decrement_multiplier {
        config.decrement_multiplier = decrement_multiplier;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::BorrowerIncentivesRate {
            deposit_rate,
            target_deposit_rate,
            threshold_deposit_rate,
            current_incentives_rate,
        } => to_binary(&query_borrower_incentives_rate(
            deps,
            deposit_rate,
            target_deposit_rate,
            threshold_deposit_rate,
            current_incentives_rate,
        )?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&state.owner)?.to_string(),
        distribution_cap: state.distribution_cap,
        distribution_floor: state.distribution_floor,
        increment_multiplier: state.increment_multiplier,
        decrement_multiplier: state.decrement_multiplier,
    };

    Ok(resp)
}

fn query_borrower_incentives_rate(
    deps: Deps,
    deposit_rate: Decimal256,
    target_deposit_rate: Decimal256,
    threshold_deposit_rate: Decimal256,
    current_incentives_rate: Decimal256,
) -> StdResult<BorrowerIncentivesRateResponse> {
    let config: Config = read_config(deps.storage)?;

    let half_dec = Decimal256::one() + Decimal256::one();
    let mid_rate = (threshold_deposit_rate + target_deposit_rate) / half_dec;
    let high_trigger = (mid_rate + target_deposit_rate) / half_dec;
    let low_trigger = (mid_rate + threshold_deposit_rate) / half_dec;

    let incentives_rate = if deposit_rate < low_trigger {
        current_incentives_rate * config.increment_multiplier
    } else if deposit_rate > high_trigger {
        current_incentives_rate * config.decrement_multiplier
    } else {
        current_incentives_rate
    };

    let incentives_rate = if incentives_rate > config.distribution_cap {
        config.distribution_cap
    } else if incentives_rate < config.distribution_floor {
        config.distribution_floor
    } else {
        incentives_rate
    };

    Ok(BorrowerIncentivesRateResponse { incentives_rate })
}
