#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_json, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};
use moneymarket::astroport_router::AssetInfo;

use crate::collateral::{
    deposit_collateral, liquidate_collateral, lock_collateral, query_borrower, query_borrowers,
    unlock_collateral, withdraw_collateral,
};
use crate::distribution::distribute_rewards;
use crate::error::ContractError;
use crate::state::{read_config, store_config, store_swap_config, Config, SwapConfig};

use cw20::Cw20ReceiveMsg;
use moneymarket::common::optional_addr_validate;
use moneymarket::custody::{
    Cw20HookMsg, ExecuteMsg, LSDConfigResponse, LSDInstantiateMsg, MigrateMsg, QueryMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: LSDInstantiateMsg,
) -> StdResult<Response> {
    // In the case of a stable CW20 token, we validate their address
    if let AssetInfo::Token { contract_addr } = msg.stable_token.clone() {
        deps.api.addr_validate(contract_addr.as_ref())?;
    }

    let config = Config {
        owner: deps.api.addr_canonicalize(&msg.owner)?,
        overseer_contract: deps.api.addr_canonicalize(&msg.overseer_contract)?,
        collateral_token: deps.api.addr_canonicalize(&msg.collateral_token)?,
        market_contract: deps.api.addr_canonicalize(&msg.market_contract)?,
        liquidation_contract: deps.api.addr_canonicalize(&msg.liquidation_contract)?,
        stable_token: msg.stable_token,
        basset_info: msg.basset_info,

        known_cw20_tokens: msg
            .known_tokens
            .iter()
            .map(|addr| deps.api.addr_validate(addr))
            .collect::<StdResult<Vec<Addr>>>()?,
    };

    let swap_config = SwapConfig {
        astroport_addr: deps.api.addr_validate(&msg.astroport_addr)?,
        phoenix_addr: deps.api.addr_validate(&msg.phoenix_addr)?,
        terraswap_addr: deps.api.addr_validate(&msg.terraswap_addr)?,
    };

    store_swap_config(deps.storage, &swap_config)?;
    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::UpdateConfig {
            owner,
            liquidation_contract,
            known_tokens,
        } => {
            let api = deps.api;
            update_config(
                deps,
                info,
                optional_addr_validate(api, owner)?,
                optional_addr_validate(api, liquidation_contract)?,
                known_tokens,
            )
        }
        ExecuteMsg::LockCollateral { borrower, amount } => {
            let borrower_addr = deps.api.addr_validate(&borrower)?;
            lock_collateral(deps, info, borrower_addr, amount)
        }
        ExecuteMsg::UnlockCollateral { borrower, amount } => {
            let borrower_addr = deps.api.addr_validate(&borrower)?;
            unlock_collateral(deps, info, borrower_addr, amount)
        }
        ExecuteMsg::DistributeRewards {} => distribute_rewards(deps, env, info),
        ExecuteMsg::WithdrawCollateral { amount } => withdraw_collateral(deps, info, amount),
        ExecuteMsg::LiquidateCollateral {
            liquidator,
            borrower,
            amount,
        } => {
            let liquidator_addr = deps.api.addr_validate(&liquidator)?;
            let borrower_addr = deps.api.addr_validate(&borrower)?;
            liquidate_collateral(deps, info, liquidator_addr, borrower_addr, amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender;

    match from_json(&cw20_msg.msg) {
        Ok(Cw20HookMsg::DepositCollateral { borrower }) => {
            // only asset contract can execute this message
            let config: Config = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())? != config.collateral_token {
                return Err(ContractError::Unauthorized {});
            }

            let borrower = borrower.unwrap_or(cw20_msg.sender);
            let borrower_addr = deps.api.addr_validate(&borrower)?;
            deposit_collateral(deps, borrower_addr, cw20_msg.amount.into())
        }
        _ => Err(ContractError::MissingDepositCollateralHook {}),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<Addr>,
    liquidation_contract: Option<Addr>,
    known_tokens: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(owner.as_str())?;
    }

    if let Some(liquidation_contract) = liquidation_contract {
        config.liquidation_contract = deps.api.addr_canonicalize(liquidation_contract.as_str())?;
    }

    if let Some(known_tokens) = known_tokens {
        config.known_cw20_tokens = known_tokens
            .iter()
            .map(|addr| deps.api.addr_validate(addr))
            .collect::<StdResult<Vec<Addr>>>()?;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Borrower { address } => {
            let addr = deps.api.addr_validate(&address)?;
            to_json_binary(&query_borrower(deps, addr)?)
        }
        QueryMsg::Borrowers { start_after, limit } => to_json_binary(&query_borrowers(
            deps,
            optional_addr_validate(deps.api, start_after)?,
            limit,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<LSDConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(LSDConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        collateral_token: deps
            .api
            .addr_humanize(&config.collateral_token)?
            .to_string(),
        overseer_contract: deps
            .api
            .addr_humanize(&config.overseer_contract)?
            .to_string(),
        market_contract: deps.api.addr_humanize(&config.market_contract)?.to_string(),
        reward_contract: "".to_string(),
        liquidation_contract: deps
            .api
            .addr_humanize(&config.liquidation_contract)?
            .to_string(),
        stable_token: config.stable_token,
        basset_info: config.basset_info,

        known_tokens: config
            .known_cw20_tokens
            .iter()
            .map(|addr| addr.to_string())
            .collect(),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
