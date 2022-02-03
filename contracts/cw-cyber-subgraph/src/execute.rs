use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, StdResult};

use crate::error::ContractError;
use crate::state::CONFIG;
use cyber_std::{create_cyberlink_msg, Link, CyberMsgWrapper};

type Response = cosmwasm_std::Response<CyberMsgWrapper>;


pub fn execute_update_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {});
    }

    let owner = deps.api.addr_validate(&new_owner)?;
    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.owner = owner;
        Ok(cfg)
    })?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_owner")]))
}

pub fn execute_update_executer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_executer: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {});
    }

    let executer = deps.api.addr_validate(&new_executer)?;
    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.executer = executer;
        Ok(cfg)
    })?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_executer")]))
}

pub fn execute_cyberlink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cyberlink: Vec<Link>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.executer {
        return Err(ContractError::Unauthorized {});
    }

    let msg = create_cyberlink_msg(env.contract.address.to_string(), cyberlink);
    Ok(Response::new().add_message(msg))
}

