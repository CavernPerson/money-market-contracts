use cavern_lsd_wrapper_interface::LsdWrapperWithLimit;
use cavern_moneymarket_interface::overseer::Overseer;
use cosmwasm_std::{coins, to_json_binary, Decimal, Uint128};
use cw_orch::prelude::ContractInstance;
use cw_orch::prelude::*;
use moneymarket::custody::Cw20HookMsg::DepositCollateral;
use moneymarket::overseer::ExecuteMsgFns as _;
use scripts::MIGALOO_1;

use cavern_lsd_wrapper_interface::{WrapperWithLimitExecuteMsgFns, WrapperWithLimitQueryMsgFns};
use scripts::migrate_custody::migrate_custody;

const SENDER: &str = "migaloo1y9yd9c68agt4zn93h82g6jhlpxcqk8d7cn8c5u";

fn deposit_test() -> anyhow::Result<()> {
    // First we create a fork testing object
    pretty_env_logger::init();
    let rt = cw_orch::tokio::runtime::Runtime::new()?;
    let mut app = cw_orch_clone_testing::CloneTesting::new(&rt, MIGALOO_1)?;
    app.set_sender(Addr::unchecked(SENDER));
    let multisig = "migaloo1epn5jukddvlf5v3k6f7cwqc90kgm9ew88wp9jm";

    let overseer_addr = "migaloo18qatlena5eujecsuwrwkpr5qccjddrf8ss4ykzlx8gmrt5dlxxkqysf3lz";
    let amp_whalet_wrapper_addr =
        "migaloo14rzz679sk3dsxrhsgcruq55hmnyt5d9cf669zpd7lhhvxfmhcw4sh396p4";
    let ampwhalet = "ibc/BEFB9AB13AB43157A0AF6214AD4B1F565AC0CA0C1760B8337BE7B9E2996F7752";
    let ampwhalet_custody = "migaloo1cwxqzf66r0mfz5ywsxwf4ch0w3eqf5jxcfpqvs2cy5hwf5kkegvs0zc9yu";
    // We execute epoch operations on the overseer contract
    let overseer = Overseer::new("overseer", app.clone());
    overseer.set_address(&Addr::unchecked(overseer_addr));

    let wrapper_token = LsdWrapperWithLimit::new("ampWhalet::token_wrapper", app.clone());
    wrapper_token.set_address(&Addr::unchecked(amp_whalet_wrapper_addr));

    let amount = 1000000u128;

    // // We need to migrate the token contract to the corrected version
    // let contract_admin = "migaloo1qltnqtp22uae923t93cdpk4h2sa44qpryl0nqn";
    // wrapper_token.upload()?;
    // wrapper_token.call_as(&Addr::unchecked(contract_admin)).migrate(&Empty{}, wrapper_token.code_id()?)?;

    let token_info = wrapper_token.token_info()?;
    log::info!("General token info : {:?}", token_info);
    let token_balance = app.query_balance(&wrapper_token.address()?, ampwhalet)?;
    log::info!("balances : {:?}", token_balance);

    let wrapper_amount = if token_balance.is_zero() {
        Uint128::from(amount)
    } else {
        Uint128::from(amount) * Decimal::from_ratio(token_info.total_supply, token_balance)
    };

    // // Let's test the decompound just before executing the message
    // wrapper_token
    //     .call_as(&Addr::unchecked(
    //         "migaloo1azsp0gzvk9xa040ctmzyxvtzzlrqq0372e2hkvaktr0pwc7n50mqz9r67x",
    //     ))
    //     .decompound(None)?;

    // Let's test mint with
    wrapper_token.mint_with(amount.into(), SENDER.to_string(), &coins(amount, ampwhalet))?;

    let new_cw20_balance = wrapper_token.balance(app.sender().to_string())?;
    log::info!("cw20 balance after mint : {:?}", new_cw20_balance);

    wrapper_token.send(
        wrapper_amount,
        ampwhalet_custody.to_string(),
        to_json_binary(&DepositCollateral { borrower: None })?,
    )?;
    overseer.lock_collateral(vec![(
        amp_whalet_wrapper_addr.to_string(),
        wrapper_amount.into(),
    )])?;

    let analysis = app.storage_analysis();

    analysis.compare_all_readable_contract_storage();
    analysis.compare_all_balances();

    Ok(())
}

fn main() {
    deposit_test().unwrap()
}
