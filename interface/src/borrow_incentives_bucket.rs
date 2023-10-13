use cw_orch::{interface, prelude::*};
use moneymarket::bucket::{ExecuteMsg, InstantiateMsg, QueryMsg};
use moneymarket_borrow_incentives_bucket::contract::{execute, instantiate, migrate, query};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct BorrowIncentivesBucket;

impl<Chain: CwEnv> Uploadable for BorrowIncentivesBucket<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("moneymarket_borrow_incentives_bucket")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate))
    }
}
