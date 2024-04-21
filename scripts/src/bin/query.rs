use cavern_lsd_wrapper_interface::LsdWrapperWithLimit;
use cavern_moneymarket_interface::market::Market;
use cavern_moneymarket_interface::overseer::Overseer;
use cosmwasm_std::{coins, to_json_binary, Decimal, Order, Uint128};
use cw_orch::daemon::queriers::Node;
use cw_orch::prelude::ContractInstance;
use cw_orch::prelude::*;
use moneymarket::overseer::{ExecuteMsgFns as _, MigrateMsg};
use moneymarket::{custody::Cw20HookMsg::DepositCollateral, overseer::QueryMsgFns};
use scripts::MIGALOO_1;

use cavern_lsd_wrapper_interface::{WrapperWithLimitExecuteMsgFns, WrapperWithLimitQueryMsgFns};
use moneymarket::market::QueryMsgFns as _;
use scripts::migrate_custody::migrate_custody;

const SENDER: &str = "migaloo1y9yd9c68agt4zn93h82g6jhlpxcqk8d7cn8c5u";

fn deposit_test() -> anyhow::Result<()> {
    // First we create a fork testing object
    pretty_env_logger::init();
    let rt = cw_orch::tokio::runtime::Runtime::new()?;
    let mut app = cw_orch_clone_testing::CloneTesting::new(&rt, MIGALOO_1)?;
    app.set_sender(Addr::unchecked(SENDER));
    let overseer_addr = "migaloo18qatlena5eujecsuwrwkpr5qccjddrf8ss4ykzlx8gmrt5dlxxkqysf3lz";
    let market_addr = "migaloo1qelh4gv5drg3yhj282l6n84a6wrrz033kwyak3ee3syvqg3mu3msgphpk4";
    let usdc = "ibc/BC5C0BAFD19A5E4133FDA0F3E04AE1FBEE75A4A226554B2CBB021089FF2E1F8A";
    let borrow_reserves = "migaloo1ufk6fm6tlh8rxjtdx6pfejn3clmw905dxkt2c2xpxsy8prshzqdsfsf93d";

    let overseer = Overseer::new("overseer", app.clone());
    overseer.set_address(&Addr::unchecked(overseer_addr));

    let market = Market::new("market", app.clone());
    market.set_address(&Addr::unchecked(market_addr));

    let config = overseer.config()?;
    overseer.dynrate_state()?;

    let overseer_balance = app
        .bank_querier()
        .balance(overseer.address()?, Some(usdc.to_string()))?;
    let borrow_reserves = app
        .bank_querier()
        .balance(borrow_reserves, Some(usdc.to_string()))?;
    log::info!("overseer balance {:?}", overseer_balance);
    log::info!("overseer balance {:?}", borrow_reserves);
    let market_reserve = market.state(None)?;

    let contract_admin = "migaloo1sxrz3w7xzrccs5avm5q6qgysyjzug86q98p04qqtxsjcxq22zmesm7hafl";
    overseer.upload()?;
    overseer
        .call_as(&Addr::unchecked(contract_admin))
        .migrate(&MigrateMsg {}, overseer.code_id()?)?;

    overseer.execute_epoch_operations()?;

    // Let's do an epoch update with a migrated contract to be able to debug

    Ok(())
}

fn main() {
    deposit_test().unwrap()
}
