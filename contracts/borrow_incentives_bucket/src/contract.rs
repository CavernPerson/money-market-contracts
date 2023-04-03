use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_std::{Coin, Empty};
use cosmwasm_std::{Uint128, WasmMsg};
use moneymarket::bucket::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use moneymarket::overseer::ExecuteMsg as OverseerExecuteMsg;

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
            owner: deps.api.addr_validate(&msg.owner)?,
            sender_contract: deps.api.addr_validate(&msg.owner)?,
            overseer_contract: deps.api.addr_validate(&msg.owner)?,
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
            sender_contract,
            overseer_contract,
        } => update_config(deps, info, owner, sender_contract, overseer_contract),
        ExecuteMsg::Send { denom, amount } => execute_send(deps, info, denom, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn execute_send(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;

    if info.sender != config.sender_contract {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_attribute("action", "empty_bucket")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.overseer_contract.to_string(),
            funds: vec![Coin { denom, amount }],
            msg: to_binary(&OverseerExecuteMsg::FundReserve {})?,
        })))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    sender_contract: Option<String>,
    overseer_contract: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let mut res = Response::new().add_attribute("action", "update_config");

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
        res = res.add_attribute("owner", owner);
    }

    if let Some(sender_contract) = sender_contract {
        config.sender_contract = deps.api.addr_validate(&sender_contract)?;
        res = res.add_attribute("sender_contract", sender_contract);
    }
    if let Some(overseer_contract) = overseer_contract {
        config.overseer_contract = deps.api.addr_validate(&overseer_contract)?;
        res = res.add_attribute("overseer_contract", overseer_contract);
    }

    store_config(deps.storage, &config)?;
    Ok(res)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: state.owner.to_string(),
        sender_contract: state.sender_contract.to_string(),
        overseer_contract: state.overseer_contract.to_string(),
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
}
