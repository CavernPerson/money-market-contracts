use cw_orch::prelude::*;
use cw_orch::{
    daemon::{ChainInfo, ChainKind, NetworkInfo},
    prelude::ContractInstance,
};
use interface::overseer::Overseer;
use moneymarket::overseer::ExecuteMsgFns as _;
use scripts::MIGALOO_1;

fn update_global_test() -> anyhow::Result<()> {
    // First we create a fork testing object
    pretty_env_logger::init();

    let multisig = "migaloo1epn5jukddvlf5v3k6f7cwqc90kgm9ew88wp9jm";

    let overseer_addr = "migaloo18qatlena5eujecsuwrwkpr5qccjddrf8ss4ykzlx8gmrt5dlxxkqysf3lz";

    // We migrate
    let app = cw_orch_fork_mock::ForkMock::new(MIGALOO_1);
    let block = app.block_info();
    println!("block: : {:?}", block);

    // We execute epoch operations on the overseer contract
    let overseer = Overseer::new("overseer", app.clone());
    overseer.set_address(&Addr::unchecked(overseer_addr));

    // let sender = Addr::unchecked(multisig);
    overseer.execute_epoch_operations()?;

    let analysis = app.storage_analysis();

    analysis.compare_all_readable_contract_storage();
    analysis.compare_all_balances();

    Ok(())
}

fn main() {
    update_global_test().unwrap()
}
