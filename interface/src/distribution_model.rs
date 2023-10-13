use cw_orch::{interface, prelude::*};
use moneymarket::distribution_model::{ExecuteMsg, InstantiateMsg, QueryMsg};
use moneymarket_distribution_model::contract::{execute, instantiate, query};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct DistributionModel;

impl<Chain: CwEnv> Uploadable for DistributionModel<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("moneymarket_distribution_model")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
    }
}
