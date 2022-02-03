use cosmwasm_std::{Deps, StdResult};
use cw721_base::state::TokenInfo;
use crate::msg::{ConfigResponse, PortidResponse, AddressResponse, SignatureResponse};
use crate::state::{CONFIG, NICKNAMES, PassportContract, PassportMetadata, PORTID};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { owner: cfg.owner.into() })
}

pub fn query_metadata_by_nickname(deps: Deps, nickname: String) -> StdResult<PassportMetadata> {
    let address_portid = NICKNAMES.load(deps.storage, &nickname)?;
    let cw721_contract = PassportContract::default();
    let token_info = cw721_contract
        .tokens
        .load(deps.storage, address_portid.portid.as_str())?;
    Ok(token_info.extension)
}

pub fn query_passport_by_nickname(deps: Deps, nickname: String) -> StdResult<TokenInfo<PassportMetadata>> {
    let address_portid = NICKNAMES.load(deps.storage, &nickname)?;
    let cw721_contract = PassportContract::default();
    let token_info = cw721_contract
        .tokens
        .load(deps.storage, address_portid.portid.as_str())?;
    Ok(token_info)
}

pub fn query_address_by_nickname(deps: Deps, nickname: String) -> StdResult<AddressResponse> {
    let address_portid = NICKNAMES.load(deps.storage, &nickname)?;
    Ok(AddressResponse { address: address_portid.address.into() })
}

pub fn query_portid(deps: Deps) -> StdResult<PortidResponse> {
    let portid = PORTID.load(deps.storage)?;
    Ok(PortidResponse { portid: portid.into() })
}

pub fn query_nickname_address_signed(
    deps: Deps,
    nickname: String,
    address: String
) -> StdResult<SignatureResponse> {
    let address_portid = NICKNAMES.load(deps.storage, &nickname)?;
    let cw721_contract = PassportContract::default();
    let token_info = cw721_contract
        .tokens
        .load(deps.storage, address_portid.portid.as_str())?;
    let mut result = false;
    if token_info.clone().extension.addresses.is_some() {
        result = token_info.clone().extension.addresses.unwrap().iter().any(|i| i.as_ref() == address);
    }
    Ok(SignatureResponse { signed:  result })
}
