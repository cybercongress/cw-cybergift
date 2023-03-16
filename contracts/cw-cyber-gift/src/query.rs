use cosmwasm_std::{Deps, StdResult};
use crate::msg::{ClaimResponse, ConfigResponse, IsClaimedResponse, MerkleRootResponse, StateResponse};
use crate::state::{CLAIM, CONFIG, MERKLE_ROOT, STATE};


pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner.map(|o| o.to_string()),
        passport: cfg.passport_addr.to_string(),
        target_claim: cfg.target_claim,
        allowed_native: cfg.allowed_native,
        initial_balance: cfg.initial_balance,
        coefficient_up: cfg.coefficient_up,
        coefficient_down: cfg.coefficient_down,
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let stt = STATE.load(deps.storage)?;
    Ok(StateResponse {
        current_balance: stt.current_balance,
        coefficient: stt.coefficient,
        claims: stt.claims,
    })
}

pub fn query_merkle_root(deps: Deps) -> StdResult<MerkleRootResponse> {
    let merkle_root = MERKLE_ROOT.load(deps.storage)?;
    let resp = MerkleRootResponse { merkle_root };

    Ok(resp)
}

pub fn query_is_claimed(deps: Deps, address: String) -> StdResult<IsClaimedResponse> {
    let claim = CLAIM.may_load(deps.storage, address)?;
    let mut is_claimed = false;
    if claim.is_some() {
        is_claimed = true;
    }
    let resp = IsClaimedResponse { is_claimed };

    Ok(resp)
}

pub fn query_claim(deps: Deps, address: String) -> StdResult<ClaimResponse> {
    let claim = CLAIM.load(deps.storage, address)?;

    let resp = ClaimResponse {
        claim: claim.claim,
        multiplier: claim.multiplier
    };

    Ok(resp)
}
