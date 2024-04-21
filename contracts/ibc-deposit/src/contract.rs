use cosmwasm_std::{
    ensure_eq, entry_point, to_json_binary, wasm_execute, Binary, Coin, CosmosMsg, DepsMut, Env,
    IbcMsg, IbcTimeout, MessageInfo, Reply, Response, StdError, StdResult, SubMsg,
};
use terra_proto_rs::cosmos::base;
use terra_proto_rs::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom};
use terra_proto_rs::traits::Message;
use terra_proto_rs::{osmosis::tokenfactory::v1beta1::MsgMint, traits::TypeUrl};

use crate::msg::InstantiateMsg;
use crate::query::{a_terra_addr, a_terra_balance};
use crate::state::{Config, CurrentTransfer, TEMP_CURRENT_TRANSFER};
use crate::{msg::ExecuteMsg, state::CONFIG};

pub const AFTER_DEPOSIT_REPLY: u64 = 456;

pub const SUBDENOM: &str = "ibc.receipt";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            market_addr: deps.api.addr_validate(&msg.market_addr)?,
            transfer_timeout: msg.transfer_timeout,
            denom: format!("factory/{}/{} ", env.contract.address, SUBDENOM),
            admin: info.sender,
        },
    )?;

    Ok(Response::new().add_message(CosmosMsg::Stargate {
        type_url: MsgCreateDenom::TYPE_URL.to_string(),
        value: Binary::from(
            MsgCreateDenom {
                sender: env.contract.address.to_string(),
                subdenom: SUBDENOM.to_string(),
            }
            .encode_to_vec(),
        ),
    }))
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Deposit {
            target_channel,
            target_addr,
        } => deposit(deps, env, info, target_channel, target_addr),
        ExecuteMsg::Withdraw { following_actions } => withdraw(deps, env, info, following_actions),
    }
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target_channel: String,
    target_addr: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let cavern_execute = wasm_execute(
        config.market_addr,
        &moneymarket::market::ExecuteMsg::DepositStable {},
        info.funds,
    )?;

    let sub_msg = SubMsg::reply_on_success(cavern_execute, AFTER_DEPOSIT_REPLY);

    let balance_before = a_terra_balance(deps.as_ref(), &env)?;

    TEMP_CURRENT_TRANSFER.save(
        deps.storage,
        &CurrentTransfer {
            addr: target_addr,
            channel_id: target_channel,
            balance_before,
        },
    )?;

    Ok(Response::new().add_submessage(sub_msg))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: CosmosMsg,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let deposit = cw_utils::one_coin(&info).map_err(crate::std_error)?;

    ensure_eq!(
        deposit.denom,
        config.denom,
        StdError::generic_err("Wrong denom to withdraw funds !")
    );

    let withdraw_msg = wasm_execute(
        config.market_addr,
        &cw20_base::msg::ExecuteMsg::Send {
            contract: a_terra_addr(deps.as_ref())?,
            amount: deposit.amount,
            msg: to_json_binary(&moneymarket::market::Cw20HookMsg::RedeemStable {})?,
        },
        vec![],
    )?;

    let burn_msg = MsgBurn {
        sender: env.contract.address.to_string(),
        amount: Some(base::v1beta1::Coin {
            amount: deposit.amount.to_string(),
            denom: config.denom,
        }),
        burn_from_address: env.contract.address.to_string(),
    };

    Ok(Response::new()
        .add_message(withdraw_msg)
        .add_message(CosmosMsg::Stargate {
            type_url: MsgBurn::TYPE_URL.to_string(),
            value: burn_msg.encode_to_vec().into(),
        })
        .add_message(actions))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        AFTER_DEPOSIT_REPLY => {
            let config = CONFIG.load(deps.storage)?;
            let current_transfer = TEMP_CURRENT_TRANSFER.load(deps.storage)?;
            // Query the aToken balance of the contract and redirect everything to the depositor

            let balance = a_terra_balance(deps.as_ref(), &env)?;

            let new_balance = balance - current_transfer.balance_before;

            // We mint new tokens for this user
            // We send them via IBC on the transfer channel
            Ok(Response::new()
                .add_message(CosmosMsg::Stargate {
                    type_url: MsgMint::TYPE_URL.to_string(),
                    value: Binary::from(
                        MsgMint {
                            sender: env.contract.address.to_string(),
                            amount: Some(base::v1beta1::Coin {
                                denom: config.denom.clone(),
                                amount: new_balance.to_string(),
                            }),
                            mint_to_address: env.contract.address.to_string(),
                        }
                        .encode_to_vec(),
                    ),
                })
                .add_message(CosmosMsg::Ibc(IbcMsg::Transfer {
                    channel_id: current_transfer.channel_id,
                    to_address: current_transfer.addr,
                    amount: Coin {
                        denom: config.denom,
                        amount: new_balance,
                    },
                    timeout: IbcTimeout::with_timestamp(
                        env.block.time.plus_seconds(config.transfer_timeout),
                    ),
                })))
        }
        _ => Err(StdError::generic_err(
            "Wrong reply on the ibc-deposit contract",
        )),
    }
}
