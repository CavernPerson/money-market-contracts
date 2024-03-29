use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_json, Decimal256};
use moneymarket::distribution_model::{
    BorrowerIncentivesRateResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use std::str::FromStr;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        distribution_cap: Decimal256::from_str("100").unwrap(),
        distribution_floor: Decimal256::from_str("10").unwrap(),
        increment_multiplier: Decimal256::percent(110),
        decrement_multiplier: Decimal256::percent(90),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_json(&res).unwrap();
    assert_eq!("owner0000", value.owner.as_str());
    assert_eq!("100", &value.distribution_cap.to_string());
    assert_eq!("10", &value.distribution_floor.to_string());
    assert_eq!("1.1", &value.increment_multiplier.to_string());
    assert_eq!("0.9", &value.decrement_multiplier.to_string());
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        distribution_cap: Decimal256::from_str("100").unwrap(),
        distribution_floor: Decimal256::from_str("10").unwrap(),
        increment_multiplier: Decimal256::percent(110),
        decrement_multiplier: Decimal256::percent(90),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update owner
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("owner0001".to_string()),
        distribution_cap: None,
        distribution_floor: None,
        increment_multiplier: None,
        decrement_multiplier: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_json(&res).unwrap();
    assert_eq!("owner0001", value.owner.as_str());
    assert_eq!("100", &value.distribution_cap.to_string());
    assert_eq!("10", &value.distribution_floor.to_string());
    assert_eq!("1.1", &value.increment_multiplier.to_string());
    assert_eq!("0.9", &value.decrement_multiplier.to_string());

    // Unauthorized err
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        distribution_cap: Some(Decimal256::from_str("100").unwrap()),
        distribution_floor: Some(Decimal256::from_str("10").unwrap()),
        increment_multiplier: Some(Decimal256::percent(110)),
        decrement_multiplier: Some(Decimal256::percent(90)),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn proper_distribution_rate() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        distribution_cap: Decimal256::from_str("100").unwrap(),
        distribution_floor: Decimal256::from_str("10").unwrap(),
        increment_multiplier: Decimal256::percent(110),
        decrement_multiplier: Decimal256::percent(90),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // high = 8.75
    // low = 6.25
    // no changes
    let query_msg = QueryMsg::BorrowerIncentivesRate {
        deposit_rate: Decimal256::percent(7),
        target_deposit_rate: Decimal256::percent(10),
        threshold_deposit_rate: Decimal256::percent(5),
        current_incentives_rate: Decimal256::from_str("99").unwrap(),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let value: BorrowerIncentivesRateResponse = from_json(&res).unwrap();
    assert_eq!("99", &value.incentives_rate.to_string());

    // increment
    let query_msg = QueryMsg::BorrowerIncentivesRate {
        deposit_rate: Decimal256::percent(5),
        target_deposit_rate: Decimal256::percent(10),
        threshold_deposit_rate: Decimal256::percent(5),
        current_incentives_rate: Decimal256::from_str("80").unwrap(),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let value: BorrowerIncentivesRateResponse = from_json(&res).unwrap();
    assert_eq!("88", &value.incentives_rate.to_string());

    // cap
    let query_msg = QueryMsg::BorrowerIncentivesRate {
        deposit_rate: Decimal256::percent(5),
        target_deposit_rate: Decimal256::percent(10),
        threshold_deposit_rate: Decimal256::percent(5),
        current_incentives_rate: Decimal256::from_str("99").unwrap(),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let value: BorrowerIncentivesRateResponse = from_json(&res).unwrap();
    assert_eq!("100", &value.incentives_rate.to_string());

    // decrement
    let query_msg = QueryMsg::BorrowerIncentivesRate {
        deposit_rate: Decimal256::percent(9),
        target_deposit_rate: Decimal256::percent(10),
        threshold_deposit_rate: Decimal256::percent(5),
        current_incentives_rate: Decimal256::from_str("99").unwrap(),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let value: BorrowerIncentivesRateResponse = from_json(&res).unwrap();
    assert_eq!("89.1", &value.incentives_rate.to_string());

    // floor
    let query_msg = QueryMsg::BorrowerIncentivesRate {
        deposit_rate: Decimal256::percent(9),
        target_deposit_rate: Decimal256::percent(10),
        threshold_deposit_rate: Decimal256::percent(5),
        current_incentives_rate: Decimal256::from_str("11").unwrap(),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let value: BorrowerIncentivesRateResponse = from_json(&res).unwrap();
    assert_eq!("10", &value.incentives_rate.to_string());
}
