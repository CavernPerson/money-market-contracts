use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Decimal, Decimal256, OwnedDeps,
    Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, Uint256, WasmQuery,
};
use std::collections::HashMap;

use moneymarket::liquidation_queue::LiquidationAmountResponse;
use moneymarket::market::{BorrowerInfoResponse, EpochStateResponse, StateResponse};
use moneymarket::oracle::PriceResponse;
use moneymarket::tokens::TokensHuman;

use std::str::FromStr;

#[cw_serde]

pub enum QueryMsg {
    /// Market state to market contract
    State { block_height: Option<u64> },
    /// Query epoch state to market contract
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
    /// Query loan amount to market contract
    BorrowerInfo {
        borrower: String,
        block_height: Option<u64>,
    },
    /// Query oracle price to oracle contract
    Price { base: String, quote: String },
    /// Query liquidation amount to liquidation model contract
    LiquidationAmount {
        borrow_amount: Uint256,
        borrow_limit: Uint256,
        collaterals: TokensHuman,
        collateral_prices: Vec<Decimal256>,
    },
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    epoch_state_querier: EpochStateQuerier,
    oracle_price_querier: OraclePriceQuerier,
    loan_amount_querier: LoanAmountQuerier,
    liquidation_percent_querier: LiquidationPercentQuerier,
    liquidation_bids_querier: LiquidationBidsQuerier,
}

#[derive(Clone, Default)]
pub struct OraclePriceQuerier {
    // this lets us iterate over all pairs that match the first string
    oracle_price: HashMap<(String, String), (Decimal256, u64, u64)>,
}

#[allow(clippy::type_complexity)]
impl OraclePriceQuerier {
    pub fn new(oracle_price: &[(&(String, String), &(Decimal256, u64, u64))]) -> Self {
        OraclePriceQuerier {
            oracle_price: oracle_price_to_map(oracle_price),
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn oracle_price_to_map(
    oracle_price: &[(&(String, String), &(Decimal256, u64, u64))],
) -> HashMap<(String, String), (Decimal256, u64, u64)> {
    let mut oracle_price_map: HashMap<(String, String), (Decimal256, u64, u64)> = HashMap::new();
    for (base_quote, oracle_price) in oracle_price.iter() {
        oracle_price_map.insert((*base_quote).clone(), **oracle_price);
    }

    oracle_price_map
}

#[derive(Clone, Default)]
pub struct EpochStateQuerier {
    // this lets us iterate over all pairs that match the first string
    epoch_state: HashMap<String, (Uint256, Decimal256, Decimal256)>,
}

impl EpochStateQuerier {
    pub fn new(epoch_state: &[(&String, &(Uint256, Decimal256, Decimal256))]) -> Self {
        EpochStateQuerier {
            epoch_state: epoch_state_to_map(epoch_state),
        }
    }
}

pub(crate) fn epoch_state_to_map(
    epoch_state: &[(&String, &(Uint256, Decimal256, Decimal256))],
) -> HashMap<String, (Uint256, Decimal256, Decimal256)> {
    let mut epoch_state_map: HashMap<String, (Uint256, Decimal256, Decimal256)> = HashMap::new();
    for (market_contract, epoch_state) in epoch_state.iter() {
        epoch_state_map.insert((*market_contract).clone(), **epoch_state);
    }
    epoch_state_map
}

#[derive(Clone, Default)]
pub struct LoanAmountQuerier {
    // this lets us iterate over all pairs that match the first string
    borrower_amount: HashMap<String, Uint256>,
}

impl LoanAmountQuerier {
    pub fn new(borrower_amount: &[(&String, &Uint256)]) -> Self {
        LoanAmountQuerier {
            borrower_amount: borrower_amount_to_map(borrower_amount),
        }
    }
}

pub(crate) fn borrower_amount_to_map(
    borrower_amount: &[(&String, &Uint256)],
) -> HashMap<String, Uint256> {
    let mut borrower_amount_map: HashMap<String, Uint256> = HashMap::new();
    for (market_contract, borrower_amount) in borrower_amount.iter() {
        borrower_amount_map.insert((*market_contract).clone(), **borrower_amount);
    }
    borrower_amount_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

#[derive(Clone, Default)]
pub struct LiquidationPercentQuerier {
    // this lets us iterate over all pairs that match the first string
    liquidation_percent: HashMap<String, Decimal256>,
}

impl LiquidationPercentQuerier {
    pub fn new(liquidation_percent: &[(&String, &Decimal256)]) -> Self {
        LiquidationPercentQuerier {
            liquidation_percent: liquidation_percent_to_map(liquidation_percent),
        }
    }
}

pub(crate) fn liquidation_percent_to_map(
    liquidation_percent: &[(&String, &Decimal256)],
) -> HashMap<String, Decimal256> {
    let mut liquidation_percent_map: HashMap<String, Decimal256> = HashMap::new();
    for (liquidation_contract, liquidation_percent) in liquidation_percent.iter() {
        liquidation_percent_map.insert((*liquidation_contract).clone(), **liquidation_percent);
    }
    liquidation_percent_map
}

#[derive(Clone, Default)]
pub struct LiquidationBidsQuerier {
    // this lets us iterate over all pairs that match the first string
    liquidation_bids: HashMap<String, HashMap<String, Uint128>>,
}

impl LiquidationBidsQuerier {
    pub fn new(liquidation_bids: &[(&String, &[(&String, &Uint128)])]) -> Self {
        LiquidationBidsQuerier {
            liquidation_bids: liquidation_bids_to_map(liquidation_bids),
        }
    }
}

pub(crate) fn liquidation_bids_to_map(
    liquidation_bids: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut liquidation_bids_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (liquidation_contract, bids_struct) in liquidation_bids.iter() {
        let mut bids_map: HashMap<String, Uint128> = HashMap::new();
        for (token, bids) in bids_struct.iter() {
            bids_map.insert(token.to_string(), **bids);
        }

        liquidation_bids_map.insert(liquidation_contract.to_string(), bids_map);
    }

    liquidation_bids_map
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match from_binary(msg).unwrap() {
                    QueryMsg::State { block_height: _ } => {
                        match self.epoch_state_querier.epoch_state.get(contract_addr) {
                            // TODO:
                            Some(_v) => {
                                SystemResult::Ok(ContractResult::from(to_binary(&StateResponse {
                                    total_liabilities: Decimal256::zero(),
                                    total_reserves: Decimal256::zero(),
                                    last_interest_updated: 0,
                                    last_reward_updated: 0,
                                    global_interest_index: Decimal256::from_str("1000000.0")
                                        .unwrap(),
                                    global_reward_index: Decimal256::zero(),
                                    reserves_rate_used_for_borrowers: Decimal256::zero(),
                                    prev_aterra_supply: Uint256::zero(),
                                    prev_exchange_rate: Decimal256::zero(),
                                    prev_borrower_incentives: Uint256::zero(),
                                })))
                            }
                            None => SystemResult::Err(SystemError::InvalidRequest {
                                error: "No epoch state exists".to_string(),
                                request: msg.as_slice().into(),
                            }),
                        }
                    }
                    QueryMsg::EpochState {
                        block_height: _,
                        distributed_interest: _,
                    } => match self.epoch_state_querier.epoch_state.get(contract_addr) {
                        Some(v) => {
                            SystemResult::Ok(ContractResult::from(to_binary(&EpochStateResponse {
                                aterra_supply: v.0,
                                exchange_rate: v.1,
                                reserves_rate_used_for_borrowers: v.2,
                                prev_borrower_incentives: Uint256::zero(),
                                last_interest_updated: 0u64,
                            })))
                        }
                        None => SystemResult::Err(SystemError::InvalidRequest {
                            error: "No epoch state exists".to_string(),
                            request: msg.as_slice().into(),
                        }),
                    },
                    QueryMsg::BorrowerInfo {
                        borrower,
                        block_height: _,
                    } => match self.loan_amount_querier.borrower_amount.get(&borrower) {
                        Some(v) => SystemResult::Ok(ContractResult::from(to_binary(
                            &BorrowerInfoResponse {
                                borrower,
                                interest_index: Decimal256::one(),
                                reward_index: Decimal256::zero(),
                                loan_amount: *v,
                                pending_rewards: Decimal256::zero(),
                            },
                        ))),
                        None => SystemResult::Err(SystemError::InvalidRequest {
                            error: "No borrow amount exists".to_string(),
                            request: msg.as_slice().into(),
                        }),
                    },
                    QueryMsg::Price { base, quote } => {
                        match self.oracle_price_querier.oracle_price.get(&(base, quote)) {
                            Some(v) => {
                                SystemResult::Ok(ContractResult::from(to_binary(&PriceResponse {
                                    rate: v.0,
                                    last_updated_base: v.1,
                                    last_updated_quote: v.2,
                                })))
                            }
                            None => SystemResult::Err(SystemError::InvalidRequest {
                                error: "No oracle price exists".to_string(),
                                request: msg.as_slice().into(),
                            }),
                        }
                    }
                    QueryMsg::LiquidationAmount {
                        borrow_amount,
                        borrow_limit,
                        collaterals,
                        collateral_prices: _,
                    } => {
                        match self
                            .liquidation_percent_querier
                            .liquidation_percent
                            .get(contract_addr)
                        {
                            Some(v) => {
                                if borrow_amount > borrow_limit {
                                    SystemResult::Ok(ContractResult::from(to_binary(
                                        &LiquidationAmountResponse {
                                            collaterals: collaterals
                                                .iter()
                                                .map(|x| {
                                                    let mut bids = Uint128::MAX;
                                                    if let Some(c) = self
                                                        .liquidation_bids_querier
                                                        .liquidation_bids
                                                        .get(contract_addr)
                                                    {
                                                        if let Some(value) = c.get(&x.0) {
                                                            bids = *value;
                                                        }
                                                    }
                                                    (
                                                        x.0.clone(),
                                                        (x.1 * *v).min(Uint256::from(bids)),
                                                    )
                                                })
                                                .collect::<TokensHuman>()
                                                .to_vec(),
                                        },
                                    )))
                                } else {
                                    SystemResult::Ok(ContractResult::from(to_binary(
                                        &LiquidationAmountResponse {
                                            collaterals: vec![],
                                        },
                                    )))
                                }
                            }
                            None => SystemResult::Err(SystemError::InvalidRequest {
                                error: "No liquidation percent exists".to_string(),
                                request: msg.as_slice().into(),
                            }),
                        }
                    }
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            epoch_state_querier: EpochStateQuerier::default(),
            oracle_price_querier: OraclePriceQuerier::default(),
            loan_amount_querier: LoanAmountQuerier::default(),
            liquidation_percent_querier: LiquidationPercentQuerier::default(),
            liquidation_bids_querier: LiquidationBidsQuerier::default(),
        }
    }

    // configure the tax mock querier
    pub fn with_tax(&mut self, _rate: Decimal, _caps: &[(&String, &Uint128)]) {}

    pub fn with_epoch_state(
        &mut self,
        epoch_state: &[(&String, &(Uint256, Decimal256, Decimal256))],
    ) {
        self.epoch_state_querier = EpochStateQuerier::new(epoch_state);
    }

    #[allow(clippy::type_complexity)]
    pub fn with_oracle_price(
        &mut self,
        oracle_price: &[(&(String, String), &(Decimal256, u64, u64))],
    ) {
        self.oracle_price_querier = OraclePriceQuerier::new(oracle_price);
    }

    pub fn with_loan_amount(&mut self, loan_amount: &[(&String, &Uint256)]) {
        self.loan_amount_querier = LoanAmountQuerier::new(loan_amount);
    }

    pub fn with_liquidation_percent(&mut self, liquidation_percent: &[(&String, &Decimal256)]) {
        self.liquidation_percent_querier = LiquidationPercentQuerier::new(liquidation_percent);
    }

    pub fn with_available_bids(&mut self, liquidation_bids: &[(&String, &[(&String, &Uint128)])]) {
        self.liquidation_bids_querier = LiquidationBidsQuerier::new(liquidation_bids);
    }
}
