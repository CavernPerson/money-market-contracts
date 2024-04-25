use cavern_moneymarket_interface::market::Market;
use cosmwasm_std::{coins, Empty};
use cw_orch::contract::interface_traits::{CwOrchInstantiate, CwOrchUpload};
use cw_orch::daemon::networks::PHOENIX_1;
use cw_orch::prelude::ContractInstance;
use cw_orch::prelude::TxHandler;
use cw_orch::{prelude::*, tokio::runtime::Runtime};
use ibc_deposit::interface::IbcDeposit;
use ibc_deposit::msg::InstantiateMsg;

pub const PACKET_TIMEOUT: u64 = 60 * 60 * 24 * 7; // 1 week

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
    contract.upload()?;
    contract.migrate(&Empty {}, contract.code_id()?)?;
    let market = Market::new("market", chain.clone());
    Ok(())
}
