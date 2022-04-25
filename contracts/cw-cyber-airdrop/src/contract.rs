#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{has_coins, to_binary, Binary, Coin, Decimal, Deps, DepsMut, Env, Response, StdResult, Uint64, MessageInfo};
use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, State, STATE};
use crate::execute::{execute_claim, execute_register_merkle_root, execute_release, execute_update_owner, execute_update_passport, execute_update_target};
use crate::query::{query_claim, query_config, query_is_claimed, query_merkle_root, query_release_stage_state, query_release_state, query_state};
use cw1_subkeys::msg::{ExecuteMsg as Cw1ExecuteMsg};

// Version info, for migration info
const CONTRACT_NAME: &str = "crates.io:cw-cyber-gift";
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

    let config = Config {
        owner: Some(owner),
        passport_addr: deps.api.addr_validate(&msg.passport)?,
        treasury_addr: deps.api.addr_validate(&msg.treasury)?,
        target_claim: msg.target_claim,
        allowed_native: msg.allowed_native,
        initial_balance: msg.initial_balance,
        coefficient_up: msg.coefficient_up,
        coefficient_down: msg.coefficient_down,
    };

    let state = State {
        current_balance: msg.initial_balance,
        coefficient: Decimal::from_ratio(msg.coefficient, 1u128),
        claims: Uint64::zero(),
        releases: Uint64::zero()
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;

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
            nickname,
            gift_claiming_address,
            gift_amount,
            proof,
        } => execute_claim(deps, env, info, nickname, gift_claiming_address, gift_amount, proof),
        ExecuteMsg::Release { gift_address } => execute_release(deps, env, info, gift_address),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::MerkleRoot {} => to_binary(&query_merkle_root(deps)?),
        QueryMsg::IsClaimed { address } => to_binary(&query_is_claimed(deps, address)?),
        QueryMsg::Claim { address } => to_binary(&query_claim(deps, address)?),
        QueryMsg::ReleaseState { address } => to_binary(&query_release_state(deps, address)?),
        QueryMsg::ReleaseStageState { stage } => to_binary(&query_release_stage_state(deps, stage)?),
    }
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
