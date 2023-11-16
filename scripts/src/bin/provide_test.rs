use cavern_lsd_wrapper_interface::LsdWrapperWithLimit;
use cw_orch::prelude::*;
use cw_orch::{
    prelude::ContractInstance,
};
use cavern_moneymarket_interface::overseer::Overseer;
use moneymarket::overseer::ExecuteMsgFns as _;
use scripts::MIGALOO_1;
use cosmwasm_std::{coins, to_json_binary, Decimal, Uint128};
use moneymarket::custody::Cw20HookMsg::DepositCollateral;

use scripts::migrate_custody::migrate_custody;
use cavern_lsd_wrapper_interface::{WrapperWithLimitQueryMsgFns, WrapperWithLimitExecuteMsgFns};

const SENDER: &str = "migaloo1y9yd9c68agt4zn93h82g6jhlpxcqk8d7cn8c5u";

fn deposit_test() -> anyhow::Result<()> {
    // First we create a fork testing object
    pretty_env_logger::init();
    let mut app = cw_orch_fork_mock::ForkMock::new(MIGALOO_1);
    app.set_sender(Addr::unchecked(SENDER));
    let multisig = "migaloo1epn5jukddvlf5v3k6f7cwqc90kgm9ew88wp9jm";

    let overseer_addr = "migaloo18qatlena5eujecsuwrwkpr5qccjddrf8ss4ykzlx8gmrt5dlxxkqysf3lz";
    let amp_whalet_wrapper_addr = "migaloo187ye4suzmdt8q0p330jae97m892h6v4gkj9sp4uuxpdjatv36n8qt43alj";
    let ampwhalet = "ibc/EA459CE57199098BA5FFDBD3194F498AA78439328A92C7D136F06A5220903DA6";
    let ampwhalet_custody = "migaloo1evdlyeq0mzcmw43v24vr2gc0k08lxmu3459mxzzqk0tv0zw9vlpq342d0s";
    // We execute epoch operations on the overseer contract
    let overseer = Overseer::new("overseer", app.clone());
    overseer.set_address(&Addr::unchecked(overseer_addr));

    let wrapper_token = LsdWrapperWithLimit::new("ampWhalet::token_wrapper", app.clone());
    wrapper_token.set_address(&Addr::unchecked(amp_whalet_wrapper_addr));

    let amount = 5000000u128;

    // // We need to migrate the token contract to the corrected version
    // wrapper_token.upload()?;
    // wrapper_token.call_as(&Addr::unchecked("migaloo1sa8lak0px48l6y7adqvag2d6t3xg7fjynxt4fx")).migrate(&Empty{}, wrapper_token.code_id()?)?;

    let token_info = wrapper_token.token_info()?;
    log::info!("suppply : {:?}", token_info.total_supply);
    let token_balance = app.query_balance(&wrapper_token.address()?, ampwhalet)?;
    log::info!("balances : {:?}", token_balance);

    let wrapper_amount = Uint128::from(amount) * Decimal::from_ratio(token_info.total_supply, token_balance);

    // Let's test the decompound just before executing the message
    wrapper_token.call_as(&Addr::unchecked("migaloo1azsp0gzvk9xa040ctmzyxvtzzlrqq0372e2hkvaktr0pwc7n50mqz9r67x")).decompound(None)?;

    // Let's test mint with
    wrapper_token.mint_with(amount.into(), SENDER.to_string(), &coins(amount, ampwhalet))?;
    wrapper_token.send(wrapper_amount, ampwhalet_custody.to_string(), to_json_binary(&DepositCollateral{borrower: None})?)?;
    overseer.lock_collateral(vec![
        (
            amp_whalet_wrapper_addr.to_string(),
            wrapper_amount.into()
        )
    ])?;

    let analysis = app.storage_analysis();

    analysis.compare_all_readable_contract_storage();
    analysis.compare_all_balances();

    Ok(())
}

fn main() {
    deposit_test().unwrap()
}
