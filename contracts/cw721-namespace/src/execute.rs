use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Coin, StdResult, attr, Binary};
use cw721::Cw721Execute;
use cw721_base::{MintMsg};
use crate::state::{Extension, SpaceContract};

use crate::error::ContractError;
use crate::state::{Config, CONFIG};

pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let cw721_contract = SpaceContract::default();

    let response = cw721_contract.transfer_nft(deps, env, info, recipient, token_id)?;
    Ok(response)
}

pub fn execute_send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary
) -> Result<Response, ContractError> {
    let cw721_contract = SpaceContract::default();

    let response = cw721_contract.send_nft(deps, env, info, contract, token_id, msg)?;
    Ok(response)
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_msg: MintMsg<Extension>,
) -> Result<Response, ContractError> {
    let cw721_contract = SpaceContract::default();

    let response = cw721_contract.mint(deps, env, info, mint_msg)?;
    Ok(response)
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let cw721_contract = SpaceContract::default();

    let response = cw721_contract.burn(deps, env, info, token_id)?;
    Ok(response)
}

pub fn execute_set_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_minter: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let owner = config.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_minter = deps.api.addr_validate(&new_minter)?;
    let cw721_contract = SpaceContract::default();
    cw721_contract.minter.save(deps.storage, &new_minter)?;

    Ok(Response::new().add_attribute("action", "set_minter"))
}

pub fn execute_set_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let owner = config.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let config = Config {
        owner: deps.api.addr_validate(&new_owner)?
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_owner")
    ]))
}
