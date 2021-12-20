use std::convert::TryInto;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg, has_coins, Coin, BankMsg};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ExecuteMsg;
use sha2::Digest;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, IsClaimedResponse, MerkleRootResponse, MigrateMsg,
    QueryMsg,
};
use crate::state::{Config, CLAIM, CONFIG, MERKLE_ROOT};

// Version info, for migration info
const CONTRACT_NAME: &str = "crates.io:cw-cyber-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    if !has_coins(&info.funds, &Coin{ denom: msg.allowed_native.clone(), amount: msg.initial_balance }){
        return Err(ContractError::InvalidInput {})
    }

    let config = Config {
        owner: Some(owner),
        allowed_native: msg.allowed_native,
        current_balance: msg.initial_balance,
        initial_balance: msg.initial_balance,
        coefficient_up: msg.coefficient_up,
        coefficient_down: msg.coefficient_down,
        coefficient: msg.coefficient,
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
        ExecuteMsg::UpdateConfig { new_owner } => execute_update_config(deps, env, info, new_owner),
        ExecuteMsg::RegisterMerkleRoot { merkle_root } => {
            execute_register_merkle_root(deps, env, info, merkle_root)
        }
        ExecuteMsg::Claim {
            amount,
            proof,
        } => execute_claim(deps, env, info, amount, proof),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: Option<String>,
) -> Result<Response, ContractError> {
    // authorize owner
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    // if owner some validated to addr, otherwise set to none
    let mut tmp_owner = None;
    if let Some(addr) = new_owner {
        tmp_owner = Some(deps.api.addr_validate(&addr)?)
    }

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.owner = tmp_owner;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_register_merkle_root(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    merkle_root: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // if owner set validate, otherwise unauthorized
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    // check merkle root length
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
    amount: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {
    // verify not claimed
    let claimed = CLAIM.may_load(deps.storage, &info.sender)?;
    if claimed.is_some() {
        return Err(ContractError::Claimed {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    // various checks
    // TODO: coefficient calculation
    let claim_amount = amount * config.coefficient;
    is_eligible(&config, claim_amount)?;

    let merkle_root = MERKLE_ROOT.load(deps.storage)?;

    let user_input = format!("{}{}", info.sender, amount);
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

    // Update claim index to the current stage
    CLAIM.save(deps.storage, &info.sender, &true)?;

    // Update coefficient
    let coefficient_up = config.coefficient_up;
    let coefficient_down = config.coefficient_down;
    let initial_balance = config.initial_balance;
    let current_balance = config.current_balance;

    let new_coefficient =
        coefficient_up + (coefficient_down - coefficient_up) * initial_balance / current_balance;

    config.coefficient = new_coefficient;
    config.current_balance = current_balance - amount;
    CONFIG.save(deps.storage, &config)?;

    let res = Response::new()
        .add_message(BankMsg::Send{
            to_address: info.sender.to_string(),
            amount: vec![Coin{denom: config.allowed_native,  amount: claim_amount}]
        })
        .add_attributes(vec![
            attr("action", "claim"),
            attr("address", info.sender),
            attr("amount", amount),
        ]);
    Ok(res)
}

fn is_eligible(cfg: &Config, claim_amount: Uint128) -> Result<(), ContractError> {
    if cfg.initial_balance - cfg.current_balance < claim_amount {
        return Err(ContractError::IsNotEligible {});
    }
    return Ok(());
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::MerkleRoot {} => to_binary(&query_merkle_root(deps)?),
        QueryMsg::IsClaimed { address } => to_binary(&query_is_claimed(deps, address)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner.map(|o| o.to_string()),
        allowed_native: cfg.allowed_native,
        current_balance: cfg.current_balance,
        initial_balance: cfg.initial_balance,
        coefficient_up: cfg.coefficient_up,
        coefficient_down: cfg.coefficient_down,
        coefficient: cfg.coefficient,
    })
}

pub fn query_merkle_root(deps: Deps) -> StdResult<MerkleRootResponse> {
    let merkle_root = MERKLE_ROOT.load(deps.storage)?;
    let resp = MerkleRootResponse { merkle_root };

    Ok(resp)
}

pub fn query_is_claimed(deps: Deps, address: String) -> StdResult<IsClaimedResponse> {
    let key = &deps.api.addr_validate(&address)?;
    let is_claimed = CLAIM.may_load(deps.storage, key)?.unwrap_or(false);
    let resp = IsClaimedResponse { is_claimed };

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
