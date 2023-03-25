use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("A feeder already exists for {0}, please update instead")]
    FeederExists(String),

    #[error("There is no feeder registered for {0}, please register instead")]
    FeederDoesntExist(String),

    #[error("You can't provide a price equals 0")]
    PriceCantBeZero {},
}
