use cw_orch::{
    interface,
    prelude::*,
};
use moneymarket::overseer::{
    ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg
};

use moneymarket_overseer::contract::{instantiate, execute, query, migrate};

use crate::WASM_SUFFIX;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Overseer;

impl<Chain: CwEnv> Uploadable for Overseer<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path(&format!("moneymarket_overseer{}", WASM_SUFFIX))
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                execute,
                instantiate,
                query,
            )
            .with_migrate(migrate)
        )
    }
}
