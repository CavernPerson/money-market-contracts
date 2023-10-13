use cw_orch::{interface, prelude::*};
use moneymarket::oracle::{ExecuteMsg, InstantiateMsg, QueryMsg};
use moneymarket_oracle::contract::{execute, instantiate, migrate, query};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct Oracle;

impl<Chain: CwEnv> Uploadable for Oracle<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("moneymarket_oracle")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate))
    }
}
