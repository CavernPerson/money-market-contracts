use crate::external::handle::RewardContractQueryMsg;
use crate::state::BLunaAccruedRewardsResponse;
use crate::swap::create_swap_msg;
use cosmwasm_std::Addr;
use cosmwasm_std::Deps;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::Uint128;
use cosmwasm_std::WasmQuery;
use cosmwasm_std::{
    attr, to_binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, StdResult,
    SubMsg, Uint256, WasmMsg,
};
use moneymarket::astroport_router::AssetInfo;
use moneymarket::custody::Asset;
use moneymarket::querier::query_all_cw20_balances;
use moneymarket::querier::query_all_token_types_balance;
use std::convert::TryInto;

use crate::contract::{CLAIM_REWARDS_OPERATION, SWAP_TO_STABLE_OPERATION};
use crate::error::ContractError;
use crate::external::handle::RewardContractExecuteMsg;
use crate::state::{read_config, Config};

use moneymarket::querier::query_all_balances;

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
    let reward_amount: Uint256 =
        query_all_token_types_balance(deps.as_ref(), contract_addr, config.stable_token.clone())?;
    let mut messages: Vec<CosmosMsg> = vec![];
    if !reward_amount.is_zero() {
        messages.push(
            Asset {
                asset_info: config.stable_token,
                amount: reward_amount.try_into()?,
            }
            .to_msg(overseer_contract)?,
        );
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "distribute_rewards"),
        attr("buffer_rewards", reward_amount),
    ]))
}

/// Swap all coins to stable_denom
/// and execute `swap_hook`
/// We also swap known tokens (from the known token list in config) to the stable denom
/// Executor: itself
/// TODO, adapt what we prefer doing, what swaps we want to have
pub fn swap_to_stable_denom(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    let contract_addr = env.contract.address.clone();

    // We start by swapping all coin balances
    let balances: Vec<Coin> = query_all_balances(deps.as_ref(), contract_addr.clone())?;
    let mut messages: Vec<SubMsg> = balances
        .iter()
        .filter(|x| !config.stable_token.clone().is_same_asset(x))
        .map(|coin: &Coin| {
            create_swap_msg(
                deps.as_ref(),
                env.clone(),
                Asset {
                    asset_info: AssetInfo::NativeToken {
                        denom: coin.denom.clone(),
                    },
                    amount: coin.amount,
                },
                config.stable_token.clone(),
            )
        })
        .flat_map(|result| match result {
            Ok(vec) => vec.into_iter().map(|item| Ok(SubMsg::new(item))).collect(),
            Err(er) => vec![Err(er)],
        })
        .collect::<StdResult<Vec<SubMsg>>>()?;

    // Then we want to swap all cw20 token balances we know to the stable denom
    // First, we query all balances
    let cw20_balances: Vec<Asset> =
        query_all_cw20_balances(deps.as_ref(), contract_addr, &config.known_cw20_tokens)?;
    let mut cw20_messages: Vec<SubMsg> = cw20_balances
        .iter()
        .filter(|asset| !asset.amount.is_zero())
        .filter(|x| config.stable_token != x.asset_info)
        .map(|asset: &Asset| {
            create_swap_msg(
                deps.as_ref(),
                env.clone(),
                Asset {
                    asset_info: asset.asset_info.clone(),
                    amount: asset.amount,
                },
                config.stable_token.clone(),
            )
        })
        .flat_map(|result| match result {
            Ok(vec) => vec.into_iter().map(|item| Ok(SubMsg::new(item))).collect(),
            Err(er) => vec![Err(er)],
        })
        .collect::<StdResult<Vec<SubMsg>>>()?;

    messages.append(&mut cw20_messages);

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
