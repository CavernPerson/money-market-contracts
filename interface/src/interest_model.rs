use cw_orch::{interface, prelude::*};
use moneymarket::interest_model::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use moneymarket_interest_model::contract::{execute, instantiate, migrate, query};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct InterestModel;

impl<Chain: CwEnv> Uploadable for InterestModel<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("moneymarket_interest_model")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate))
    }
}
