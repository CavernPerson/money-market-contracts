use crate::external::handle::RewardContractQueryMsg;
use crate::state::BLunaAccruedRewardsResponse;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::Empty;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, BalanceResponse, BankQuery, CanonicalAddr, Coin,
    ContractResult, Decimal, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::TokenInfoResponse;
use moneymarket::astroport_router::QueryMsg as SwapQueryMsg;
use moneymarket::astroport_router::SimulateSwapOperationsResponse;
use std::collections::HashMap;
use std::marker::PhantomData;
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
    token_querier: TokenQuerier,
    accrued_rewards: BLunaAccruedRewardsResponse,
    reward_balance: Uint128,
    other_balance: Uint128,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
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
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let key: &[u8] = key.as_slice();

                let prefix_token_info = to_length_prefixed(b"token_info").to_vec();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();

                let balances: &HashMap<String, Uint128> =
                    match self.token_querier.balances.get(contract_addr) {
                        Some(balances) => balances,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!(
                                    "No balance info exists for the contract {}",
                                    contract_addr
                                ),
                                request: key.into(),
                            })
                        }
                    };

                if key.to_vec() == prefix_token_info {
                    let mut total_supply = Uint128::zero();

                    for balance in balances {
                        total_supply += *balance.1;
                    }

                    SystemResult::Ok(ContractResult::from(to_binary(
                        &to_binary(&TokenInfoResponse {
                            name: "mAPPL".to_string(),
                            symbol: "mAPPL".to_string(),
                            decimals: 6,
                            total_supply,
                        })
                        .unwrap(),
                    )))
                } else if key[..prefix_balance.len()].to_vec() == prefix_balance {
                    let key_address: &[u8] = &key[prefix_balance.len()..];
                    let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);
                    let api: MockApi = MockApi::default();
                    let address: Addr = match api.addr_humanize(&address_raw) {
                        Ok(v) => v,
                        Err(e) => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!("Parsing query request: {}", e),
                                request: key.into(),
                            })
                        }
                    };
                    let balance = match balances.get(&address.to_string()) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.into(),
                            })
                        }
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(
                        &to_binary(&balance).unwrap(),
                    )))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => match contract_addr
                .as_ref()
            {
                "astroport_addr" => match from_binary(msg).unwrap() {
                    SwapQueryMsg::SimulateSwapOperations { offer_amount, .. } => SystemResult::Ok(
                        ContractResult::from(to_binary(&SimulateSwapOperationsResponse {
                            amount: offer_amount * Uint128::from(9u128) / Uint128::from(10u128),
                        })),
                    ),
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "not covered".to_string(),
                        request: msg.clone(),
                    }),
                },
                "phoenix_addr" => match from_binary(msg).unwrap() {
                    SwapQueryMsg::SimulateSwapOperations { offer_amount, .. } => SystemResult::Ok(
                        ContractResult::from(to_binary(&SimulateSwapOperationsResponse {
                            amount: offer_amount * Uint128::from(11u128) / Uint128::from(10u128),
                        })),
                    ),
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "not covered".to_string(),
                        request: msg.clone(),
                    }),
                },
                "terraswap_addr" => match from_binary(msg).unwrap() {
                    SwapQueryMsg::SimulateSwapOperations { offer_amount, .. } => SystemResult::Ok(
                        ContractResult::from(to_binary(&SimulateSwapOperationsResponse {
                            amount: offer_amount,
                        })),
                    ),
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "not covered".to_string(),
                        request: msg.clone(),
                    }),
                },
                _ => match from_binary(msg).unwrap() {
                    RewardContractQueryMsg::AccruedRewards { address: _ } => SystemResult::Ok(
                        ContractResult::from(to_binary(&BLunaAccruedRewardsResponse {
                            rewards: self.accrued_rewards.rewards,
                        })),
                    ),
                },
            },
            QueryRequest::Bank(BankQuery::Balance { address, denom }) => {
                if address == "reward" && denom == "uusd" {
                    let bank_res = BalanceResponse {
                        amount: Coin {
                            amount: self.reward_balance,
                            denom: denom.to_string(),
                        },
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&bank_res)))
                } else {
                    let bank_res = BalanceResponse {
                        amount: Coin {
                            amount: self.other_balance,
                            denom: denom.to_string(),
                        },
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&bank_res)))
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
            token_querier: TokenQuerier::default(),
            accrued_rewards: BLunaAccruedRewardsResponse::default(),
            reward_balance: Uint128::zero(),
            other_balance: Uint128::zero(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    // configure the tax mock querier
    pub fn with_tax(&mut self, _rate: Decimal, _caps: &[(&String, &Uint128)]) {}

    pub fn set_accrued_rewards(&mut self, new_state: BLunaAccruedRewardsResponse) {
        self.accrued_rewards = new_state
    }

    pub fn set_reward_balance(&mut self, balance: Uint128) {
        self.reward_balance = balance
    }

    pub fn set_other_balances(&mut self, balance: Uint128) {
        self.other_balance = balance
    }
}
