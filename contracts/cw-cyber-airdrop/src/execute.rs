#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, has_coins, to_binary, BankMsg, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, Uint64,
};
use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::helpers;
use crate::helpers::{update_coefficient, verify_cosmos, verify_merkle_proof};
use crate::msg::{
    ClaimMsg, ClaimerType, ConfigResponse, ExecuteMsg, InstantiateMsg, IsClaimedResponse,
    MerkleRootResponse, MigrateMsg, QueryMsg, ReleaseStateResponse,
};
use crate::state::{Config, ReleaseState, CLAIM, CONFIG, MERKLE_ROOT, RELEASE};
use cw_utils::{Duration, Expiration, Expiration::Never, DAY};
use std::ops::{Add, Mul, Sub};

// Version info, for migration info
const CONTRACT_NAME: &str = "crates.io:cw-cyber-gift";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASE_STAGES: u64 = 9;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .map_or(Ok(info.sender), |o| deps.api.addr_validate(&o))?;

    if !has_coins(
        &info.funds,
        &Coin {
            denom: msg.allowed_native.clone(),
            amount: msg.initial_balance,
        },
    ) {
        return Err(ContractError::InvalidInput {});
    }

    let config = Config {
        owner: Some(owner),
        passport_addr: deps.api.addr_validate(&msg.passport)?,
        target_claim: msg.target_claim,
        allowed_native: msg.allowed_native,
        current_balance: msg.initial_balance,
        initial_balance: msg.initial_balance,
        coefficient_up: msg.coefficient_up,
        coefficient_down: msg.coefficient_down,
        coefficient: Decimal::from_ratio(msg.coefficient, 1u128),
        claims: Uint64::zero(),
        releases: Uint64::zero(),
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { new_owner } => execute_update_owner(deps, env, info, new_owner),
        ExecuteMsg::UpdatePassportAddr { new_passport_addr: new_passport } => {
            execute_update_passport(deps, env, info, new_passport)
        }
        ExecuteMsg::UpdateTarget { new_target } => {
            execute_update_target(deps, env, info, new_target)
        }
        ExecuteMsg::RegisterMerkleRoot { merkle_root } => {
            execute_register_merkle_root(deps, env, info, merkle_root)
        }
        ExecuteMsg::Claim {
            claim_msg,
            signature,
            claim_amount,
            proof,
        } => execute_claim(deps, env, info, claim_msg, signature, claim_amount, proof),
        ExecuteMsg::Release {} => execute_release(deps, env, info),
    }
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

pub fn execute_claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    claim_msg: ClaimMsg,
    signature: Binary,
    amount: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {
    let claimed = CLAIM.may_load(deps.storage, claim_msg.target_address.clone())?;
    if claimed.is_some() {
        return Err(ContractError::Claimed {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    let claim_amount = amount * config.coefficient;

    // TODO: delete after debug
    println!("{:?}", "execute_claim");
    println!("{:?}", config.coefficient);
    println!("{:?}", claim_amount.to_string());

    if config.current_balance < claim_amount {
        return Err(ContractError::GiftIsOver {});
    }

    is_eligible(deps.as_ref(), &claim_msg, signature)?;

    verify_merkle_proof(
        &deps,
        &info,
        claim_msg.clone().gift_claiming_address,
        amount,
        proof,
    )?;

    CLAIM.save(deps.storage, claim_msg.target_address.clone(), &true)?;

    update_coefficient(deps.storage, claim_amount, &mut config)?;

    let release_state = ReleaseState {
        balance_claim: claim_amount.sub(Uint128::new(100000)),
        stage: Uint64::zero(),
        stage_expiration: Expiration::default(),
    };

    RELEASE.save(
        deps.storage,
        claim_msg.target_address.clone(),
        &release_state,
    )?;

    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.claims = cfg.claims.add(Uint64::new(1));
        Ok(cfg)
    })?;

    let res = Response::new()
        .add_message(BankMsg::Send {
            to_address: claim_msg.clone().target_address,
            amount: vec![Coin {
                denom: config.allowed_native,
                amount: Uint128::new(100000),
            }],
        })
        .add_attributes(vec![
            attr("action", "claim"),
            attr("original", claim_msg.clone().gift_claiming_address),
            attr(
                "type",
                claim_msg.clone().gift_claiming_address_type.to_string(),
            ),
            attr("target", claim_msg.clone().target_address),
            attr("amount", claim_amount),
        ]);
    Ok(res)
}

pub fn execute_release(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let claimed = CLAIM.may_load(deps.storage, info.clone().sender.into_string())?;
    if claimed.is_none() {
        return Err(ContractError::NotClaimed {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    if config.claims < config.target_claim {
        return Err(ContractError::NotActivated {});
    }

    let mut release_state = RELEASE.load(deps.storage, info.clone().sender.into_string())?;

    let amount: Uint128;
    // let expiration:Expiration;
    // let stage:u64;

    if release_state.balance_claim.is_zero() {
        return Err(ContractError::GiftReleased {});
    }

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
        info.clone().sender.to_string(),
        &release_state,
    )?;

    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.releases = cfg.releases.add(Uint64::new(1));
        Ok(cfg)
    })?;

    let res = Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: config.allowed_native,
                amount,
            }],
        })
        .add_attributes(vec![
            attr("action", "release"),
            attr("address", info.sender),
            attr("stage", release_state.stage.to_string()),
            attr("amount", amount),
        ]);
    Ok(res)
}

fn is_eligible(deps: Deps, claim_msg: &ClaimMsg, signature: Binary) -> Result<bool, ContractError> {
    match claim_msg.gift_claiming_address_type {
        ClaimerType::Ethereum => helpers::verify_ethereum(deps, &claim_msg, signature),
        ClaimerType::Cosmos => verify_cosmos(deps, &claim_msg, signature),
        _ => Err(ContractError::IsNotEligible {
            msg: "address prefix not allowed".to_string(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::MerkleRoot {} => to_binary(&query_merkle_root(deps)?),
        QueryMsg::IsClaimed { address } => to_binary(&query_is_claimed(deps, address)?),
        QueryMsg::ReleaseState { address } => to_binary(&query_release_state(deps, address)?),
    }
}

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
    let is_claimed = CLAIM.may_load(deps.storage, address)?.unwrap_or(false);
    let resp = IsClaimedResponse { is_claimed };

    Ok(resp)
}

pub fn query_release_state(deps: Deps, address: String) -> StdResult<ReleaseStateResponse> {
    let release_state = RELEASE
        .may_load(deps.storage, address)?
        .unwrap_or(ReleaseState {
            balance_claim: Default::default(),
            stage: Uint64::zero(),
            stage_expiration: Default::default(),
        });
    let resp = ReleaseStateResponse {
        balance_claim: release_state.balance_claim,
        stage: release_state.stage,
        stage_expiration: release_state.stage_expiration,
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
