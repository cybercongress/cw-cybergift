use cosmwasm_std::{Deps, StdResult};
use crate::msg::{ClaimResponse, ConfigResponse, IsClaimedResponse, MerkleRootResponse, ReleaseStateResponse};
use crate::state::{CLAIM, CONFIG, MERKLE_ROOT, RELEASE};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner.map(|o| o.to_string()),
        passport: cfg.passport_addr.to_string(),
        target_claim: cfg.target_claim,
        allowed_native: cfg.allowed_native,
        current_balance: cfg.current_balance,
        initial_balance: cfg.initial_balance,
        coefficient_up: cfg.coefficient_up,
        coefficient_down: cfg.coefficient_down,
        coefficient: cfg.coefficient,
        claims: cfg.claims,
        releases: cfg.releases,
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

pub fn query_release_state(deps: Deps, address: String) -> StdResult<ReleaseStateResponse> {
    let release_state = RELEASE.load(deps.storage, address)?;
    let resp = ReleaseStateResponse {
        address: release_state.address.into(),
        balance_claim: release_state.balance_claim,
        stage: release_state.stage,
        stage_expiration: release_state.stage_expiration,
    };
    Ok(resp)
}
