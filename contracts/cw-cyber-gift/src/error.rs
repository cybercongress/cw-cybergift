use cosmwasm_std::{OverflowError, RecoverPubkeyError, StdError, VerificationError};
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

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Your gift is already claimed")]
    Claimed {},

    #[error("Your gift is not claimed yet")]
    NotClaimed {},

    #[error("Gift is not activated yet")]
    NotActivated {},

    #[error("Stage released, wait for the next stage")]
    StageReleased {},

    #[error("Your gift is fully released")]
    GiftReleased {},

    #[error("Wrong length")]
    WrongLength {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Cannot migrate from unsupported version: {previous_version}")]
    CannotMigrateVersion { previous_version: String },

    #[error("Address is not proved with your passport")]
    IsNotProved {},

    #[error("Gift is over")]
    GiftIsOver {},

    #[error("Verification failed")]
    VerificationFailed {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
