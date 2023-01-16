use crate::swap::create_swap_msg;
use cosmwasm_std::{
    attr, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response,
    StdResult, SubMsg, Uint256, WasmMsg,
};
use std::convert::TryInto;

use crate::contract::{CLAIM_REWARDS_OPERATION, SWAP_TO_STABLE_OPERATION};
use crate::error::ContractError;
use crate::external::handle::RewardContractExecuteMsg;
use crate::state::{read_config, Config};

use moneymarket::querier::{query_all_balances, query_balance};

/// Request withdraw reward operation to
/// reward contract and execute `distribute_hook`
/// Executor: overseer
pub fn distribute_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if config.overseer_contract != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    let reward_contract = deps.api.addr_humanize(&config.reward_contract)?;

    // Do not emit the event logs here
    Ok(
        Response::new().add_submessages(vec![SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract.to_string(),
                funds: vec![],
                msg: to_binary(&RewardContractExecuteMsg::ClaimRewards { recipient: None })?,
            }),
            CLAIM_REWARDS_OPERATION,
        )]),
    )
}

/// Apply swapped reward to global index
/// Executor: itself
pub fn distribute_hook(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let contract_addr = env.contract.address;
    let config: Config = read_config(deps.storage)?;
    let overseer_contract = deps.api.addr_humanize(&config.overseer_contract)?;

    // reward_amount = (prev_balance + reward_amount) - prev_balance
    // = (0 + reward_amount) - 0 = reward_amount = balance
    let reward_amount: Uint256 = query_balance(
        deps.as_ref(),
        contract_addr,
        config.stable_denom.to_string(),
    )?;
    let mut messages: Vec<CosmosMsg> = vec![];
    if !reward_amount.is_zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: overseer_contract.to_string(),
            amount: vec![Coin {
                denom: config.stable_denom,
                amount: reward_amount.try_into()?,
            }],
        }));
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "distribute_rewards"),
        attr("buffer_rewards", reward_amount),
    ]))
}

/// Swap all coins to stable_denom
/// and execute `swap_hook`
/// Executor: itself
/// TODO, adapt what we prefer doing, what swaps we want to have
pub fn swap_to_stable_denom(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    let contract_addr = env.contract.address.clone();
    let balances: Vec<Coin> = query_all_balances(deps.as_ref(), contract_addr)?;
    let mut messages: Vec<SubMsg> = balances
        .iter()
        .filter(|x| x.denom != config.stable_denom)
        .map(|coin: &Coin| {
            create_swap_msg(
                deps.as_ref(),
                env.clone(),
                coin.clone(),
                config.stable_denom.clone(),
            )
        })
        .flat_map(|result| match result {
            Ok(vec) => vec.into_iter().map(|item| Ok(SubMsg::new(item))).collect(),
            Err(er) => vec![Err(er)],
        })
        .collect::<StdResult<Vec<SubMsg>>>()?;

    if let Some(last) = messages.last_mut() {
        last.id = SWAP_TO_STABLE_OPERATION;
        last.reply_on = ReplyOn::Success;
    } else {
        return distribute_hook(deps, env);
    }

    Ok(Response::new().add_submessages(messages))
}
