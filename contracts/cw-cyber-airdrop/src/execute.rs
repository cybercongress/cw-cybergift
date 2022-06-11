use cosmwasm_std::{Addr, attr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Empty, Env, MessageInfo, Response, StdResult, to_binary, Uint128, Uint64, WasmMsg};

use crate::error::ContractError;
use crate::helpers::{update_coefficient, verify_merkle_proof};
use crate::state::{ReleaseState, CLAIM, CONFIG, MERKLE_ROOT, RELEASE, ClaimState, STATE, RELEASE_INFO};
use cw_utils::{Expiration, DAY, HOUR};
use std::ops::{Add, Mul};
use cw_cyber_passport::msg::{QueryMsg as PassportQueryMsg};
use crate::msg::{AddressResponse, SignatureResponse};
use cw1_subkeys::msg::{ExecuteMsg as Cw1ExecuteMsg};

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

    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.passport_addr = passport;
        Ok(cfg)
    })?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_passport"),
        attr("passport", new_passport),
    ]))
}

pub fn execute_update_treasury(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_treasury: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let treasury = deps.api.addr_validate(&new_treasury)?;

    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.treasury_addr = treasury;
        Ok(cfg)
    })?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_treasury"),
        attr("treasury", new_treasury),
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

    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.target_claim = new_target;
        Ok(cfg)
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
    mut gift_claiming_address: String,
    gift_amount: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {
    gift_claiming_address = gift_claiming_address.to_lowercase();

    let claimed = CLAIM.may_load(deps.storage, gift_claiming_address.clone())?;
    if claimed.is_some() {
        return Err(ContractError::Claimed {});
    }

    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let claim_amount = gift_amount * state.coefficient;

    if state.current_balance < claim_amount {
        return Err(ContractError::GiftIsOver {});
    }

    let res: SignatureResponse = deps.querier.query_wasm_smart(
        config.clone().passport_addr,
        &PassportQueryMsg::PassportSigned{
                nickname: nickname.clone(),
                address: gift_claiming_address.clone()
        }
    )?;

    if res.signed == false {
        return Err(ContractError::IsNotEligible { msg: "passport isn't assigned".to_string() });
    }

    // returns error of proof is invalid
    verify_merkle_proof(
        &deps,
        &info,
        gift_claiming_address.clone(),
        gift_amount.clone(),
        proof,
    )?;

    // only claim once by given verified address
    CLAIM.save(deps.storage, gift_claiming_address.clone(), &ClaimState{ claim: claim_amount, multiplier: state.coefficient })?;

    update_coefficient(deps.storage, claim_amount, &config, &mut state)?;

    // get address of the passport by nickname
    let res: AddressResponse = deps.querier.query_wasm_smart(
        config.passport_addr,
        &PassportQueryMsg::AddressByNickname {
            nickname: nickname.clone(),
        }
    )?;

    let release_state = ReleaseState {
        address: Addr::unchecked(res.clone().address),
        balance_claim: claim_amount.checked_sub(Uint128::new(CLAIM_BOUNTY))?,
        // balance_claim: claim_amount.checked_sub(Uint128::new(0))?,
        stage: Uint64::zero(),
        stage_expiration: Expiration::Never {},
    };

    RELEASE.save(
        deps.storage,
        gift_claiming_address.clone(),
        &release_state,
    )?;

    STATE.update(deps.storage, |mut stt| -> StdResult<_> {
        stt.claims = stt.claims.add(Uint64::new(1));
        Ok(stt)
    })?;

    // send funds from treasury controlled by Congress
    Ok(Response::new()
       .add_message(WasmMsg::Execute {
           contract_addr: config.treasury_addr.to_string(),
           msg: to_binary(&Cw1ExecuteMsg::Execute::<Empty> {
               msgs: vec![
                   CosmosMsg::Bank(BankMsg::Send {
                       to_address: res.address.clone(),
                       amount: vec![Coin {
                           denom: config.allowed_native,
                           amount: Uint128::new(CLAIM_BOUNTY),
                       }],
                   }).into()
               ]})?,
           funds: vec![]
       })
       //  .add_message(BankMsg::Send {
       //      to_address: res.address.clone(),
       //      amount: vec![Coin {
       //          denom: config.allowed_native,
       //          amount: Uint128::new(CLAIM_BOUNTY),
       //      }],
       //  })
       .add_attributes(vec![
           attr("action", "claim"),
           attr("original", gift_claiming_address),
           attr("target", res.address),
           attr("amount", claim_amount),
       ])
    )
}

pub fn execute_release(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut gift_address: String
) -> Result<Response, ContractError> {
    gift_address = gift_address.to_lowercase();

    let claimed = CLAIM.may_load(deps.storage, gift_address.clone())?;
    if claimed.is_none() {
        return Err(ContractError::NotClaimed {});
    }

    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    if state.claims < config.target_claim {
        return Err(ContractError::NotActivated {});
    }

    let mut release_state = RELEASE.load(deps.storage, gift_address.clone())?;

    if release_state.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if release_state.balance_claim.is_zero() {
        return Err(ContractError::GiftReleased {});
    }

    // TODO change HOUR to DAY after tests
    let amount: Uint128;
    if release_state.stage.is_zero() {
        // first claim, amount 10% of claim
        amount = release_state
            .balance_claim
            .mul(Decimal::percent(10));
        release_state.stage_expiration = HOUR.after(&env.block);
        release_state.stage = Uint64::new(RELEASE_STAGES);
    } else {
        if release_state.stage_expiration.is_expired(&env.block) {
            // last claim, amount is rest
            if release_state.stage.u64() == 1 {
                amount = release_state.balance_claim;
                release_state.stage_expiration = Expiration::Never {};
                release_state.stage = Uint64::zero();
            } else {
                // amount is equal during all intermediate stages
                amount = release_state
                    .balance_claim
                    .mul(Decimal::from_ratio(1u128, release_state.stage));
                release_state.stage_expiration = HOUR.after(&env.block);
                release_state.stage = release_state.stage.checked_sub(Uint64::new(1))?;
            }
        } else {
            return Err(ContractError::StageReleased {});
        }
    }

    release_state.balance_claim = release_state.balance_claim - amount;

    RELEASE_INFO.update(deps.storage, release_state.stage.u64(), |rls: Option<Uint64>| -> StdResult<Uint64> {
        Ok(rls.unwrap_or_default().add(Uint64::new(1)))
    })?;

    RELEASE.save(
        deps.storage,
        gift_address.clone(),
        &release_state,
    )?;

    STATE.update(deps.storage, |mut stt| -> StdResult<_> {
        stt.releases = stt.releases.add(Uint64::new(1));
        Ok(stt)
    })?;

    // send funds from treasury controlled by Congress
    Ok(Response::new()
       .add_message(WasmMsg::Execute {
           contract_addr: config.treasury_addr.to_string(),
           msg: to_binary(&Cw1ExecuteMsg::Execute::<Empty> {
               msgs: vec![
                   CosmosMsg::Bank(BankMsg::Send {
                   to_address: release_state.clone().address.into(),
                   amount: vec![Coin {
                       denom: config.allowed_native,
                       amount: amount,
                   }],
               }).into()
           ]})?,
           funds: vec![]
       })
        // .add_message(BankMsg::Send {
        //     to_address: release_state.clone().address.into(),
        //     amount: vec![Coin {
        //         denom: config.allowed_native,
        //         amount
        //     }],
        // })
       .add_attributes(vec![
           attr("action", "release"),
           attr("address", release_state.clone().address.to_string()),
           attr("gift_address", gift_address),
           attr("stage", release_state.stage.to_string()),
           attr("amount", amount),
       ])
    )
}
