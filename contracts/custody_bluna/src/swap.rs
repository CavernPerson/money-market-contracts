use crate::state::SWAP_CONFIG;
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Deps, Env, QueryRequest, StdResult, Uint128, WasmMsg, WasmQuery,
};
use moneymarket::astroport_router::{
    AssetInfo, ExecuteMsg as AstroportExecuteMsg, QueryMsg as AstroportQueryMsg,
    SimulateSwapOperationsResponse, SwapMessageType, SwapOperation,
};
use serde::Serialize;
use strum::IntoEnumIterator;

pub fn into_cosmos_msg<M: Serialize, T: Into<String>>(
    message: M,
    contract_addr: T,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    let msg = to_binary(&message)?;
    let execute = WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg,
        funds,
    };
    Ok(execute.into())
}

pub fn create_swap_msg(
    deps: Deps,
    env: Env,
    c: Coin,
    stable_denom: String,
) -> StdResult<Vec<CosmosMsg>> {
    let (best_price_index, best_price): (usize, Uint128) = SwapMessageType::iter()
        .map(|message_type| {
            get_swap_result_for(deps, c.clone(), stable_denom.clone(), message_type)
        })
        .enumerate()
        .filter(|(_, best_price)| best_price.is_ok())
        .map(|(best_price_index, best_price)| (best_price_index, best_price.unwrap()))
        .max_by_key(|&(_, item)| item)
        .unwrap();

    if best_price == Uint128::zero() {
        return Ok(vec![]);
    }

    let best_price_marketplace: SwapMessageType =
        SwapMessageType::iter().nth(best_price_index).unwrap();

    Ok(vec![create_swap_message_for(
        deps,
        env,
        c,
        stable_denom,
        best_price_marketplace,
    )?])
}

// Astroport router smart-contract
// https://docs.astroport.fi/astroport/smart-contracts/astroport-contract-addresses
// Testnet : terra1na348k6rvwxje9jj6ftpsapfeyaejxjeq6tuzdmzysps20l6z23smnlv64
// Mainnet : terra1j8hayvehh3yy02c2vtw5fdhz9f4drhtee8p5n5rguvg3nyd6m83qd2y90a

// Phoenix Finance
// https://docs.phoenixfi.so/developers/contract-addresses
// Testnet : terra1j7kv9r7rg34fufszsr09sx73jkwruc79e43axs2kraxj940z6ulssp36hs
// Mainnet : terra1r634fv9kj8s6vjgnrwdha35cwhz6jcpz0h0trhc4yehllvtzzxuq5slv0g

// Terraswap
// https://docs.terraswap.io/docs/contract_resources/contract_addresses/
// Testnet : terra1xp6xe6uwqrspumrkazdg90876ns4h78yw03vfxghhcy03yexcrcsdaqvc8
// Mainnet : terra13ehuhysn5mqjeaheeuew2gjs785f6k7jm8vfsqg3jhtpkwppcmzqcu7chk

fn get_astroport_swap_operation(
    offer_denom: String,
    ask_denom: String,
    message_type: SwapMessageType,
) -> SwapOperation {
    match message_type {
        SwapMessageType::Astroport => SwapOperation::AstroSwap {
            offer_asset_info: AssetInfo::NativeToken { denom: offer_denom },
            ask_asset_info: AssetInfo::NativeToken { denom: ask_denom },
        },
        SwapMessageType::Phoenix => SwapOperation::TokenSwap {
            offer_asset_info: AssetInfo::NativeToken { denom: offer_denom },
            ask_asset_info: AssetInfo::NativeToken { denom: ask_denom },
        },
        SwapMessageType::TerraSwap => SwapOperation::TerraSwap {
            offer_asset_info: AssetInfo::NativeToken { denom: offer_denom },
            ask_asset_info: AssetInfo::NativeToken { denom: ask_denom },
        },
    }
}

fn get_contract_address(deps: Deps, message_type: SwapMessageType) -> StdResult<String> {
    let swap_config = SWAP_CONFIG.load(deps.storage)?;
    Ok(match message_type {
        SwapMessageType::Astroport => swap_config.astroport_addr,
        SwapMessageType::Phoenix => swap_config.phoenix_addr,
        SwapMessageType::TerraSwap => swap_config.terraswap_addr,
    }
    .to_string())
}

pub fn create_swap_message_for(
    deps: Deps,
    _env: Env,
    c: Coin,
    stable_denom: String,
    message_type: SwapMessageType,
) -> StdResult<CosmosMsg> {
    let contract_address = get_contract_address(deps, message_type)?;

    into_cosmos_msg(
        AstroportExecuteMsg::ExecuteSwapOperations {
            operations: vec![get_astroport_swap_operation(
                c.denom.clone(),
                stable_denom,
                message_type,
            )],
            to: None,
            max_spread: None,
            minimum_receive: None,
        },
        contract_address,
        vec![c],
    )
}

pub fn get_swap_result_for(
    deps: Deps,
    c: Coin,
    stable_denom: String,
    message_type: SwapMessageType,
) -> StdResult<Uint128> {
    let contract_address = get_contract_address(deps, message_type)?;

    let swap_operation_response: SimulateSwapOperationsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_address,
            msg: to_binary(&AstroportQueryMsg::SimulateSwapOperations {
                offer_amount: c.amount,
                operations: vec![get_astroport_swap_operation(
                    c.denom,
                    stable_denom,
                    message_type,
                )],
            })?,
        }))?;
    Ok(swap_operation_response.amount)
}
