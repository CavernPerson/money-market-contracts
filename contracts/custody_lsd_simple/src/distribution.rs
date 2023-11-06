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
    return Ok(Response::default());
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
