use cavern_moneymarket_interface::market::Market;
use cosmwasm_std::coins;
use cw_orch::contract::interface_traits::{CwOrchInstantiate, CwOrchUpload};
use cw_orch::daemon::networks::PHOENIX_1;
use cw_orch::prelude::ContractInstance;
use cw_orch::prelude::TxHandler;
use cw_orch::{prelude::DaemonBuilder, tokio::runtime::Runtime};
use ibc_deposit::interface::IbcDeposit;
use ibc_deposit::msg::InstantiateMsg;

pub const PACKET_TIMEOUT: u64 = 60 * 60 * 24 * 7; // 1 week
pub const AXELAR_GMP: &str = "axelar1dv4u5k73pzqrxlzujxg3qp8kvc3pje7jtdvu72npnt5zhq05ejcsn5qme5";

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();

    // We upload code_ids
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(PHOENIX_1)
        .handle(rt.handle())
        .build()?;

    let contract = IbcDeposit::new("ibc-deposit", chain.clone());
    let market = Market::new("market", chain.clone());
    contract.upload()?;

    contract.instantiate(
        &InstantiateMsg {
            market_addr: market.address()?.to_string(),
            transfer_timeout: PACKET_TIMEOUT,
            gmp_receiver: AXELAR_GMP.to_string(),
        },
        Some(&chain.sender()),
        Some(&coins(10_000_000, "uluna")), // For denom creation
    )?;

    Ok(())
}
