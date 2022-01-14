#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use cw2::{get_contract_version, set_contract_version};
pub use cw721_base::{MintMsg, MinterResponse};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::NamespaceContract;

use crate::execute::{execute_mint, execute_burn, execute_set_minter, execute_set_owner, execute_set_fee, execute_transfer_nft, execute_send_nft};

use crate::query::{query_config};
use crate::state::{Config, CONFIG};
use crate::{error::ContractError};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let owner = msg.clone()
        .owner
        .map_or(Ok(info.clone().sender), |o| deps.api.addr_validate(&o))?;

    let config = Config {
        fee: msg.clone().fee,
        owner: Some(owner)
    };

    CONFIG.save(deps.storage, &config)?;

    NamespaceContract::default().instantiate(deps, env, info, msg.into())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetMinter { minter } => execute_set_minter(deps, env, info, minter),
        ExecuteMsg::SetOwner { owner } => execute_set_owner(deps, env, info, owner),
        ExecuteMsg::SetFee { fee } => execute_set_fee(deps, env, info, fee),
        // Overwrite CW721 methods
        ExecuteMsg::TransferNft { recipient, token_id} => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft { contract, token_id, msg} => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::Mint(mint_msg) => execute_mint(deps, env, info, mint_msg),
        // CW721 methods
        _ => NamespaceContract::default()
            .execute(deps, env, info, msg.into())
            .map_err(|err| err.into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        // CW721 methods
        _ => NamespaceContract::default().query(deps, env, msg.into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg<Config>,
) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg { version, config } => try_migrate(deps, version, config),
    }
}

fn try_migrate(
    deps: DepsMut,
    version: String,
    config: Option<Config>,
) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;
    set_contract_version(deps.storage, contract_version.contract, version)?;

    if config.is_some() {
        CONFIG.save(deps.storage, &config.unwrap())?
    }

    Ok(Response::new()
        .add_attribute("method", "try_migrate")
        .add_attribute("version", contract_version.version))
}
