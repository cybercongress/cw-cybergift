use cosmwasm_std::{RecoverPubkeyError, StdError, VerificationError};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Hex(#[from] FromHexError),

    #[error("{0}")]
    Pubkey(#[from] RecoverPubkeyError),

    #[error("{0}")]
    Verification(#[from] VerificationError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid input")]
    InvalidInput {},

    #[error("Already claimed")]
    Claimed {},

    #[error("Wrong length")]
    WrongLength {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Address is not eligible to claim airdrop, {msg}")]
    IsNotEligible { msg: String },
}
