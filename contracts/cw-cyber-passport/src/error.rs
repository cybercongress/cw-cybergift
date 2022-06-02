#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;

use cosmwasm_std::{RecoverPubkeyError, StdError};
use cw721_base::ContractError as CW721ContractError;
use thiserror::Error;
use cyber_std::particle::ParticleError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Disabled Functionality")]
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

    #[error("Wrong amount for the name")]
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

    #[error("Address is not eligible to claim airdrop, {msg}")]
    IsNotEligible { msg: String },

    // -----
    // TODO check this overwrites of error messages

    #[error("Invalid hash format")]
    InvalidHashFormat,

    #[error("Invalid signature format")]
    InvalidSignatureFormat,

    #[error("Invalid recovery parameter. Supported values: 0 and 1.")]

    InvalidRecoveryParam,
    #[error("Unknown error: {error_code}")]

    UnknownErr {
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
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

impl From<RecoverPubkeyError> for ContractError {
    fn from(msg: RecoverPubkeyError) -> ContractError {
        match msg {
            RecoverPubkeyError::InvalidHashFormat {} => ContractError::InvalidHashFormat{},
            RecoverPubkeyError::InvalidSignatureFormat {} => ContractError::InvalidHashFormat{},
            RecoverPubkeyError::InvalidRecoveryParam {} => ContractError::InvalidHashFormat{},
            RecoverPubkeyError::UnknownErr {
                error_code, ..
            } => ContractError::UnknownErr {
                error_code,
                #[cfg(feature = "backtraces")]
                backtrace
            },
        }
    }
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
