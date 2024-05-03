use cosmrs::Any;
use cw_orch::daemon::networks::{OSMOSIS_1, PHOENIX_1};
use cw_orch::interchain::ChannelCreationValidator;
use cw_orch::interchain::DaemonInterchainEnv;
use cw_orch::prelude::InterchainEnv;
use cw_orch::prelude::TxHandler;
use cw_orch::{prelude::*, tokio::runtime::Runtime};
use ibc_deposit::interface::IbcDeposit;
use scripts::AXELAR_1;
use serde_json::json;
use terra_proto_rs::cosmos::base;
use terra_proto_rs::ibc::applications::transfer::v1::MsgTransfer;
use terra_proto_rs::traits::Message;
use terra_proto_rs::traits::TypeUrl;

pub const PACKET_TIMEOUT: u64 = 60 * 60 * 24 * 7; // 1 week

pub const TERRA_USDC: &str = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
pub const TERRA_CHANNEL: &str = "channel-1";
pub const OSMOSIS_CHANNEL: &str = "channel-251";

pub const OSMOSIS_DENOM: &str =
    "ibc/FC430DC0EB7FE8D1C5FFAB61660E92C6E8544F8620CE88C36D261737620FFA6E";
pub const TERRA_AXELAR_CHANNEL: &str = "channel-6";

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();

    // We upload code_ids
    let rt = Runtime::new()?;
    let terra = DaemonBuilder::default()
        .chain(PHOENIX_1)
        .handle(rt.handle())
        .build()?;

    let osmosis = DaemonBuilder::default()
        .chain(OSMOSIS_1)
        .handle(rt.handle())
        .build()?;

    let axelar = DaemonBuilder::default()
        .chain(AXELAR_1)
        .handle(rt.handle())
        .build()?;

    // let balance = osmosis.bank_querier().balance(osmosis.sender(), None)?;
    // panic!("{:?}", balance);

    let contract = IbcDeposit::new("ibc-deposit", terra.clone());
    let current_block = osmosis.block_info()?;

    let withdraw_msg = ibc_deposit::msg::ExecuteMsg::Withdraw {
        to_axelar_channel: TERRA_AXELAR_CHANNEL.to_string(),
        axelar_receiver: "".to_string(),
    };

    let memo = json!({
        "wasm": {
            "contract": contract.address()?,
            "msg":withdraw_msg
        },
    })
    .to_string();

    let response = osmosis.commit_any::<bool>(
        vec![Any {
            type_url: MsgTransfer::TYPE_URL.to_string(),
            value: MsgTransfer {
                source_port: "transfer".to_string(),
                source_channel: OSMOSIS_CHANNEL.to_string(),
                token: Some(base::v1beta1::Coin {
                    amount: "100000".to_string(),
                    denom: OSMOSIS_DENOM.to_string(),
                }),
                sender: osmosis.sender().to_string(),
                receiver: contract.address()?.to_string(),
                timeout_height: None,
                timeout_timestamp: current_block.time.plus_seconds(3_600).nanos(),
                memo,
            }
            .encode_to_vec(),
        }],
        None,
    )?;

    let interchain = DaemonInterchainEnv::from_daemons(
        &terra.rt_handle,
        vec![terra.clone(), osmosis, axelar],
        &ChannelCreationValidator,
    );

    interchain.wait_ibc("osmosis-1", response)?;

    Ok(())
}
