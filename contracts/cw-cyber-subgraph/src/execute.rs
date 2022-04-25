use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, StdResult, SubMsg};

use crate::error::ContractError;
use crate::state::CONFIG;
use cyber_std::{create_cyberlink_msg, Link, CyberMsgWrapper};
use crate::contract::map_validate;

type Response = cosmwasm_std::Response<CyberMsgWrapper>;
pub const CYBERLINK_ID_MSG: u64 = 42;

pub fn execute_update_admins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admins: Vec<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if !cfg.can_modify(info.sender.as_ref()) {
        return Err(ContractError::Unauthorized {});
    }

    let admins = map_validate(deps.api, &new_admins)?;
    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.admins = admins;
        Ok(cfg)
    })?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_admins")]))
}

pub fn execute_update_executors(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_executors: Vec<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if !cfg.can_modify(info.sender.as_ref()) {
        return Err(ContractError::Unauthorized {});
    }

    let executors = map_validate(deps.api, &new_executors)?;
    CONFIG.update(deps.storage, |mut cfg| -> StdResult<_> {
        cfg.executors = executors;
        Ok(cfg)
    })?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_executors")]))
}

pub fn execute_cyberlink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cyberlink: Vec<Link>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if !cfg.can_execute(info.sender.as_ref()) {
        return Err(ContractError::Unauthorized {});
    }

    let msg = create_cyberlink_msg(env.contract.address.to_string(), cyberlink);
    Ok(Response::new().add_submessage(SubMsg::reply_on_error(msg, CYBERLINK_ID_MSG)))
}

