#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Decimal, Deps, DepsMut, Env, StdResult, Uint64, MessageInfo, Empty};
use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, RELEASES_STATS, State, STATE};
use crate::execute::{execute_claim, execute_execute, execute_register_merkle_root, execute_release, execute_update_coefficients, execute_update_owner, execute_update_target, execute_update_treasury};
use crate::query::{query_all_release_stage_states, query_claim, query_config, query_is_claimed, query_merkle_root, query_release_stage_state, query_release_state, query_state};
use cyber_std::CyberMsgWrapper;
use semver::Version;

type Response = cosmwasm_std::Response<CyberMsgWrapper>;

// Version info, for migration info
const CONTRACT_NAME: &str = "cyber-gift";
const CONTRACT_VERSION: &str = "3.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
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
    for i in 0..100 {
        RELEASES_STATS.save(deps.storage, i as u8, &(0 as u32))?;
    }

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
        ExecuteMsg::Execute { msgs } => execute_execute(deps, env, info, msgs),
        ExecuteMsg::UpdateOwner { new_owner } => execute_update_owner(deps, env, info, new_owner),
        ExecuteMsg::UpdateTreasuryAddr { new_treasury_addr: new_treasury } => {
            execute_update_treasury(deps, env, info, new_treasury)
        }
        ExecuteMsg::UpdateTarget { new_target } => {
            execute_update_target(deps, env, info, new_target)
        }
        ExecuteMsg::UpdateCoefficients { new_coefficient_up, new_coefficient_down } => {
            execute_update_coefficients(deps, env, info, new_coefficient_up, new_coefficient_down)
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
        QueryMsg::AllReleaseStageStates {} => to_binary(&query_all_release_stage_states(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: Empty,
) -> Result<Response, ContractError> {
    let stored = get_contract_version(deps.storage)?;
    if stored.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: stored.contract,
        });
    }

    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version > version {
        return Err(ContractError::CannotMigrateVersion {
            previous_version: stored.version,
        });
    }

    if storage_version < version {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}
