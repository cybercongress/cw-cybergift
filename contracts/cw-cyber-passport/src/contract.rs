#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, StdResult, to_binary};
use cw2::{get_contract_version, set_contract_version};
use cyber_std::CyberMsgWrapper;
use semver::Version;

use crate::error::ContractError;
use crate::execute::{execute_burn, execute_create_passport, execute_mint, execute_proof_address, execute_remove_address, execute_send_nft, execute_set_active, execute_set_owner, execute_set_subgraphs, execute_transfer_nft, execute_update_avatar, execute_update_name, CYBERSPACE_ID_MSG, execute_set_address_label, execute_update_data, execute_update_particle, execute_execute};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::{query_active_passport, query_address_by_nickname, query_config, query_metadata_by_nickname, query_passport_signed, query_passport_by_nickname, query_portid};
use crate::state::{Config, CONFIG, PassportContract, PORTID};

type Response = cosmwasm_std::Response<CyberMsgWrapper>;

const CONTRACT_NAME: &str = "cyber-passport";
const CONTRACT_VERSION: &str = "1.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        owner: deps.api.addr_validate(&msg.clone().owner)?,
        name_subgraph: deps.api.addr_validate(&msg.clone().name_subgraph)?,
        avatar_subgraph: deps.api.addr_validate(&msg.clone().avatar_subgraph)?,
        proof_subgraph: deps.api.addr_validate(&msg.clone().proof_subgraph)?
    };

    CONFIG.save(deps.storage, &config)?;
    PORTID.save(deps.storage, &0u64)?;

    // override minter to contract itself
    msg.minter = env.clone().contract.address.into_string();
    let res = PassportContract::default().instantiate(deps.branch(), env, info, msg.into())?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(res)
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
        ExecuteMsg::CreatePassport {nickname, avatar} => execute_create_passport(deps, env, info, nickname, avatar),
        ExecuteMsg::UpdateName { old_nickname, new_nickname} => execute_update_name(deps, env, info, old_nickname, new_nickname),
        ExecuteMsg::UpdateAvatar { nickname, new_avatar} => execute_update_avatar(deps, env, info, nickname, new_avatar),
        ExecuteMsg::UpdateData { nickname, data} => execute_update_data(deps, env, info, nickname, data),
        ExecuteMsg::UpdateParticle { nickname, particle} => execute_update_particle(deps, env, info, nickname, particle),
        ExecuteMsg::ProofAddress {
            nickname,
            address,
            signature
        } => execute_proof_address(deps, env, info, nickname, address, signature),
        ExecuteMsg::RemoveAddress {nickname, address } => execute_remove_address(deps, env, info, nickname, address),
        ExecuteMsg::SetOwner { owner } => execute_set_owner(deps, env, info, owner),
        ExecuteMsg::SetActive { token_id } => execute_set_active(deps, env, info, token_id),
        ExecuteMsg::SetSubgraphs {
            name_subgraph,
            avatar_subgraph,
            proof_subgraph
        } => execute_set_subgraphs(deps, env, info, name_subgraph, avatar_subgraph, proof_subgraph),
        ExecuteMsg::SetAddressLabel {
            nickname,
            address,
            label
        } => execute_set_address_label(deps, env, info, nickname, address, label),
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
        QueryMsg::PassportSigned {nickname, address} => to_binary(&query_passport_signed(deps, nickname, address)?),
        QueryMsg::ActivePassport { address } => to_binary(&query_active_passport(deps, address)?),
        // CW721 methods
        _ => PassportContract::default().query(deps, env, msg.into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != CYBERSPACE_ID_MSG {
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
