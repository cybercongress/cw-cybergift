use cosmwasm_std::{attr, BankMsg, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, StdResult, to_binary, Uint128, Uint64, WasmMsg};

use crate::error::ContractError;
use crate::helpers::{update_coefficient, verify_merkle_proof};
use crate::state::{CLAIM, CONFIG, MERKLE_ROOT, ClaimState, STATE};
use std::ops::{Add};
use cw_cyber_passport::msg::{QueryMsg as PassportQueryMsg};
use crate::msg::{AddressResponse, SignatureResponse};
use cw1_subkeys::msg::{ExecuteMsg as Cw1ExecuteMsg};
use cyber_std::CyberMsgWrapper;

type Response = cosmwasm_std::Response<CyberMsgWrapper>;

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
                        to_address: res.clone().address,
                        amount: vec![Coin {
                            denom: config.allowed_native,
                            amount: claim_amount,
                        }],
                    }).into()
                ]})?,
            funds: vec![]
        })
        .add_attributes(vec![
            attr("action", "release"),
            attr("address", res.clone().address),
            attr("gift_address", gift_claiming_address),
            attr("amount", claim_amount),
        ])
    )
}
