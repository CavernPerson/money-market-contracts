use cosmrs::Any;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, coins, CosmosMsg};
use cw_orch::daemon::networks::{OSMOSIS_1, PHOENIX_1};
use cw_orch::interchain::ChannelCreationValidator;
use cw_orch::interchain::DaemonInterchainEnv;
use cw_orch::prelude::InterchainEnv;
use cw_orch::prelude::TxHandler;
use cw_orch::{prelude::*, tokio::runtime::Runtime};
use ibc_deposit::interface::IbcDeposit;
use ibc_deposit::msg::ExecuteMsgFns;
use scripts::AXELAR_1;
use serde_json::to_string;
use terra_proto_rs::cosmos::base;
use terra_proto_rs::ibc::applications::transfer::v1::MsgTransfer;
use terra_proto_rs::traits::Message;
use terra_proto_rs::traits::TypeUrl;

pub const PACKET_TIMEOUT: u64 = 60 * 60 * 24 * 7; // 1 week

pub const TERRA_USDC: &str = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
pub const TERRA_CHANNEL: &str = "channel-1";
pub const OSMOSIS_CHANNEL: &str = "channel-251";
pub const TERRA_AXELAR_CHANNEL: &str = "channel-6";

pub const AXELAR_GMP: &str = "axelar1dv4u5k73pzqrxlzujxg3qp8kvc3pje7jtdvu72npnt5zhq05ejcsn5qme5";
pub const AXELAR_FEE_RECIPIENT: &str = "axelar1aythygn6z5thymj6tmzfwekzh05ewg3l7d6y89";

pub const OSMOSIS_DENOM: &str =
    "ibc/FC430DC0EB7FE8D1C5FFAB61660E92C6E8544F8620CE88C36D261737620FFA6E";

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

    let contract = IbcDeposit::new("ibc-deposit", terra.clone());
    let current_block = osmosis.block_info()?;

    let response = contract.withdraw(
        "axelar1xsx3ha077ykj9qkaq56c2dua3rrgszczpyeycs6gy8g5268tf6sqgkq7az".to_string(),
        "channel-6".to_string(),
        &coins(
            300_000,
            "factory/terra1cdlxuptclg4rudp92ek5ejwm6xgfum0zu5gu0ewedvf7ddejl9csmf3s32/ibc.receipt",
        ),
    )?;

    // // Let's just send an AXELAR IBC Send message
    // let gmp_msg = GmpMessage {
    //     destination_chain: "osmosis".to_string(),
    //     destination_address: osmosis.sender().to_string(),
    //     type_: 3,
    //     fee: None,
    // };

    // let response = terra.commit_any::<bool>(
    //     vec![Any {
    //         type_url: MsgTransfer::TYPE_URL.to_string(),
    //         value: MsgTransfer {
    //             source_port: "transfer".to_string(),
    //             source_channel: TERRA_AXELAR_CHANNEL.to_string(),
    //             token: Some(base::v1beta1::Coin {
    //                 amount: 1_000_000u128.to_string(),
    //                 denom: TERRA_USDC.to_string(),
    //             }),
    //             sender: terra.sender().to_string(),
    //             receiver: "axelar1kxxyh8pwls4nst96n0jz6a25gxkvnthm8qdwnwemcgzmhhakzdhqva5gf8"
    //                 .to_string(),
    //             timeout_height: None,
    //             timeout_timestamp: current_block.time.plus_seconds(PACKET_TIMEOUT).nanos(),
    //             memo: "".to_string(),
    //         }
    //         .encode_to_vec(),
    //     }],
    //     None,
    // )?;

    let interchain = DaemonInterchainEnv::from_daemons(
        &terra.rt_handle,
        vec![terra.clone(), osmosis, axelar],
        &ChannelCreationValidator,
    );

    interchain.wait_ibc("phoenix-1", response)?;

    Ok(())
}

#[cw_serde]
pub struct GmpMessage {
    pub destination_chain: String,
    pub destination_address: String,
    #[serde(rename = "type")]
    pub type_: i64,
    pub fee: Option<Fee>,
}

#[cw_serde]
pub struct Fee {
    pub amount: String,
    pub recipient: String,
}
