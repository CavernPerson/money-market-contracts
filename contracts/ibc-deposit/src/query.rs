use cosmwasm_std::{Deps, Env, StdResult, Uint128};
use cw20::BalanceResponse;
use moneymarket::market::ConfigResponse;

use crate::state::CONFIG;

pub fn a_terra_addr(deps: Deps) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    let market_config: ConfigResponse = deps.querier.query_wasm_smart(
        config.market_addr,
        &moneymarket::market::QueryMsg::Config {},
    )?;

    Ok(market_config.aterra_contract)
}
pub fn a_terra_balance(deps: Deps, env: &Env) -> StdResult<Uint128> {
    let a_terra_address = a_terra_addr(deps)?;

    let balance: BalanceResponse = deps.querier.query_wasm_smart(
        a_terra_address,
        &cw20_base::msg::QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    Ok(balance.balance)
}
