#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult, to_binary};
use cyber_std::CyberMsgWrapper;

use crate::error::ContractError;
use crate::execute::{execute_burn, execute_create_passport, execute_mint, execute_proof_address, execute_remove_address, execute_send_nft, execute_set_active, execute_set_minter, execute_set_owner, execute_set_subspaces, execute_transfer_nft, execute_update_avatar, execute_update_name, try_migrate, CYBERLINK_ID_MSG};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{query_active_passport, query_address_by_nickname, query_config, query_metadata_by_nickname, query_passort_signed, query_passport_by_nickname, query_portid};
use crate::state::{Config, CONFIG, PassportContract, PORTID};

type Response = cosmwasm_std::Response<CyberMsgWrapper>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        owner: deps.api.addr_validate(&msg.clone().owner)?,
        name_subspace: deps.api.addr_validate(&msg.clone().name_subspace)?,
        avatar_subspace: deps.api.addr_validate(&msg.clone().avatar_subspace)?,
        proof_subspace: deps.api.addr_validate(&msg.clone().proof_subspace)?
    };

    CONFIG.save(deps.storage, &config)?;
    PORTID.save(deps.storage, &0u64)?;

    PassportContract::default().instantiate(deps, env, info, msg.into())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePassport {nickname, avatar} => execute_create_passport(deps, env, info, nickname, avatar),
        ExecuteMsg::UpdateName { old_nickname, new_nickname} => execute_update_name(deps, env, info, old_nickname, new_nickname),
        ExecuteMsg::UpdateAvatar { nickname, new_avatar} => execute_update_avatar(deps, env, info, nickname, new_avatar),
        ExecuteMsg::ProofAddress { nickname, address, signature } => execute_proof_address(deps, env, info, nickname, address, signature),
        ExecuteMsg::RemoveAddress {nickname, address } => execute_remove_address(deps, env, info, nickname, address),
        ExecuteMsg::SetMinter { minter } => execute_set_minter(deps, env, info, minter),
        ExecuteMsg::SetOwner { owner } => execute_set_owner(deps, env, info, owner),
        ExecuteMsg::SetActive { token_id } => execute_set_active(deps, env, info, token_id),
        ExecuteMsg::SetSubspaces {
            name_subspace,
            avatar_subspace,
            proof_subspace
        } => execute_set_subspaces(deps, env, info, name_subspace, avatar_subspace, proof_subspace),
        // Overwrite CW721 methods
        ExecuteMsg::TransferNft { recipient, token_id} => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft { contract, token_id, msg} => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::Mint(mint_msg) => execute_mint(deps, env, info, mint_msg),
        // CW721 methods
        _ => PassportContract::default()
            .execute(deps, env, info, msg.into())
            .map_err(|err| err.into()),

    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Portid {} => to_binary(&query_portid(deps)?),
        QueryMsg::AddressByNickname {nickname} => to_binary(&query_address_by_nickname(deps, nickname)?),
        QueryMsg::PassportByNickname {nickname} => to_binary(&query_passport_by_nickname(deps, nickname)?),
        QueryMsg::MetadataByNickname {nickname} => to_binary(&query_metadata_by_nickname(deps, nickname)?),
        QueryMsg::PassportSigned {nickname, address} => to_binary(&query_passort_signed(deps, nickname, address)?),
        QueryMsg::ActivePassport { address } => to_binary(&query_active_passport(deps, address)?),
        // CW721 methods
        _ => PassportContract::default().query(deps, env, msg.into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != CYBERLINK_ID_MSG {
        return Err(ContractError::UnknownReplyId { id: reply.id });
    }
    // let res = match reply.result {
    //     ContractResult::Ok(_) => Response::new(),
    //     ContractResult::Err(_) => Response::new()
    // };
    Ok(Response::new())
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
