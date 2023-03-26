use cosmwasm_std::{ConversionOverflowError, OverflowError, StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot liquidate safely collateralized loan")]
    CannotLiquidateSafeLoan {},

    #[error("An epoch has not passed yet; last executed height: {0}")]
    EpochNotPassed(u64),

    #[error("Token is already registered as collateral")]
    TokenAlreadyRegistered {},

    #[error("Unlock amount cannot exceed locked amount")]
    UnlockExceedsLocked {},

    #[error("Unlock amount too high; Loan liability becomes greater than borrow limit: {0}")]
    UnlockTooLarge(Uint128),

    #[error("LTV should be lower than 1")]
    InvalidLTV {},

    #[error("Distribution factor should be lower than 1")]
    InvalidDistributionFactor {},

    #[error("Too much collaterals were already registered")]
    TooMuchCollaterals {},
}
