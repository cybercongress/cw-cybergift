use crate::state::{Config, MERKLE_ROOT, State, STATE};
use crate::ContractError;
use cosmwasm_std::{
    Decimal, DepsMut, MessageInfo, StdResult, Storage,
    Uint128,
};
use sha2::Digest;
use std::convert::TryInto;
use std::ops::{Add, Mul, Sub};

pub fn update_coefficient(
    store: &mut dyn Storage,
    amount: Uint128,
    config: &Config,
    state: &mut State,
) -> StdResult<()> {
    let coefficient_up = config.coefficient_up;
    let coefficient_down = config.coefficient_down;
    let initial_balance = config.initial_balance;
    let current_balance = state.current_balance;

    let new_balance_ratio = Decimal::from_ratio(current_balance, initial_balance);

    let new_coefficient = Decimal::one()
        .sub(new_balance_ratio)
        .mul(Decimal::from_ratio(coefficient_down, 1u128))
        .add(Decimal::from_ratio(coefficient_up, 1u128).mul(new_balance_ratio));

    state.coefficient = new_coefficient;
    state.current_balance = current_balance - amount;
    STATE.save(store, &state)
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
        return Err(ContractError::VerificationFailed {});
    }
    Ok(true)
}
