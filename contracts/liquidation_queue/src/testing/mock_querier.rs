use cosmwasm_std::Empty;
use moneymarket::overseer::{WhitelistResponse, WhitelistResponseElem};
use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Decimal, Decimal256, OwnedDeps,
    Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use std::collections::HashMap;

use moneymarket::oracle::PriceResponse;

#[cw_serde]
pub enum QueryMsg {
    /// Query oracle price to oracle contract
    Price { base: String, quote: String },
    Whitelist {
        collateral_token: Option<String>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let contract_addr = MOCK_CONTRACT_ADDR.to_string();
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(&contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    oracle_price_querier: OraclePriceQuerier,
    collateral_querier: CollateralQuerier,
}

#[derive(Clone, Default)]
pub struct CollateralQuerier {
    collaterals: HashMap<String, Decimal256>,
}

impl CollateralQuerier {
    pub fn new(collaterals: &[(&String, &Decimal256)]) -> Self {
        CollateralQuerier {
            collaterals: collaterals_to_map(collaterals),
        }
    }
}

pub(crate) fn collaterals_to_map(
    collaterals: &[(&String, &Decimal256)],
) -> HashMap<String, Decimal256> {
    let mut collateral_map: HashMap<String, Decimal256> = HashMap::new();
    for (col, max_ltv) in collaterals.iter() {
        collateral_map.insert((*col).clone(), **max_ltv);
    }
    collateral_map
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

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => match from_binary(msg).unwrap() {
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
                QueryMsg::Whitelist {
                    collateral_token,
                    start_after: _,
                    limit: _,
                } => {
                    match self
                        .collateral_querier
                        .collaterals
                        .get(&collateral_token.unwrap())
                    {
                        Some(v) => {
                            SystemResult::Ok(ContractResult::from(to_binary(&WhitelistResponse {
                                elems: vec![WhitelistResponseElem {
                                    name: "name".to_string(),
                                    symbol: "symbol".to_string(),
                                    max_ltv: *v,
                                    custody_contract: "custody0000".to_string(),
                                    collateral_token: "token0000".to_string(),
                                }],
                            })))
                        }
                        None => SystemResult::Err(SystemError::InvalidRequest {
                            error: "".to_string(),
                            request: msg.as_slice().into(),
                        }),
                    }
                }
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            oracle_price_querier: OraclePriceQuerier::default(),
            collateral_querier: CollateralQuerier::default(),
        }
    }

    // configure the tax mock querier
    pub fn with_tax(&mut self, _rate: Decimal, _caps: &[(&String, &Uint128)]) {}

    #[allow(clippy::type_complexity)]
    pub fn with_oracle_price(
        &mut self,
        oracle_price: &[(&(String, String), &(Decimal256, u64, u64))],
    ) {
        self.oracle_price_querier = OraclePriceQuerier::new(oracle_price);
    }

    pub fn with_collateral_max_ltv(&mut self, collaterals: &[(&String, &Decimal256)]) {
        self.collateral_querier = CollateralQuerier::new(collaterals);
    }
}
