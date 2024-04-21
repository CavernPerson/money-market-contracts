use cavern_lsd_wrapper_interface::LsdWrapperWithLimit;
use cavern_moneymarket_interface::market::Market;
use cavern_moneymarket_interface::overseer::Overseer;
use cosmwasm_std::{coins, to_json_binary, Decimal, Order, Uint128};
use cw_orch::daemon::queriers::Node;
use cw_orch::prelude::ContractInstance;
use cw_orch::prelude::*;
use moneymarket::overseer::ExecuteMsgFns as _;
use moneymarket::{custody::Cw20HookMsg::DepositCollateral, overseer::QueryMsgFns};
use scripts::MIGALOO_1;

use cavern_lsd_wrapper_interface::{WrapperWithLimitExecuteMsgFns, WrapperWithLimitQueryMsgFns};
use cosmrs::proto::cosmos::{base::query::v1beta1::PageRequest, tx::v1beta1::OrderBy};

use cw_orch::tokio::runtime::Runtime;
use moneymarket::market::QueryMsgFns as _;
use scripts::migrate_custody::migrate_custody;
const SENDER: &str = "migaloo1y9yd9c68agt4zn93h82g6jhlpxcqk8d7cn8c5u";

fn deposit_test() -> anyhow::Result<()> {
    // First we create a fork testing object
    pretty_env_logger::init();
    let rt = cw_orch::tokio::runtime::Runtime::new()?;
    let mut chain = MIGALOO_1;
    chain.grpc_urls = &[" migaloo-mainnet-grpc.autostake.com:443"];
    let mut app = cw_orch_clone_testing::CloneTesting::new(&rt, MIGALOO_1)?;
    app.set_sender(Addr::unchecked(SENDER));
    let overseer_addr = "migaloo18qatlena5eujecsuwrwkpr5qccjddrf8ss4ykzlx8gmrt5dlxxkqysf3lz";
    let market_addr = "migaloo1qelh4gv5drg3yhj282l6n84a6wrrz033kwyak3ee3syvqg3mu3msgphpk4";
    let usdc = "ibc/BC5C0BAFD19A5E4133FDA0F3E04AE1FBEE75A4A226554B2CBB021089FF2E1F8A";
    let borrow_reserves = "migaloo1ufk6fm6tlh8rxjtdx6pfejn3clmw905dxkt2c2xpxsy8prshzqdsfsf93d";

    // Querying for events on the deposit rate epoch operations
    let grpc_channel = app.state.borrow().daemon_state.grpc_channel.clone();
    let rt = Runtime::new()?;

    let node = Node::new_async(grpc_channel);

    let mut deposit_rates = vec![];
    for page in 1..2 {
        let events = rt.block_on(node._find_tx_by_events(
            vec!["wasm.action='epoch_operations'".to_string()],
            Some(page),
            Some(OrderBy::Desc),
        ))?;

        deposit_rates.extend(events.iter().map(|tx| {
            let events = tx.get_events("wasm");
            let attributes = events
                .iter()
                .map(|e| e.get_attributes("deposit_rate"))
                .flatten()
                .collect::<Vec<_>>();
            println!("attributes : {:?}", attributes.len());

            (
                attributes.last().unwrap().value.clone(),
                tx.event_attr_value("wasm", "interest_buffer").unwrap(),
                tx.timestamp,
            )
        }));
    }
    println!("deposit_rates: {:?}", deposit_rates);
    Ok(())
}

fn main() {
    deposit_test().unwrap()
}
