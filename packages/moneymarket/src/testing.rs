use crate::mock_querier::mock_dependencies;
use crate::oracle::PriceResponse;
use crate::querier::{query_price, TimeConstraints};
use crate::tokens::{Tokens, TokensHuman, TokensMath, TokensToRaw};

use cosmwasm_std::{Addr, Api, CanonicalAddr, Decimal256, StdError, Uint256};

#[test]
fn oracle_price_querier() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_oracle_price(&[(
        &("terra123123".to_string(), "uusd".to_string()),
        &(
            Decimal256::from_ratio(Uint256::from(131u128), Uint256::from(2u128)),
            123,
            321,
        ),
    )]);

    let oracle_price = query_price(
        deps.as_ref(),
        Addr::unchecked("oracle"),
        "terra123123".to_string(),
        "uusd".to_string(),
        None,
    )
    .unwrap();

    assert_eq!(
        oracle_price,
        PriceResponse {
            rate: Decimal256::from_ratio(Uint256::from(131u128), Uint256::from(2u128)),
            last_updated_base: 123,
            last_updated_quote: 321,
        }
    );

    query_price(
        deps.as_ref(),
        Addr::unchecked("oracle"),
        "terra123123".to_string(),
        "ukrw".to_string(),
        None,
    )
    .unwrap_err();

    let res = query_price(
        deps.as_ref(),
        Addr::unchecked("oracle"),
        "terra123123".to_string(),
        "uusd".to_string(),
        Some(TimeConstraints {
            block_time: 500u64,
            valid_timeframe: 60u64,
        }),
    );

    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Price is too old"),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn tokens_math() {
    let deps = mock_dependencies(&[]);

    let tokens_1: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token2".to_string(), Uint256::from(1000000u64)),
        ("token3".to_string(), Uint256::from(1000000u64)),
        ("token5".to_string(), Uint256::from(1000000u64)),
    ];

    // not existing item
    let tokens_2: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token4".to_string(), Uint256::from(1000000u64)),
    ];

    // not existing item
    let tokens_3: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token6".to_string(), Uint256::from(1000000u64)),
    ];

    // sub bigger than source
    let tokens_4: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token2".to_string(), Uint256::from(1200000u64)),
    ];

    let mut tokens_1_raw: Tokens = tokens_1.to_raw(deps.as_ref()).unwrap();
    let tokens_2_raw: Tokens = tokens_2.to_raw(deps.as_ref()).unwrap();
    let tokens_3_raw: Tokens = tokens_3.to_raw(deps.as_ref()).unwrap();
    let tokens_4_raw: Tokens = tokens_4.to_raw(deps.as_ref()).unwrap();

    assert!(tokens_1_raw.clone().sub(tokens_2_raw).is_err());
    assert!(tokens_1_raw.clone().sub(tokens_3_raw).is_err());
    assert!(tokens_1_raw.sub(tokens_4_raw).is_err());
}

#[test]
fn tokens_math_normal_add() {
    let deps = mock_dependencies(&[]);

    let acct1 = deps
        .api
        .addr_humanize(&CanonicalAddr::from(vec![
            1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))
        .unwrap()
        .to_string();

    let acct2 = deps
        .api
        .addr_humanize(&CanonicalAddr::from(vec![
            1, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))
        .unwrap()
        .to_string();

    let acct3 = deps
        .api
        .addr_humanize(&CanonicalAddr::from(vec![
            1, 1, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))
        .unwrap()
        .to_string();

    let acct4 = deps
        .api
        .addr_humanize(&CanonicalAddr::from(vec![
            1, 1, 1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))
        .unwrap()
        .to_string();

    let acct5 = deps
        .api
        .addr_humanize(&CanonicalAddr::from(vec![
            1, 1, 1, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))
        .unwrap()
        .to_string();

    let tokens_1: TokensHuman = vec![
        (acct1.clone(), Uint256::from(1000000u64)),
        (acct2, Uint256::from(1000000u64)),
        (acct3, Uint256::from(1000000u64)),
        (acct5, Uint256::from(1000000u64)),
    ];

    let tokens_2: TokensHuman = vec![
        (acct1, Uint256::from(1000000u64)),
        (acct4, Uint256::from(1000000u64)),
    ];

    let mut tokens_1_raw: Tokens = tokens_1.to_raw(deps.as_ref()).unwrap();
    let tokens_2_raw: Tokens = tokens_2.to_raw(deps.as_ref()).unwrap();

    tokens_1_raw.add(tokens_2_raw);

    assert_eq!(tokens_1_raw[0].1, Uint256::from(2000000u64));
    assert_eq!(tokens_1_raw.len(), 5);
}

#[test]
fn token_math_zero_token() {
    let deps = mock_dependencies(&[]);

    let tokens_1: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token2".to_string(), Uint256::from(1000000u64)),
    ];

    let tokens_2: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token2".to_string(), Uint256::from(1000000u64)),
    ];

    let mut tokens_1_raw: Tokens = tokens_1.to_raw(deps.as_ref()).unwrap();
    let tokens_2_raw: Tokens = tokens_2.to_raw(deps.as_ref()).unwrap();

    tokens_1_raw.sub(tokens_2_raw).unwrap();
    assert_eq!(tokens_1_raw.len(), 0);
}

#[test]
#[should_panic]
fn token_math_invalid_token() {
    let deps = mock_dependencies(&[]);

    let tokens_1: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token2".to_string(), Uint256::from(1000000u64)),
        ("token3".to_string(), Uint256::from(1000000u64)),
        ("token5".to_string(), Uint256::from(1000000u64)),
    ];

    // duplicated item
    let tokens_2: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token3".to_string(), Uint256::from(1000000u64)),
    ];

    let mut tokens_1_raw: Tokens = tokens_1.to_raw(deps.as_ref()).unwrap();
    let tokens_2_raw: Tokens = tokens_2.to_raw(deps.as_ref()).unwrap();

    let _ = tokens_1_raw.sub(tokens_2_raw);
}

#[test]
#[should_panic]
fn token_math_invalid_token_2() {
    let deps = mock_dependencies(&[]);

    let tokens_1: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token2".to_string(), Uint256::from(1000000u64)),
        ("token2".to_string(), Uint256::from(1000000u64)),
        ("token3".to_string(), Uint256::from(1000000u64)),
        ("token5".to_string(), Uint256::from(1000000u64)),
    ];

    // duplicated item
    let tokens_2: TokensHuman = vec![
        ("token1".to_string(), Uint256::from(1000000u64)),
        ("token2".to_string(), Uint256::from(1000000u64)),
        ("token3".to_string(), Uint256::from(1000000u64)),
    ];

    let mut tokens_1_raw: Tokens = tokens_1.to_raw(deps.as_ref()).unwrap();
    let tokens_2_raw: Tokens = tokens_2.to_raw(deps.as_ref()).unwrap();

    let _ = tokens_1_raw.sub(tokens_2_raw);
}
