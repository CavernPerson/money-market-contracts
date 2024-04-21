use std::error::Error;

use cosmwasm_std::StdError;

pub mod contract;
pub mod msg;
pub mod query;
pub mod state;

pub fn std_error<E: Error>(e: E) -> StdError {
    StdError::generic_err(e.to_string())
}
