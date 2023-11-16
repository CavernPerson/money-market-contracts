use cw_orch::daemon::networks::PHOENIX_1;
use cw_orch::prelude::*;
use cw_orch::{
    daemon::{ChainInfo, ChainKind, NetworkInfo},
    prelude::ContractInstance,
};
use cavern_moneymarket_interface::overseer::Overseer;
use moneymarket::overseer::ExecuteMsgFns as _;
use scripts::MIGALOO_1;
use scripts::migrate_custody::migrate_custody;


pub const ADMIN: &str = "terra1ytj0hhw39j88qsx4yapsr6ker83jv3aj354gmj";
pub const MULTISIG_CONTRACT: &str = "terra12nyw0759lkf2zrxrjjtz8x9mdj502m6jwfanqztwjm3hchfqljts2nwju0";

fn update_global_test() -> anyhow::Result<()> {
    // First we create a fork testing object
    pretty_env_logger::init();

    let overseer_addr = "terra1l6rq7905263uqmayurtulzc09sfcgxdedsfen7m0y6wf28s49tvqdkwau9";

    // We migrate
    let app = cw_orch_fork_mock::ForkMock::new(PHOENIX_1);

    // // We migrate the custody contracts to make sure swaps go through
    let admin = Addr::unchecked(ADMIN);
    let multisig = Addr::unchecked(MULTISIG_CONTRACT);
    migrate_custody(app.clone(), admin, multisig)?;

    // We execute epoch operations on the overseer contract
    let overseer = Overseer::new("overseer", app.clone());
    overseer.set_address(&Addr::unchecked(overseer_addr));
    overseer.execute_epoch_operations()?;

    let analysis = app.storage_analysis();

    analysis.compare_all_readable_contract_storage();
    analysis.compare_all_balances();

    Ok(())
}

fn main() {
    update_global_test().unwrap()
}
