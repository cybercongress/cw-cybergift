use std::ops::{Add, Mul};
use std::str::FromStr;

use cid::{Cid, Version};
use cid::multihash::{Code, MultihashDigest};
use cosmwasm_std::{attr, Binary, DepsMut, Env, MessageInfo, Uint128};
use cw2::{get_contract_version, set_contract_version};
use cw721::{Cw721Execute, Cw721Query};
use cw721_base::MintMsg;
use cw_utils::must_pay;

use cyber_std::{
    create_cyberlink_msg, CyberMsgWrapper, Link,
};

use crate::error::ContractError;
use crate::helpers::{proof_address_cosmos, proof_address_ethereum, decode_address};
use crate::state::{ACTIVE, AddressPortID, Extension, NICKNAMES, PassportContract, PassportMetadata, PORTID};
use crate::state::{Config, CONFIG};

type Response = cosmwasm_std::Response<CyberMsgWrapper>;

// TODO discuss and make this configurable in config
const CONSTITUTION: &str = "QmRX8qYgeZoYM3M5zzQaWEpVFdpin6FvVXvp6RPQK3oufV";

pub fn execute_create_passport(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    avatar: String,
) -> Result<Response, ContractError> {
    if NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameAlreadyExists {});
    }

    let cw721_contract = PassportContract::default();

    let nickname_length = nickname.clone().len();
    if nickname_length < 9 {
        let must_pay = must_pay(&info, "boot").unwrap_or_default();
        let to_pay = Uint128::new(1_000_000_000).checked_pow(9-nickname_length as u32).unwrap();
        if must_pay != to_pay {
            return Err(ContractError::WrongAmountForName {});
        }
    }

    let nick_particle = _prepare_particle(nickname.clone());
    let avatar_particle = _check_particle(avatar.clone())?;
    let address_particle = _prepare_particle(info.clone().sender.into());

    // nickname <-> address <-> avatar
    // NOTE if one of cyberlinks from the set is exist it will revert whole message and other links
    // TODO split in separate submessages?
    let links= vec![
        Link{
            from: address_particle.clone().into(),
            to: nick_particle.clone().into(),
        },
        Link{
            from: address_particle.clone().into(),
            to: avatar_particle.clone().into()
        },
        Link{
            from: nick_particle.clone().into(),
            to: address_particle.clone().into()
        },
        Link{
            from: avatar_particle.clone().into(),
            to: address_particle.clone().into()
        }
    ];

    let contract = env.clone().contract.address;
    let cyberlink_msg = create_cyberlink_msg(contract.into(), links);

    let last_portid = PORTID.load(deps.storage).unwrap().add(1);
    let mint_msg = MintMsg {
        token_id: last_portid.to_string(),
        owner: info.clone().sender.into(),
        token_uri: None,
        extension: PassportMetadata {
            addresses: None,
            avatar,
            nickname: nickname.clone()
        },
    };

    PORTID.save(deps.storage, &last_portid)?;

    NICKNAMES.save(
        deps.storage,
        &nickname,
        &AddressPortID{
            address: info.clone().sender,
            portid: last_portid.to_string()
        }
    )?;

    if !ACTIVE.has(deps.storage, &info.clone().sender) {
        ACTIVE.save(deps.storage, &info.clone().sender, &last_portid.to_string())?;
    }

    // // let sub = SubMsg::reply_on_success(msg, CYBERLINK_ID).with_gas_limit(128_000);
    // let response = cw721_contract.mint(deps, env, info, mint_msg)?;
    // // https://github.com/cybercongress/go-cyber/blob/main/x/graph/types/errors.go#L8
    // // TODO run transaction if cyberlink already exist (graph error)
    // // TODO fail transaction in all other error cases (graph errors) or ignore this case
    // // Ok(response.add_submessage(sub))

    let internal_info = MessageInfo {
        sender: env.clone().contract.address,
        funds: info.funds,
    };

    let response = cw721_contract.mint(deps, env, internal_info, mint_msg)?;

    Ok(response.add_message(cyberlink_msg))
}

fn _prepare_particle(input: String) -> Cid {
    let h = Code::Sha2_256.digest(&input.as_bytes());
    let particle = Cid::new_v0(h).unwrap();
    particle
}

fn _check_particle(input: String) -> Result<Cid, ContractError> {
    let particle:Cid;
    let try_particle = Cid::from_str(&input.clone());
    if try_particle.is_ok() {
        particle = try_particle.unwrap();
        if particle.version() != Version::V0 {
            return Err(ContractError::InvalidParticleVersion {});
        }
    } else {
        return Err(ContractError::InvalidParticle {});
    }
    Ok(particle)
}

pub fn execute_update_name(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    old_name: String,
    new_name: String
) -> Result<Response, ContractError> {
    if NICKNAMES.has(deps.storage, &new_name.clone()) {
        return Err(ContractError::NicknameAlreadyExists {});
    }

    if !NICKNAMES.has(deps.storage, &old_name.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };

    let nickname_length = new_name.clone().len();
    if nickname_length < 8 {
        let must_pay = must_pay(&info, "boot").unwrap_or_default();
        let mul = 10u64.checked_pow(8-nickname_length as u32).unwrap();
        let to_pay = Uint128::new(1_000_000).mul(Uint128::from(mul));
        if must_pay != to_pay {
            return Err(ContractError::WrongAmountForName {});
        }
    }

    let cw721_contract = PassportContract::default();
    let address_portid = NICKNAMES.load(deps.storage, &old_name.clone())?;
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env.clone(), address_portid.clone().portid, false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }

    cw721_contract
        .tokens
        .update(deps.storage, &address_portid.clone().portid, |token| match token {
        Some(mut token_info) => {
            token_info.extension.nickname = new_name.clone();
            Ok(token_info)
        }
        None => return Err(ContractError::TokenNotFound {}),
    })?;

    NICKNAMES.remove(deps.storage, old_name.as_str());
    NICKNAMES.save(
        deps.storage,
        &new_name.clone(),
        &AddressPortID{
            address: info.clone().sender,
            portid: address_portid.portid
        }
    )?;

    let nick_particle = _prepare_particle(new_name.clone());
    let address_particle = _prepare_particle(info.clone().sender.into());

    // nickname <-> address
    // NOTE if one of cyberlinks from set is exist it will revert whole message and other links
    // TODO split in separate submessages?
    let links= vec![
        Link{
            from: address_particle.clone().into(),
            to: nick_particle.clone().into(),
        },
        Link{
            from: nick_particle.clone().into(),
            to: address_particle.clone().into()
        }
    ];

    let contract = env.clone().contract.address;
    let cyberlink_msg = create_cyberlink_msg(contract.into(), links);

    Ok(Response::new()
        .add_message(cyberlink_msg)
        .add_attributes(vec![
            attr("action", "update_nickname"),
            attr("old_name", old_name),
            attr("new_name", new_name),
    ]))
}

pub fn execute_update_avatar(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    new_avatar: String
) -> Result<Response, ContractError> {
    if !NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };

    let cw721_contract = PassportContract::default();
    let address_portid = NICKNAMES.load(deps.storage, &nickname)?;
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env.clone(), address_portid.clone().portid, false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }

    cw721_contract
        .tokens
        .update(deps.storage, &address_portid.clone().portid, |token| match token {
            Some(mut token_info) => {
                token_info.extension.avatar = new_avatar.clone();
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    let avatar_particle = _check_particle(new_avatar.clone())?;
    let address_particle = _prepare_particle(info.clone().sender.into());

    // nickname <-> address
    // NOTE if one of cyberlinks from set is exist it will revert whole message and other links
    // TODO split in separate submessages?
    let links= vec![
        Link{
            from: address_particle.clone().into(),
            to: avatar_particle.clone().into(),
        },
        Link{
            from: avatar_particle.clone().into(),
            to: address_particle.clone().into()
        }
    ];

    let contract = env.clone().contract.address;
    let cyberlink_msg = create_cyberlink_msg(contract.into(), links);

    Ok(Response::new()
        .add_message(cyberlink_msg)
        .add_attributes(vec![
            attr("action", "update_avatar"),
            attr("nickname", nickname),
            attr("new_avatar", new_avatar),
    ]))
}

pub fn execute_proof_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    address: String,
    signature: Binary
) -> Result<Response, ContractError> {
    if !NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };

    let cw721_contract = PassportContract::default();
    let address_portid = NICKNAMES.load(deps.storage, &nickname)?;
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env.clone(), address_portid.clone().portid, false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }

    let proof_res:bool;
    if decode_address(&address).is_err() {
        proof_res = proof_address_cosmos(deps.as_ref(), address.clone(), info.sender.to_string(), CONSTITUTION.into(), signature)
            .map_err(|err| ContractError::IsNotEligible {
                msg: err.to_string(),
        })?;
    } else {
        proof_res = proof_address_ethereum(deps.as_ref(), address.clone(), info.sender.to_string(),CONSTITUTION.into(), signature)
            .map_err(|err| ContractError::IsNotEligible {
                msg: err.to_string(),
        })?;
    }


    if proof_res {
        let mut token_info = cw721_contract.tokens.load(deps.storage, &address_portid.clone().portid)?;
        if token_info.extension.addresses.is_some() {
            let mut addresses = token_info.extension.addresses.unwrap();
            if addresses.len() > 7 {
                return Err(ContractError::IsNotEligible {
                    msg: "Too many addresses".to_string(),
                });
            }
            if addresses.iter().position(|x| *x == address.clone()).is_some() {
                return Err(ContractError::IsNotEligible {
                    msg: "Address already exist".to_string(),
                });
            }
            addresses.push(address.clone());
            token_info.extension.addresses = Some(addresses);
        } else {
            token_info.extension.addresses = Some(vec![address.clone()]);
        };
        cw721_contract.tokens.save(deps.storage, &address_portid.clone().portid, &token_info)?;
    } else {
        return Err(ContractError::IsNotEligible {
            msg: "Address is not eligible".to_string(),
        });
    }

    let proved_address_particle = _prepare_particle(address.clone());
    let address_particle = _prepare_particle(info.clone().sender.into());

    // prooved_address <-> address
    // NOTE if one of cyberlinks from set is exist it will revert whole message and other links
    // TODO split in separate submessages?
    let links= vec![
        Link{
            from: address_particle.clone().into(),
            to: proved_address_particle.clone().into(),
        },
        Link{
            from: proved_address_particle.clone().into(),
            to: address_particle.clone().into()
        }
    ];

    let contract = env.clone().contract.address;
    let cyberlink_msg = create_cyberlink_msg(contract.into(), links);

    Ok(Response::new()
        .add_message(cyberlink_msg)
        .add_attributes(vec![
            attr("action", "proof_address"),
            attr("nickname", nickname),
            attr("address", address),
    ]))
}

pub fn execute_remove_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    address: String,
) -> Result<Response, ContractError> {
    if !NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };

    let cw721_contract = PassportContract::default();
    let address_portid = NICKNAMES.load(deps.storage, &nickname.clone())?;
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env, address_portid.clone().portid, false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }

    cw721_contract
        .tokens
        .update(deps.storage, &address_portid.clone().portid, |token| match token {
            Some(mut token_info) => {
                let mut addresses = token_info.clone().extension.addresses.unwrap();
                let index = addresses.iter().position(|x| *x == address.clone()).unwrap();
                addresses.remove(index);
                token_info.extension.addresses = Some(addresses);
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "remove_address"),
            attr("nickname", nickname),
            attr("address", address),
        ]))
}

// proved addresses are empty during mint
// allow owner to mint passports
pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_msg: MintMsg<Extension>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.clone().sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if mint_msg.clone().extension.addresses.is_some() {
        return Err(ContractError::InvalidInitialization {});
    }

    let internal_info = MessageInfo {
        sender: env.clone().contract.address,
        funds: info.funds,
    };

    let cw721_contract = PassportContract::default();
    let response = cw721_contract.mint(deps, env, internal_info, mint_msg)?;
    Ok(response)
}

pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let cw721_contract = PassportContract::default();

    let mut nickname = String::default();
    let mut avatar = String::default();
    cw721_contract
        .tokens
        .update(deps.storage, &token_id.clone(), |token| match token {
            Some(mut token_info) => {
                nickname = token_info.clone().extension.nickname;
                avatar = token_info.clone().extension.avatar;
                token_info.extension.addresses = Some(vec![]);
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    if !NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };
    NICKNAMES.save(
        deps.storage,
        &nickname.clone(),
        &AddressPortID{
            address: info.clone().sender,
            portid: token_id.clone()
        }
    )?;

    if ACTIVE.has(deps.storage, &info.clone().sender) {
        let active = ACTIVE.load(deps.storage, &info.clone().sender)?;
        if active == token_id {
            ACTIVE.remove(deps.storage, &info.clone().sender);
        }
    }

    let nick_particle = _prepare_particle(nickname.clone());
    let avatar_particle = _check_particle(avatar.clone())?;
    let address_particle = _prepare_particle(info.clone().sender.into());

    // nickname <-> address <-> avatar
    // NOTE if one of cyberlinks from set is exist it will revert whole message and other links
    // TODO split in separate submessages?
    let links= vec![
        Link{
            from: address_particle.clone().into(),
            to: nick_particle.clone().into(),
        },
        Link{
            from: address_particle.clone().into(),
            to: avatar_particle.clone().into()
        },
        Link{
            from: nick_particle.clone().into(),
            to: address_particle.clone().into()
        },
        Link{
            from: avatar_particle.clone().into(),
            to: address_particle.clone().into()
        }
    ];

    let contract = env.clone().contract.address;
    let cyberlink_msg = create_cyberlink_msg(contract.into(), links);

    let response = cw721_contract.transfer_nft(deps, env, info, recipient, token_id)?;

    Ok(response
        .add_message(cyberlink_msg)
    )
}

pub fn execute_send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary
) -> Result<Response, ContractError> {
    let cw721_contract = PassportContract::default();

    let mut nickname = String::default();
    let mut avatar = String::default();
    cw721_contract
        .tokens
        .update(deps.storage, &token_id.clone(), |token| match token {
            Some(mut token_info) => {
                nickname = token_info.clone().extension.nickname;
                avatar = token_info.clone().extension.avatar;
                token_info.extension.addresses = Some(vec![]);
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    if ACTIVE.has(deps.storage, &info.clone().sender) {
        let active = ACTIVE.load(deps.storage, &info.clone().sender)?;
        if active == token_id {
            ACTIVE.remove(deps.storage, &info.clone().sender);
        }
    }

    // TODO think about contract as passport holder (cyberlinks/nickname?)

    let response = cw721_contract.send_nft(deps, env, info, contract, token_id, msg)?;
    Ok(response)
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let cw721_contract = PassportContract::default();

    let token_info = cw721_contract.tokens.load(deps.storage, &token_id.clone())?;

    if !NICKNAMES.has(deps.storage, &token_info.clone().extension.nickname) {
        return Err(ContractError::NicknameNotFound {});
    };
    NICKNAMES.remove(deps.storage, &token_info.clone().extension.nickname);

    let nick_particle = _prepare_particle(token_info.clone().extension.nickname);
    let cyberhole_particle = _prepare_particle("cyberhole".into());

    if ACTIVE.has(deps.storage, &info.clone().sender) {
        let active = ACTIVE.load(deps.storage, &info.clone().sender)?;
        if active == token_id {
            ACTIVE.remove(deps.storage, &info.clone().sender);
        }
    }

    // avatar <-> cyberhole <-> nickname
    // NOTE if one of cyberlinks from set is exist it will revert whole message and other links
    // TODO split in separate submessages?
    let links= vec![
        Link{
            from: cyberhole_particle.clone().into(),
            to: nick_particle.clone().into(),
        },
        Link{
            from: nick_particle.clone().into(),
            to: cyberhole_particle.clone().into()
        },
        Link{
            from: cyberhole_particle.clone().into(),
            to: token_info.extension.avatar.clone().into()
        },
        Link{
            from: token_info.extension.avatar.clone().into(),
            to: cyberhole_particle.clone().into()
        }
    ];

    let contract = env.clone().contract.address;
    let cyberlink_msg = create_cyberlink_msg(contract.into(), links);

    let response = cw721_contract.burn(deps, env, info, token_id)?;

    Ok(response
        .add_message(cyberlink_msg)
    )
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
    let cw721_contract = PassportContract::default();
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

    let owner = deps.api.addr_validate(&new_owner)?;

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.owner = owner;
            Ok(config)
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_owner")
    ]))
}

pub fn execute_set_active(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let cw721_contract = PassportContract::default();
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env, token_id.clone(), false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }
    ACTIVE.save(deps.storage, &info.clone().sender, &token_id.clone())?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "set_active"),
        attr("address", info.sender.to_string()),
        attr("token_id", token_id)
    ]))
}

pub fn try_migrate(
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
