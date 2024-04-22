use cosmwasm_std::coins;
use cw_orch::daemon::networks::{OSMOSIS_1, PHOENIX_1};
use cw_orch::interchain::ChannelCreationValidator;
use cw_orch::interchain::DaemonInterchainEnv;
use cw_orch::prelude::InterchainEnv;
use cw_orch::prelude::TxHandler;
use cw_orch::{prelude::DaemonBuilder, tokio::runtime::Runtime};
use ibc_deposit::interface::IbcDeposit;
use ibc_deposit::msg::ExecuteMsgFns;

pub const PACKET_TIMEOUT: u64 = 60 * 60 * 24 * 7; // 1 week

pub const TERRA_USDC: &str = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
pub const OSMOSIS_CHANNEL: &str = "channel-1";

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

    let contract = IbcDeposit::new("ibc-deposit", terra.clone());

    let response = contract.deposit(
        osmosis.sender().to_string(),
        OSMOSIS_CHANNEL.to_string(),
        &coins(10, TERRA_USDC),
    )?;

    let interchain = DaemonInterchainEnv::from_daemons(
        &terra.rt_handle,
        vec![terra.clone(), osmosis],
        &ChannelCreationValidator,
    );

    interchain.wait_ibc("phoenix-1", response)?;

    Ok(())
}
