use cosmwasm_std::Empty;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};

pub const IBC_DEPOSIT: &str = "ibc-deposit";
#[cw_orch::interface(InstantiateMsg, ExecuteMsg, Empty, MigrateMsg)]
pub struct IbcDeposit;

#[cfg(not(target_arch = "wasm32"))]
mod interface_impl {
    use super::IbcDeposit;
    use cw_orch::prelude::*;
    use cw_orch::{contract::interface_traits::Uploadable, environment::CwEnv};

    impl<Chain: CwEnv> Uploadable for IbcDeposit<Chain> {
        /// Return the path to the wasm file corresponding to the contract
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path("ibc_deposit")
                .unwrap()
        }
        /// Returns a CosmWasm contract wrapper
        fn wrapper() -> Box<dyn MockContract<Empty>> {
            Box::new(ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ))
        }
    }
}
