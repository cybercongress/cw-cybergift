#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, StdResult, MessageInfo, Reply, Api, Addr, Empty};
use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use crate::execute::{CYBERLINK_ID_MSG, execute_cyberlink, execute_update_admins, execute_update_executors};
use crate::query::query_config;

use cyber_std::CyberMsgWrapper;
use semver::Version;

type Response = cosmwasm_std::Response<CyberMsgWrapper>;

const CONTRACT_NAME: &str = "cyber-subgraph";
const CONTRACT_VERSION: &str = "1.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admins: map_validate(deps.api, &msg.admins)?,
        executors: map_validate(deps.api, &msg.executers)?,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn map_validate(api: &dyn Api, admins: &[String]) -> StdResult<Vec<Addr>> {
    admins.iter().map(|addr| api.addr_validate(addr)).collect()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmins { new_admins } => execute_update_admins(deps, env, info, new_admins),
        ExecuteMsg::UpdateExecutors { new_executors } => execute_update_executors(deps, env, info, new_executors),
        ExecuteMsg::Cyberlink { links } => execute_cyberlink(deps, env, info, links),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != CYBERLINK_ID_MSG {
        return Err(ContractError::UnknownReplyId { id: reply.id });
    }
    Ok(Response::new())
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
