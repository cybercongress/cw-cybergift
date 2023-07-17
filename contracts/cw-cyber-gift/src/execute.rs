use cosmwasm_std::{Addr, attr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Empty, Env, MessageInfo, StdResult, to_binary, Uint128, Uint64, WasmMsg};

use crate::error::ContractError;
use crate::helpers::{update_coefficient, verify_merkle_proof};
use crate::state::{ReleaseState, CLAIM, CONFIG, MERKLE_ROOT, RELEASE, ClaimState, STATE, RELEASES_STATS};
use cw_utils::{Expiration};
use std::ops::{Add, Mul, Sub};
use cw_cyber_passport::msg::{QueryMsg as PassportQueryMsg};
use crate::msg::{AddressResponse, SignatureResponse};
use cw1_subkeys::msg::{ExecuteMsg as Cw1ExecuteMsg};
use cyber_std::CyberMsgWrapper;
use crate::indexed_referral::{has_ref, ref_chains, REFERRALS, set_ref};

type Response = cosmwasm_std::Response<CyberMsgWrapper>;

const CLAIM_BOUNTY: u128 =  100000;
const COMMUNITY_POOL: &str = "alice";

pub fn execute_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<CyberMsgWrapper>>,
) -> Result<Response, ContractError> {
    let mut res = Response::new().add_attribute("action", "execute");

    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    res = res.add_messages(msgs);

    Ok(res)
}

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

pub fn execute_claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    nickname: String,
    mut gift_claiming_address: String,
    gift_amount: Uint128,
    proof: Vec<String>,
    referral: Option<String>,
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
        return Err(ContractError::IsNotProved {});
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

    if referral.is_some() {
        if deps.api.addr_validate(&referral.clone().unwrap())?.ne(&Addr::unchecked(res.address.clone())) {
            if has_ref(deps.storage, &Addr::unchecked(res.address.clone()))?.eq(&false) {
                set_ref(
                    deps.storage,
                    &Addr::unchecked(res.address.clone()),
                    &Addr::unchecked(referral.unwrap())
                )?;
            };
        }
    }

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
    _env: Env,
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

    let mut state_stage = Uint128::new(100u128).mul(Decimal::from_ratio(state.claims, config.target_claim));
    if state_stage.ge(&Uint128::new(100u128)) {
        state_stage = Uint128::new(100u128);
    }

    let mut release_state = RELEASE.load(deps.storage, gift_address.clone())?;

    if release_state.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if release_state.balance_claim.is_zero() {
        return Err(ContractError::GiftReleased {});
    }

    if release_state.stage.eq(&Uint64::new(state_stage.clone().u128() as u64)) {
        return Err(ContractError::StageReleased {});
    }

    if release_state.stage.gt(&Uint64::new(state_stage.clone().u128() as u64)) {
        return Err(ContractError::Unauthorized {})
    }

    let amount;
    if release_state.stage.is_zero() {
        amount = release_state
            .balance_claim
            .mul(Decimal::percent(state_stage.u128() as u64));
    } else {
        if state_stage.ne(&Uint128::new(100u128)) {
            amount = CLAIM.load(deps.storage, gift_address.clone())?
                .claim
                .sub(Uint128::from(CLAIM_BOUNTY))
                .mul(Decimal::from_ratio(state_stage.sub(Uint128::from(release_state.stage)), 100u128))
        } else {
            amount = release_state.balance_claim;
        }
    }

    for i in (release_state.stage.u64())..(state_stage.u128() as u64) {
        RELEASES_STATS.update(deps.storage, i as u8, |count| -> StdResult<_> {
            Ok(count.unwrap() + 1u32)
        })?;
    }

    release_state.stage = Uint64::from(state_stage.u128() as u64);
    release_state.balance_claim = release_state.balance_claim - amount;

    RELEASE.save(
        deps.storage,
        gift_address.clone(),
        &release_state,
    )?;

    STATE.update(deps.storage, |mut stt| -> StdResult<_> {
        stt.releases = stt.releases.add(Uint64::new(1));
        Ok(stt)
    })?;

    let mut messages: Vec<CosmosMsg> = vec![];
    let amount_gift = Decimal::percent(80u64).mul(amount);
    messages.push(
        CosmosMsg::from(BankMsg::Send {
            to_address: release_state.clone().address.into(),
            amount: vec![Coin {
                denom: config.clone().allowed_native,
                amount: amount_gift,
            }],
        })
    );

    if has_ref(deps.storage, &release_state.clone().address)?.eq(&true) {
        let chain = ref_chains(deps.storage, &release_state.clone().address, Some(4))?;
        let amount_referral = amount
            .sub(amount_gift).
            mul(Decimal::from_ratio(Uint128::from(1u64), Uint128::from(chain.len() as u64)));
        for addr in chain.into_iter() {
            messages.push(CosmosMsg::from(BankMsg::Send {
                to_address: addr.into_string(),
                amount: vec![Coin {
                    denom: config.clone().allowed_native,
                    amount: amount_referral,
                }]
            }))
        }
    } else {
        let amount_pool = amount.sub(amount_gift);
        messages.push(
            CosmosMsg::from(BankMsg::Send {
                to_address: String::from(COMMUNITY_POOL),
                amount: vec![Coin {
                    denom: config.clone().allowed_native,
                    amount: amount_pool,
                }],
            })
        );
    }

    // HOW design affected with passport mapped to address (what if address will change)
    Ok(Response::new()
       .add_message(WasmMsg::Execute {
           contract_addr: config.treasury_addr.to_string(),
           msg: to_binary(&Cw1ExecuteMsg::Execute::<Empty> {
               msgs: messages
           })?,
           funds: vec![]
       })
       .add_attributes(vec![
           attr("action", "release"),
           attr("address", release_state.clone().address.to_string()),
           attr("gift_address", gift_address),
           attr("stage", release_state.stage.to_string()),
           attr("amount", amount),
       ])
    )
}
