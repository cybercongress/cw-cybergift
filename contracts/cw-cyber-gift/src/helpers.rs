use crate::msg::ClaimMsg;
use crate::state::{Config, CONFIG, MERKLE_ROOT};
use crate::ContractError;
use anyhow::Result;
use cosmwasm_std::{from_binary, to_vec, Binary, Deps, DepsMut, MessageInfo, StdError, StdResult, Uint128, VerificationError, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::convert::TryInto;
use serde_json::to_string;
use std::ops::{Add, Mul, Sub};

pub fn update_coefficient(deps: DepsMut, amount: Uint128, config: &mut Config) -> StdResult<()> {
    let coefficient_up = config.coefficient_up;
    let coefficient_down = config.coefficient_down;
    let initial_balance = config.initial_balance;
    let current_balance = config.current_balance;

    // TODO delete after debug
    println!("{:?}", "update_coefficient");
    println!("{:?}", coefficient_up.to_string());
    println!("{:?}", coefficient_down.to_string());
    println!("{:?}", initial_balance.to_string());
    println!("{:?}", current_balance.to_string());
    println!("{:?}", amount.to_string());

    let new_balance_ratio = Decimal::from_ratio(current_balance, initial_balance);

    let new_coefficient = Decimal::one().sub(new_balance_ratio)
        .mul(Decimal::from_ratio(coefficient_down, 1u128))
        .add(Decimal::from_ratio(coefficient_up, 1u128).mul(new_balance_ratio));


    // TODO delete after debug
    println!("{:?}", new_balance_ratio.to_string());
    println!("{:?}", new_coefficient.to_string());

    config.coefficient = new_coefficient;
    config.current_balance = current_balance - amount;
    CONFIG.save(deps.storage, &config)
}

pub fn verify_merkle_proof(
    deps: &DepsMut,
    _info: &MessageInfo,
    claimer: String,
    amount: Uint128,
    proof: Vec<String>,
) -> Result<bool, ContractError> {
    let merkle_root = MERKLE_ROOT.load(deps.storage)?;

    let user_input = format!("{}{}", claimer, amount);
    let hash = sha2::Sha256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .map_err(|_| ContractError::WrongLength {})?;

    let hash = proof.into_iter().try_fold(hash, |hash, p| {
        let mut proof_buf = [0; 32];
        hex::decode_to_slice(p, &mut proof_buf)?;
        let mut hashes = [hash, proof_buf];
        hashes.sort_unstable();
        sha2::Sha256::digest(&hashes.concat())
            .as_slice()
            .try_into()
            .map_err(|_| ContractError::WrongLength {})
    })?;

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root, &mut root_buf)?;
    if root_buf != hash {
        return Err(StdError::verification_err(VerificationError::GenericErr {}).into());
    }
    Ok(true)
}

pub fn verify_eth(
    deps: Deps,
    claim_msg: &ClaimMsg,
    signature: Binary,
) -> Result<bool, ContractError> {
    let mut hasher = Keccak256::new();

    let msg = to_string(claim_msg).map_err(|_| ContractError::InvalidInput {})?;

    hasher.update(format!("\x19Ethereum Signed Message:\n{}", msg.len()));
    hasher.update(msg);
    let hash = hasher.finalize();
    let sig = decode_signature(&signature.clone().to_string())?;
    // Decompose signature
    let (v, rs) = match sig.split_last() {
        Some(pair) => pair,
        None => {
            return Err(ContractError::IsNotEligible {
                msg: "Signature must not be empty".to_string(),
            })
        }
    };
    let recovery = get_recovery_param(*v)?;

    // Verification
    let calculated_pubkey = deps.api.secp256k1_recover_pubkey(&hash, rs, recovery)?;
    let calculated_address = ethereum_address_raw(&calculated_pubkey)?;
    let signer_address = decode_address(claim_msg.gift_claiming_address.clone().as_str())?;
    if signer_address != calculated_address {
        return Err(ContractError::IsNotEligible {
            msg: "signer address is not calculated addr".to_string(),
        });
    }
    deps.api
        .secp256k1_verify(&hash, rs, &calculated_pubkey)
        .map_err(|err| ContractError::IsNotEligible {
            msg: err.to_string(),
        })
}

fn get_recovery_param(v: u8) -> StdResult<u8> {
    match v {
        27 => Ok(0),
        28 => Ok(1),
        _ => Err(StdError::generic_err("Values of v other than 27 and 28 not supported. Replay protection (EIP-155) cannot be used here."))
    }
}

/// Returns a raw 20 byte Ethereum address
fn ethereum_address_raw(pubkey: &[u8]) -> StdResult<[u8; 20]> {
    let (tag, data) = match pubkey.split_first() {
        Some(pair) => pair,
        None => return Err(StdError::generic_err("Public key must not be empty")),
    };
    if *tag != 0x04 {
        return Err(StdError::generic_err("Public key must start with 0x04"));
    }
    if data.len() != 64 {
        return Err(StdError::generic_err("Public key must be 65 bytes long"));
    }

    let hash = Keccak256::digest(data);
    Ok(hash[hash.len() - 20..].try_into().unwrap())
}

/// Returns a raw 20 byte Ethereum address from hex
pub fn decode_address(input: &str) -> StdResult<[u8; 20]> {
    if input.len() != 42 {
        return Err(StdError::generic_err(
            "Ethereum address must be 42 characters long",
        ));
    }
    if !input.starts_with("0x") {
        return Err(StdError::generic_err("Ethereum address must start wit 0x"));
    }
    let data = hex::decode(&input[2..]).map_err(|_| StdError::generic_err("hex decoding error"))?;
    Ok(data.try_into().unwrap())
}

/// Returns a raw 65 byte Ethereum signature from hex
pub fn decode_signature(input: &str) -> StdResult<[u8; 65]> {
    if input.len() != 132 {
        return Err(StdError::generic_err(
            "Ethereum signature must be 132 characters long",
        ));
    }
    if !input.starts_with("0x") {
        return Err(StdError::generic_err("Ethereum signature must start wit 0x"));
    }
    let data = hex::decode(&input[2..]).map_err(|_| StdError::generic_err("hex decoding error"))?;
    Ok(data.try_into().unwrap())
}

pub fn verify_cosmos(
    deps: Deps,
    claim_msg: &ClaimMsg,
    signature: Binary,
) -> Result<bool, ContractError> {
    let msg_raw = to_vec(claim_msg)?;
    // Hashing
    let hash = Sha256::digest(&msg_raw);
    // Verification
    let sig:CosmosSignature = from_binary(&signature).unwrap();
    let result = deps.api
        .secp256k1_verify(hash.as_ref(), sig.signature.as_slice(), sig.pub_key.as_slice())
        .map_err(|err| ContractError::IsNotEligible {
            msg: err.to_string(),
        });
    return result;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CosmosSignature {
    pub_key: Binary,
    signature: Binary,
}
