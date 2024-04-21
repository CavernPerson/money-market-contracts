use cavern_moneymarket_interface::{market::Market, overseer::Overseer};
use cosmwasm_std::{coins, Decimal};
use cw20_base::msg::QueryMsgFns;
use cw_orch::{daemon::networks::PHOENIX_1, prelude::*};
use cw_plus_interface::cw20_base::Cw20Base;
use moneymarket::{
    market::{ExecuteMsgFns as _, QueryMsgFns as _},
    overseer::{ExecuteMsgFns as _, MigrateMsg, QueryMsgFns as _},
};

const SENDER: &str = "terra1ytj0hhw39j88qsx4yapsr6ker83jv3aj354gmj";
const USDC: &str = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
const MULTISIG: &str = "terra12nyw0759lkf2zrxrjjtz8x9mdj502m6jwfanqztwjm3hchfqljts2nwju0";
fn deposit_test() -> anyhow::Result<()> {
    // First we create a fork testing object
    pretty_env_logger::init();
    dotenv::dotenv()?;
    let rt = cw_orch::tokio::runtime::Runtime::new()?;
    let mut app = cw_orch_clone_testing::CloneTesting::new(&rt, PHOENIX_1)?;
    app.set_sender(Addr::unchecked(SENDER));
    app.set_balance(&Addr::unchecked(SENDER), coins(100_000_000_000, USDC))?;

    let market = Market::new("market", app.clone());
    let overseer = Overseer::new("overseer", app.clone());
    let a_terra = Cw20Base::new("a_terra", app.clone());

    // We migrate the overseer to be able to debug
    overseer.upload()?;
    overseer
        .call_as(&Addr::unchecked(MULTISIG))
        .migrate(&MigrateMsg {}, overseer.code_id()?)?;

    // We migrate the market to be able to debug
    market.upload()?;
    market
        .call_as(&Addr::unchecked(MULTISIG))
        .migrate(&moneymarket::market::MigrateMsg {}, market.code_id()?)?;

    let state = overseer.epoch_state()?;
    let balance_one = a_terra.balance(SENDER.to_string())?.balance;
    println!(
        "Overseer deposit rate before, {:?} - balance one {:?}",
        state.deposit_rate, balance_one
    );

    market.deposit_stable(&coins(100_000_000_000, USDC))?;

    let state = overseer.epoch_state()?;
    let balance_two = a_terra.balance(SENDER.to_string())?.balance;
    println!(
        "Overseer deposit rate after, {:?} - balance two {:?} - {:?}",
        state.deposit_rate,
        balance_two,
        balance_two - balance_one
    );
    println!(
        "Exchange_rate for first deposit, {:?}",
        Decimal::from_ratio(100_000_000_000u128, (balance_two - balance_one).u128())
    );

    // We advance time a little and see if the exchange rate is the same

    app.wait_blocks(1000)?;
    app.set_balance(&Addr::unchecked(SENDER), coins(100_000_000_000, USDC))?;
    market.deposit_stable(&coins(100_000_000_000, USDC))?;
    let balance_three = a_terra.balance(SENDER.to_string())?.balance;
    println!(
        "Exchange_rate for second deposit, {:?}",
        Decimal::from_ratio(100_000_000_000u128, (balance_three - balance_two).u128())
    );

    // Now what happens when epoch updates happen
    let epoch_duration = overseer.config()?.epoch_period;
    app.wait_blocks(epoch_duration)?;

    overseer.execute_epoch_operations()?;
    let state = overseer.epoch_state()?;
    println!("Overseer deposit rate after, {:?}", state.deposit_rate,);

    Ok(())
}

fn main() {
    deposit_test().unwrap()
}
