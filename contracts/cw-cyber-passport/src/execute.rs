use std::ops::{Add, Mul};
use cosmwasm_std::{attr, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Uint128};
use cw721::{Cw721Execute, Cw721Query};
use cw721_base::MintMsg;
use cw_utils::must_pay;

use cyber_std::{CyberMsgWrapper, Link};
use cyber_std::particle::{check_particle, prepare_particle};

use crate::error::ContractError;
use crate::helpers::{proof_address_cosmos, proof_address_ethereum, decode_address, prepare_cyberlink_submsg};
use crate::state::{ACTIVE, AddressPortID, Extension, LabeledAddress, NICKNAMES, PassportContract, PassportMetadata, PORTID};
use crate::state::{Config, CONFIG};

type Response = cosmwasm_std::Response<CyberMsgWrapper>;

// TODO set to constitution in production deployment
const CONSTITUTION: &str = "QmRX8qYgeZoYM3M5zzQaWEpVFdpin6FvVXvp6RPQK3oufV";
pub const CYBERSPACE_ID_MSG: u64 = 420;

pub fn execute_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<CyberMsgWrapper>>,
) -> Result<Response, ContractError> {
    let mut res = Response::new().add_attribute("action", "execute");

    let config = CONFIG.load(deps.storage)?;
    let owner = config.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    res = res.add_messages(msgs);

    Ok(res)
}

pub fn execute_create_passport(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    avatar: String,
    signature: Binary
) -> Result<Response, ContractError> {
    let verified = proof_address_cosmos(deps.as_ref(), info.clone().sender.to_string(), info.clone().sender.to_string(), CONSTITUTION.into(), signature)?;

    if !verified {
        return Err(ContractError::VerificationFailed {
            msg: "Signature verification failed".to_string(),
        });
    }

    if NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameAlreadyExists {});
    }

    let cw721_contract = PassportContract::default();

    let nickname_length = nickname.clone().len();

    if nickname_length > 32 || nickname_length < 3 {
        return Err(ContractError::NotValidName {});
    }

    for byte in nickname.as_bytes().iter() {
        // - && 0-9 && a-z
        if (*byte != 45) && (*byte < 48 || *byte > 57) && (*byte < 97 || *byte > 122) {
            return Err(ContractError::NotValidName {});
        }
    }

    if nickname_length < 8 {
        let must_pay = must_pay(&info, "boot").unwrap_or_default();
        let mul = 10u64.checked_pow(8-nickname_length as u32).unwrap();
        let to_pay = Uint128::new(1_000_000).mul(Uint128::from(mul));
        if must_pay != to_pay {
            return Err(ContractError::WrongAmountForName {});
        }
    }

    let nickname_particle = prepare_particle(nickname.clone())?;
    let avatar_particle = check_particle(avatar.clone())?;
    let address_particle = prepare_particle(info.clone().sender.into())?;

    let config = CONFIG.load(deps.storage)?;

    // prepare address <- nickname -> avatar cyberlinks
    // nickname -> address cyberlink
    let name_subgraph_submsg = prepare_cyberlink_submsg(
        config.name_subgraph.into(),
        vec![
            Link{
                from: nickname_particle.clone().into(),
                to: address_particle.clone().into()
            },
        ]
    );

    // nickname -> avatar cyberlink
    let avatar_subgraph_submsg = prepare_cyberlink_submsg(
        config.avatar_subgraph.into(),
        vec![
            Link{
                from: nickname_particle.clone().into(),
                to: avatar_particle.clone().into()
            },
        ]
    );

    let new_last_portid = PORTID.load(deps.storage).unwrap().add(1);
    let mint_msg = MintMsg {
        token_id: new_last_portid.to_string(),
        owner: info.clone().sender.into(),
        token_uri: None,
        extension: PassportMetadata {
            addresses: None,
            avatar: avatar.clone(),
            nickname: nickname.clone(),
            data: None,
            particle: None
        },
    };

    PORTID.save(deps.storage, &new_last_portid)?;

    NICKNAMES.save(
        deps.storage,
        &nickname,
        &AddressPortID{
            address: info.clone().sender,
            portid: new_last_portid.to_string()
        }
    )?;

    // set this passport as active if it's the first one
    if !ACTIVE.has(deps.storage, &info.clone().sender) {
        ACTIVE.save(deps.storage, &info.clone().sender, &new_last_portid.to_string())?;
    }

    // contract itself can only mint
    let internal_info = MessageInfo {
        sender: env.clone().contract.address,
        funds: info.funds,
    };

    let response = cw721_contract.mint(deps, env, internal_info, mint_msg)?;

    Ok(response
        .add_submessage(name_subgraph_submsg)
        .add_submessage(avatar_subgraph_submsg)
    )
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

    if nickname_length > 32 || nickname_length < 3 {
        return Err(ContractError::NotValidName {});
    }

    for byte in new_name.as_bytes().iter() {
        // - && 0-9 && a-z
        if (*byte != 45) && (*byte < 48 || *byte > 57) && (*byte < 97 || *byte > 122) {
            return Err(ContractError::NotValidName {});
        }
    }

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

    let nickname_particle = prepare_particle(new_name.clone())?;
    let address_particle = prepare_particle(info.clone().sender.into())?;

    let config = CONFIG.load(deps.storage)?;

    // prepare new nickname -> address cyberlink
    let name_subgraph_submsg = prepare_cyberlink_submsg(
        config.name_subgraph.into(),
        vec![
            Link{
                from: nickname_particle.clone().into(),
                to: address_particle.clone().into()
            },
        ]
    );

    Ok(Response::new()
        .add_submessage(name_subgraph_submsg)
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

    let avatar_particle = check_particle(new_avatar.clone())?;
    let nickname_particle = prepare_particle(nickname.clone())?;

    let config = CONFIG.load(deps.storage)?;

    // prepare nickname -> new avatar cyberlink
    let avatar_subgraph_submsg = prepare_cyberlink_submsg(
        config.avatar_subgraph.into(),
        vec![
            Link{
                from: nickname_particle.clone().into(),
                to: avatar_particle.clone().into()
            },
        ]
    );

    Ok(Response::new()
        .add_submessage(avatar_subgraph_submsg)
        .add_attributes(vec![
            attr("action", "update_avatar"),
            attr("nickname", nickname),
            attr("new_avatar", new_avatar),
    ]))
}

pub fn execute_update_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    new_data: Option<String>
) -> Result<Response, ContractError> {
    if new_data.is_some() {
        let data_length = new_data.clone().unwrap().len();
        if data_length > 256 || data_length < 3 {
            return Err(ContractError::NotValidData {});
        }
    }

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
                token_info.extension.data = new_data.clone();
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "update_data"),
            attr("nickname", nickname),
        ]))
}

pub fn execute_update_particle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    new_particle: Option<String>
) -> Result<Response, ContractError> {
    if new_particle.is_some() {
        check_particle(new_particle.clone().unwrap())?;
    }

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
                token_info.extension.particle = new_particle.clone();
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "update_particle"),
            attr("nickname", nickname),
        ]))
}

pub fn execute_proof_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    mut address: String,
    signature: Binary
) -> Result<Response, ContractError> {
    address = address.to_lowercase();

    if !NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };

    let cw721_contract = PassportContract::default();
    let address_portid = NICKNAMES.load(deps.storage, &nickname)?;
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env.clone(), address_portid.clone().portid, false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }

    // check address type and call needed proof function
    let proof_res:bool;
    if decode_address(&address).is_err() {
        proof_res = proof_address_cosmos(deps.as_ref(), address.clone(), info.sender.to_string(), CONSTITUTION.into(), signature)?
    } else {
        proof_res = proof_address_ethereum(deps.as_ref(), address.clone(), info.sender.to_string(),CONSTITUTION.into(), signature)?
    }

    // save address if not exists or there is enought space for address (<=8)
    if proof_res {
        let mut token_info = cw721_contract.tokens.load(deps.storage, &address_portid.clone().portid)?;
        if token_info.extension.addresses.is_some() {
            let mut addresses = token_info.extension.addresses.unwrap();
            if addresses.len() > 7 {
                return Err(ContractError::ErrAddAddress {
                    msg: "Too many addresses".to_string(),
                });
            }
            if addresses.iter().position(|x| *x.address == address.clone()).is_some() {
                return Err(ContractError::ErrAddAddress {
                    msg: "Address already exist".to_string(),
                });
            }
            addresses.push(LabeledAddress { label: None, address: address.clone() });
            token_info.extension.addresses = Some(addresses);
        } else {
            token_info.extension.addresses = Some(vec![LabeledAddress { label: None, address: address.clone() }]);
        };
        cw721_contract.tokens.save(deps.storage, &address_portid.clone().portid, &token_info)?;
    } else {
        return Err(ContractError::VerificationFailed {
            msg: "Signature verification failed".to_string(),
        });
    }

    let proved_address_particle = prepare_particle(address.clone())?;
    let nickname_particle = prepare_particle(nickname.clone())?;

    let config = CONFIG.load(deps.storage)?;

    // nickname -> proved_address cyberlink
    let proof_subgraph_submsg = prepare_cyberlink_submsg(
        config.proof_subgraph.into(),
        vec![
            Link{
                from: nickname_particle.clone().into(),
                to: proved_address_particle.clone().into(),
            },
        ]
    );

    Ok(Response::new()
        .add_submessage(proof_subgraph_submsg)
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
    mut address: String,
) -> Result<Response, ContractError> {
    address = address.to_lowercase();

    if !NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };

    let cw721_contract = PassportContract::default();
    let address_portid = NICKNAMES.load(deps.storage, &nickname.clone())?;
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env, address_portid.clone().portid, false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }

    // delete given address from passport metadata
    cw721_contract
        .tokens
        .update(deps.storage, &address_portid.clone().portid, |token| match token {
            Some(mut token_info) => {
                if token_info.clone().extension.addresses.is_none() {
                    return Err(ContractError::AddressNotFound {})
                }
                let mut addresses = token_info.clone().extension.addresses.unwrap();
                let index = addresses.iter().position(|x| *x.address == address.clone());
                if index.is_none() {
                    return Err(ContractError::AddressNotFound {})
                }
                addresses.remove(index.unwrap());
                if addresses.len() == 0 {
                    token_info.extension.addresses = None;
                } else {
                    token_info.extension.addresses = Some(addresses);
                }
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

// NOTE disabled
pub fn execute_mint(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _mint_msg: MintMsg<Extension>,
) -> Result<Response, ContractError> {
    Err(ContractError::DisabledFunctionality {})
}

pub fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let cw721_contract = PassportContract::default();

    let mut nickname = String::default();

    let new_owner = deps.api.addr_validate(&recipient)?;

    // clear proved addresses and data
    cw721_contract
        .tokens
        .update(deps.storage, &token_id.clone(), |token| match token {
            Some(mut token_info) => {
                nickname = token_info.clone().extension.nickname;
                token_info.extension.addresses = Some(vec![]);
                token_info.extension.data = None;
                token_info.extension.particle = None;
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    if !NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };

    // map nickname to new owner
    NICKNAMES.save(
        deps.storage,
        &nickname.clone(),
        &AddressPortID{
            address: new_owner.clone(),
            portid: token_id.clone()
        }
    )?;

    // clear this passport as active
    if ACTIVE.has(deps.storage, &info.clone().sender) {
        let active = ACTIVE.load(deps.storage, &info.clone().sender)?;
        if active == token_id {
            ACTIVE.remove(deps.storage, &info.clone().sender);
        }
    }

    let nickname_particle = prepare_particle(nickname.clone())?;
    let address_particle = prepare_particle(new_owner.clone().to_string())?;

    // link passport to new owner
    // nickname -> new address cyberlink
    let name_subgraph_submsg = prepare_cyberlink_submsg(
        config.name_subgraph.into(),
        vec![
            Link{
                from: nickname_particle.clone().into(),
                to: address_particle.clone().into()
            },
        ]
    );

    let response = cw721_contract.transfer_nft(deps, env, info, recipient, token_id)?;
    Ok(response.add_submessage(name_subgraph_submsg))
}

// NOTE disabled
pub fn execute_send_nft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _contract: String,
    _token_id: String,
    _msg: Binary
) -> Result<Response, ContractError> {
    // let cw721_contract = PassportContract::default();
    //
    // let mut nickname = String::default();
    // let mut avatar = String::default();
    // cw721_contract
    //     .tokens
    //     .update(deps.storage, &token_id.clone(), |token| match token {
    //         Some(mut token_info) => {
    //             nickname = token_info.clone().extension.nickname;
    //             avatar = token_info.clone().extension.avatar;
    //             token_info.extension.addresses = Some(vec![]);
    //             token_info.extension.data = None;
    //             token_info.extension.particle = None;
    //             Ok(token_info)
    //         }
    //         None => return Err(ContractError::TokenNotFound {}),
    //     })?;
    //
    // if !NICKNAMES.has(deps.storage, &nickname.clone()) {
    //     return Err(ContractError::NicknameNotFound {});
    // };
    //
    // // map nickname to new owner (contract in this case)
    // NICKNAMES.save(
    //     deps.storage,
    //     &nickname.clone(),
    //     &AddressPortID{
    //         address: deps.api.addr_validate(&contract)?,
    //         portid: token_id.clone()
    //     }
    // )?;
    //
    // if ACTIVE.has(deps.storage, &info.clone().sender) {
    //     let active = ACTIVE.load(deps.storage, &info.clone().sender)?;
    //     if active == token_id {
    //         ACTIVE.remove(deps.storage, &info.clone().sender);
    //     }
    // }
    //
    // let response = cw721_contract.send_nft(deps, env, info, contract, token_id, msg)?;
    // Ok(response)
    Err(ContractError::DisabledFunctionality {})
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let cw721_contract = PassportContract::default();

    let token_info = cw721_contract.tokens.load(deps.storage, &token_id.clone())?;

    // strict access only for owner without approvals
    let address_portid = NICKNAMES.load(deps.storage, &token_info.clone().extension.nickname)?;
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env.clone(), address_portid.clone().portid, false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }

    if !NICKNAMES.has(deps.storage, &token_info.clone().extension.nickname) {
        return Err(ContractError::NicknameNotFound {});
    };
    NICKNAMES.remove(deps.storage, &token_info.clone().extension.nickname);

    let nickname_particle = prepare_particle(token_info.clone().extension.nickname)?;
    let cyberhole_particle = prepare_particle("cyberhole".into())?;

    if ACTIVE.has(deps.storage, &info.clone().sender) {
        let active = ACTIVE.load(deps.storage, &info.clone().sender)?;
        if active == token_id {
            ACTIVE.remove(deps.storage, &info.clone().sender);
        }
    }

    let config = CONFIG.load(deps.storage)?;

    // nickname -> cyberhole cyberlink
    let name_subgraph_submsg = prepare_cyberlink_submsg(
        config.name_subgraph.into(),
        vec![
            Link{
                from: nickname_particle.clone().into(),
                to: cyberhole_particle.clone().into()
            },
        ]
    );

    let response = cw721_contract.burn(deps, env, info, token_id)?;
    Ok(response.add_submessage(name_subgraph_submsg))
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
        attr("action", "update_owner"),
        attr("address", new_owner.to_string()),
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

pub fn execute_set_subgraphs(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_name_subgraph: String,
    new_avatar_subgraph: String,
    new_proof_subgraph: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let owner = config.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    let name_subgraph = deps.api.addr_validate(&new_name_subgraph)?;
    let avatar_subgraph = deps.api.addr_validate(&new_avatar_subgraph)?;
    let proof_subgraph = deps.api.addr_validate(&new_proof_subgraph)?;

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.name_subgraph = name_subgraph.clone();
            config.avatar_subgraph = avatar_subgraph.clone();
            config.proof_subgraph = proof_subgraph.clone();
            Ok(config)
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_subgraphs"),
        attr("name_subgraph", name_subgraph.to_string()),
        attr("avatar_subgraph", avatar_subgraph.to_string()),
        attr("proof_subgraph", proof_subgraph.to_string()),
    ]))
}

pub fn execute_set_address_label(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nickname: String,
    address: String,
    label: Option<String>,
) -> Result<Response, ContractError> {

    if label.is_some() && label.clone().unwrap().len() > 16 {
        return Err(ContractError::NotValidLabel {});
    }

    if !NICKNAMES.has(deps.storage, &nickname.clone()) {
        return Err(ContractError::NicknameNotFound {});
    };

    let cw721_contract = PassportContract::default();
    let address_portid = NICKNAMES.load(deps.storage, &nickname.clone())?;
    let nft_owner = cw721_contract.owner_of(deps.as_ref(), env, address_portid.clone().portid, false)?;
    if nft_owner.owner != info.clone().sender {
        return Err(ContractError::Unauthorized {});
    }

    // find needed address and save label
    cw721_contract
        .tokens
        .update(deps.storage, &address_portid.clone().portid, |token| match token {
            Some(mut token_info) => {
                if token_info.clone().extension.addresses.is_none() {
                    return Err(ContractError::AddressNotFound {})
                }
                let mut addresses = token_info.clone().extension.addresses.unwrap();
                let index = addresses.iter().position(|x| *x.address == address.clone());
                if index.is_none() {
                    return Err(ContractError::AddressNotFound {});
                }
                addresses[index.unwrap()].label = label.clone();
                token_info.extension.addresses = Some(addresses);
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "set_address_label"),
        attr("nickname", nickname),
        attr("address", address),
        attr("label", label.unwrap_or_else(|| "".to_string())),
    ]))
}
