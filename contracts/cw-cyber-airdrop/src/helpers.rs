use crate::state::{Config, CONFIG, MERKLE_ROOT};
use crate::ContractError;
use cosmwasm_std::{
    Decimal, DepsMut, MessageInfo, StdError, StdResult, Storage,
    Uint128, VerificationError,
};
use sha2::Digest;
use std::convert::TryInto;
use std::ops::{Add, Mul, Sub};

pub fn update_coefficient(
    store: &mut dyn Storage,
    amount: Uint128,
    config: &mut Config,
) -> StdResult<()> {
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

    let new_coefficient = Decimal::one()
        .sub(new_balance_ratio)
        .mul(Decimal::from_ratio(coefficient_down, 1u128))
        .add(Decimal::from_ratio(coefficient_up, 1u128).mul(new_balance_ratio));

    // TODO delete after debug
    println!("{:?}", new_balance_ratio.to_string());
    println!("{:?}", new_coefficient.to_string());

    config.coefficient = new_coefficient;
    config.current_balance = current_balance - amount;
    CONFIG.save(store, &config)
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
