use cw_orch::{interface, prelude::*};
use moneymarket::market::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use moneymarket_market::contract::{execute, instantiate, migrate, query};

use crate::WASM_SUFFIX;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Market;

impl<Chain: CwEnv> Uploadable for Market<Chain> {
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate))
    }
}
