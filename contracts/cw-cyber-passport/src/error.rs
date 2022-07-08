#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;

use cosmwasm_std::{StdError};
use cw721_base::ContractError as CW721ContractError;
use thiserror::Error;
use cyber_std::particle::ParticleError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Disabled functionality")]
    DisabledFunctionality {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("Address not found")]
    AddressNotFound {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Cannot migrate from unsupported version: {previous_version}")]
    CannotMigrateVersion { previous_version: String },

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    // -----

    #[error("Got a submessage reply with unknown id: {id}")]
    UnknownReplyId { id: u64 },

    // -----

    #[error("Invalid data for the particle")]
    InvalidParticleData {},

    #[error("Invalid particle")]
    InvalidParticle {},

    #[error("Invalid particle version")]
    InvalidParticleVersion {},

    #[error("Invalid initialization")]
    InvalidInitialization {},

    #[error("Wrong token amount for this name")]
    WrongAmountForName {},

    #[error("Name is not valid")]
    NotValidName {},

    #[error("Label is not valid")]
    NotValidLabel {},

    #[error("Data is not valid")]
    NotValidData {},

    #[error("Nickname already exists")]
    NicknameAlreadyExists {},

    #[error("Nickname not found")]
    NicknameNotFound {},

    #[error("Token not found")]
    TokenNotFound {},

    #[error("Cannot add the address to the passport, {msg}")]
    ErrorAddAddress { msg: String },

    #[error("Verification failed")]
    VerificationFailed { msg: String },

    #[error("Data parsing failed")]
    ErrorDataParse { },

    #[error("Key recovery failed")]
    ErrorKeyRecovery { },
}

impl From<ParticleError> for ContractError {
    fn from(msg: ParticleError) -> ContractError {
        match msg {
            ParticleError::InvalidParticleData {} => ContractError::InvalidParticleData {},
            ParticleError::InvalidParticle {} => ContractError::InvalidParticle {},
            ParticleError::InvalidParticleVersion {} => ContractError::InvalidParticleVersion {}
        }
    }
}

impl From<CW721ContractError> for ContractError {
    fn from(msg: CW721ContractError) -> ContractError {
        match msg {
            CW721ContractError::Std(e) => ContractError::Std(e),
            CW721ContractError::Unauthorized {} => ContractError::Unauthorized {},
            CW721ContractError::Claimed {} => ContractError::Claimed {},
            CW721ContractError::Expired {} => ContractError::Expired {},
            CW721ContractError::ApprovalNotFound {spender} => ContractError::ApprovalNotFound {spender}
        }
    }
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
