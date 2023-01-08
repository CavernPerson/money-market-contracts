use crate::swap::create_swap_msg;
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    ReplyOn, Response, StdResult, SubMsg, Uint128, Uint256, WasmMsg, WasmQuery,
};
use std::convert::TryInto;

use crate::contract::{CLAIM_REWARDS_OPERATION, SWAP_TO_STABLE_OPERATION};
use crate::error::ContractError;
use crate::external::handle::{RewardContractExecuteMsg, RewardContractQueryMsg};
use crate::state::{read_config, BLunaAccruedRewardsResponse, Config};

use moneymarket::querier::{query_all_balances, query_balance};

// REWARD_THRESHOLD
// This value is used as the minimum reward claim amount
// thus if a user's reward is less than 1 ust do not send the ClaimRewards msg
const REWARDS_THRESHOLD: Uint128 = Uint128::new(1000000);

/// Request withdraw reward operation to
/// reward contract and execute `distribute_hook`
/// Executor: overseer
pub fn distribute_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if config.overseer_contract != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    let contract_addr = env.contract.address;
    let reward_contract = deps.api.addr_humanize(&config.reward_contract)?;

    let accrued_rewards =
        get_accrued_rewards(deps.as_ref(), reward_contract.clone(), contract_addr)?;
    if accrued_rewards < REWARDS_THRESHOLD {
        return Ok(Response::default());
    }

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

pub(crate) fn get_accrued_rewards(
    deps: Deps,
    reward_contract_addr: Addr,
    contract_addr: Addr,
) -> StdResult<Uint128> {
    let rewards: BLunaAccruedRewardsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: reward_contract_addr.to_string(),
            msg: to_binary(&RewardContractQueryMsg::AccruedRewards {
                address: contract_addr.to_string(),
            })?,
        }))?;

    Ok(rewards.rewards)
}
