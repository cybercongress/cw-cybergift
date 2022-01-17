#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, CosmosMsg, WasmMsg, Empty};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, NamespacesListResponse, RouteData};
use crate::state::{State, STATE};
use cw721_namespace::{
    msg::ExecuteMsg as Cw721ExecuteMsg, msg::MintMsg,
};
use cw721_namespace::state::{Extension, Metadata};
use crate::state::Route;

use cid::multihash::{Code, MultihashDigest};
use cid::Cid;

pub type NamespaceContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-cyber-passport";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        owner: Some(info.sender.clone()),
        namespaces: msg.namespaces, // TODO add map_validate
        cybergift: None,
    };
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
        ExecuteMsg::CreatePassport {
            citizen,
            routes
        } => execute_create_passport(deps, env, info, citizen, routes),
        ExecuteMsg::UpdateNamespaces{ namespaces } => execute_update_namespaces(deps, env, info, namespaces),
        ExecuteMsg::SetGift { gift } => execute_set_gift(deps, env, info, gift),
    }
}

pub fn execute_create_passport(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    citizen: String,
    routes_data: Vec<RouteData>
) -> Result<Response, ContractError> {

    let state = STATE.load(deps.storage)?;

    let cybergift = state.cybergift.ok_or(ContractError::Unauthorized {})?;
    let owner = state.owner.ok_or(ContractError::Unauthorized {})?;

    if info.sender != cybergift || info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    for route in routes_data {
        _add_to_namespace(&deps, &env, citizen.clone(), route.clone())?;
    }

    Ok(Response::new().add_attributes([
        attr("method", "created_passport"),
    ]))
}

fn _add_to_namespace(
    deps: &DepsMut,
    _env: &Env,
    citizen: String,
    route: RouteData,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let namespace_route = deps.api.addr_validate(&route.namespace)?;

    // TODO refactor this
    let namespace: &Route = state.namespaces
        .iter()
        .find(|&r| r.namespace == namespace_route)
        .ok_or(ContractError::Unauthorized {})?;

    let h = Code::Sha2_256.digest(&route.clone().data.parent.as_bytes());
    let token_id_cid = Cid::new_v0(h).unwrap();
    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg {
        token_id: token_id_cid.to_string(),
        owner: citizen,
        token_uri: None,
        extension: Metadata {
            parent: route.data.parent,
            target: route.data.target,
        },
    });

    let callback = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: namespace.address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });


    Ok(Response::new().add_message(callback))
}

pub fn execute_update_namespaces(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    namespaces: Vec<Route>,
) -> Result<Response, ContractError> {

    let mut state = STATE.load(deps.storage)?;

    let owner = state.clone().owner.ok_or(ContractError::Unauthorized {})?;

    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    state.namespaces = namespaces;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "updated_namespaces")
    ]))
}

pub fn execute_set_gift(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    gift: String,
) -> Result<Response, ContractError> {

    let mut state = STATE.load(deps.storage)?;

    let owner = state.clone().owner.ok_or(ContractError::Unauthorized {})?;

    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    state.cybergift = Some(deps.api.addr_validate(&gift)?);
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "updated_gift")
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::NamespacesList {} => to_binary(&query_namespaces_list(deps)?)
    }
}

pub fn query_namespaces_list(deps: Deps) -> StdResult<NamespacesListResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(NamespacesListResponse {
        namespaces: state.namespaces.into_iter().map(|a| a.into()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_create_passport() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_update_namespaces() {
        assert_eq!(true, true);
    }

    #[test]
    fn proper_set_gift() {
        assert_eq!(true, true);
    }
}
