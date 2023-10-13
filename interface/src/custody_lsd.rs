use cw_orch::{interface, prelude::*};
use moneymarket::custody::{ExecuteMsg, LSDInstantiateMsg, MigrateMsg, QueryMsg};
use moneymarket_custody_lsd::contract::{execute, instantiate, migrate, query};

#[interface(LSDInstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct LsdCustody;

impl<Chain: CwEnv> Uploadable for LsdCustody<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("moneymarket_custody_lsd")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate))
    }
}
