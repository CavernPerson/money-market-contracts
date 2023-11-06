use cw_orch::{interface, prelude::*};
use moneymarket::custody::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use moneymarket_custody_lsd::contract::{execute, instantiate, migrate, query};

use crate::WASM_SUFFIX;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct LsdCustody;

impl<Chain: CwEnv> Uploadable for LsdCustody<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path(&format!("moneymarket_custody_lsd{}", WASM_SUFFIX))
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate))
    }
}
