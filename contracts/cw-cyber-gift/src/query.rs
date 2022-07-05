use cosmwasm_std::{Deps, Order, StdResult, Uint64};
use cw_storage_plus::Bound;
// use test::RunIgnored::No;
use crate::msg::{AllReleaseStageStateResponse, ClaimResponse, ConfigResponse, IsClaimedResponse, MerkleRootResponse, ReleaseStageStateResponse, ReleaseStateResponse, StateResponse};
use crate::state::{CLAIM, CONFIG, MERKLE_ROOT, RELEASE, RELEASE_INFO, STATE};


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
        releases: stt.releases
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

pub fn query_release_stage_state(deps: Deps, stage: Uint64) -> StdResult<ReleaseStageStateResponse> {
    let release_stage_state = RELEASE_INFO.load(deps.storage, stage.u64())?;
    let resp = ReleaseStageStateResponse {
        releases: release_stage_state
    };
    Ok(resp)
}

pub fn query_all_release_stage_state(deps: Deps, start: Option<u8>, limit: Option<u8>) -> StdResult<AllReleaseStageStateResponse> {
    let start = start.map(Bound::inclusive);
    let limit = limit.unwrap_or(100) as usize;

    let all_release_stage_state = RELEASE_INFO
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.unwrap().1)
        .collect::<Vec<Uint64>>();
    let resp = AllReleaseStageStateResponse {
        releases: all_release_stage_state
    };
    Ok(resp)
}
