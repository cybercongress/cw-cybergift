use cosmwasm_std::{Addr, attr, BankMsg, Coin, Decimal, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Uint64};

use crate::error::ContractError;
use crate::helpers::{update_coefficient, verify_merkle_proof};
use crate::msg::{PassportSignedQuery, PassportSignedResponse, PassportAddressQuery, PassportAddressResponse};
use crate::state::{ReleaseState, CLAIM, CONFIG, MERKLE_ROOT, RELEASE};
use cw_utils::{Expiration, DAY};
use std::ops::{Add, Mul};

const RELEASE_STAGES: u64 = 9;



pub fn execute_update_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut tmp_owner = None;
    if let Some(addr) = new_owner {
        tmp_owner = Some(deps.api.addr_validate(&addr)?)
    }

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.owner = tmp_owner;
        Ok(exists)
    })?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_owner")]))
}

pub fn execute_update_passport(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_passport: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let passport = deps.api.addr_validate(&new_passport)?;

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.passport_addr = passport;
        Ok(exists)
    })?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_passport"),
        attr("passport", new_passport),
    ]))
}

pub fn execute_update_target(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_target: Uint64,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.target_claim = new_target;
        Ok(exists)
    })?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_target"),
        attr("target", new_target.to_string()),
    ]))
}

pub fn execute_register_merkle_root(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    merkle_root: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root.to_string(), &mut root_buf)?;

    MERKLE_ROOT.save(deps.storage, &merkle_root)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "register_merkle_root"),
        attr("merkle_root", merkle_root),
    ]))
}

const CLAIM_BOUNTY: u128 =  100000;

pub fn execute_claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    nickname: String,
    gift_claiming_address: String,
    gift_amount: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {
    let claimed = CLAIM.may_load(deps.storage, gift_claiming_address.clone())?;
    if claimed.is_some() {
        return Err(ContractError::Claimed {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    let claim_amount = gift_amount * config.coefficient;

    // TODO: delete after debug
    println!("{:?}", "execute_claim");
    println!("{:?}", config.coefficient);
    println!("{:?}", claim_amount.to_string());

    if config.current_balance < claim_amount {
        return Err(ContractError::GiftIsOver {});
    }

    // is_eligible(deps.as_ref(), &claim_msg, signature)?;

    let res:PassportSignedResponse = deps.querier.query_wasm_smart(
        config.clone().passport_addr,
        &PassportSignedQuery{
                nickname: nickname.clone(),
                address: gift_claiming_address.clone()
        }
    )?;

    if res.signed == false {
        return Err(ContractError::IsNotEligible { msg: "passport isn't assigned".to_string() });
    }

    verify_merkle_proof(
        &deps,
        &info,
        gift_claiming_address.clone(),
        gift_amount.clone(),
        proof,
    )?;

    CLAIM.save(deps.storage, gift_claiming_address.clone(), &true)?;

    update_coefficient(deps.storage, claim_amount, &mut config)?;

    let res:PassportAddressResponse = deps.querier.query_wasm_smart(
        config.passport_addr,
        &PassportAddressQuery{
            nickname: nickname.clone(),
        }
    )?;

    let release_state = ReleaseState {
        address: Addr::unchecked(res.clone().address),
        balance_claim: claim_amount.checked_sub(Uint128::new(CLAIM_BOUNTY))?,
        stage: Uint64::zero(),
        stage_expiration: Expiration::Never {},
    };

    RELEASE.save(
        deps.storage,
        gift_claiming_address.clone(),
        &release_state,
    )?;

    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.claims = cfg.claims.add(Uint64::new(1));
        Ok(cfg)
    })?;

    let res = Response::new()
        .add_message(BankMsg::Send {
            to_address: res.address.clone(),
            amount: vec![Coin {
                denom: config.allowed_native,
                amount: Uint128::new(CLAIM_BOUNTY),
            }],
        })
        .add_attributes(vec![
            attr("action", "claim"),
            attr("original", gift_claiming_address),
            attr("target", res.address),
            attr("amount", claim_amount),
        ]);
    Ok(res)
}

pub fn execute_release(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    gift_address: String
) -> Result<Response, ContractError> {
    let claimed = CLAIM.may_load(deps.storage, gift_address.clone())?;
    if claimed.is_none() {
        return Err(ContractError::NotClaimed {});
    }

    let config = CONFIG.load(deps.storage)?;
    if config.claims < config.target_claim {
        return Err(ContractError::NotActivated {});
    }

    let mut release_state = RELEASE.load(deps.storage, gift_address.clone())?;

    if release_state.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if release_state.balance_claim.is_zero() {
        return Err(ContractError::GiftReleased {});
    }

    let amount: Uint128;
    if release_state.stage.is_zero() {
        amount = release_state.balance_claim.mul(Decimal::percent(10));
        release_state.stage_expiration = DAY.after(&env.block);
        release_state.stage = Uint64::new(RELEASE_STAGES);
    } else {
        if release_state.stage_expiration.is_expired(&env.block) {
            if release_state.stage.u64() == 1 {
                amount = release_state.balance_claim;
                release_state.stage_expiration = Expiration::Never {};
                release_state.stage = Uint64::zero();
            } else {
                amount = release_state
                    .balance_claim
                    .mul(Decimal::from_ratio(1u128, release_state.stage));
                release_state.stage_expiration = DAY.after(&env.block);
                release_state.stage = release_state.stage.checked_sub(Uint64::new(1))?;
            }
        } else {
            return Err(ContractError::StageReleased {});
        }
    }

    release_state.balance_claim = release_state.balance_claim - amount;

    RELEASE.save(
        deps.storage,
        gift_address.clone(),
        &release_state,
    )?;

    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.releases = cfg.releases.add(Uint64::new(1));
        Ok(cfg)
    })?;

    let res = Response::new()
        .add_message(BankMsg::Send {
            to_address: release_state.clone().address.into(),
            amount: vec![Coin {
                denom: config.allowed_native,
                amount,
            }],
        })
        .add_attributes(vec![
            attr("action", "release"),
            attr("address", release_state.clone().address.to_string()),
            attr("gift_address", gift_address),
            attr("stage", release_state.stage.to_string()),
            attr("amount", amount),
        ]);
    Ok(res)
}
