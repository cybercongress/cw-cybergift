use std::convert::TryInto;
use std::ops::Add;

use bech32::{ToBase32, Variant};
use cosmwasm_std::{Addr, Binary, Deps, from_binary, StdError, StdResult};
use primitive_types::H256;
use ripemd160::Digest as Ripemd160Digest;
use ripemd160::Ripemd160;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sha3::Keccak256;

use crate::error::ContractError;

pub fn proof_address_ethereum(
    deps: Deps,
    address: String,
    passport_owner: String,
    message: String,
    signature: Binary,
) -> Result<bool, ContractError> {
    let mut hasher = Keccak256::new();

    let msg = passport_owner.add(":").add(&message);
    println!("msg: {}", msg);
    // TODO add address:particle as sign message, where address is passport holder address
    hasher.update(format!("\x19Ethereum Signed Message:\n{}", msg.len()));
    hasher.update(msg);
    let hash = hasher.finalize();
    let sig = decode_signature(&signature.clone().to_string())?;

    // Decompose signature
    let (v, rs) = match sig.split_last() {
        Some(pair) => pair,
        None => {
            return Err(ContractError::IsNotEligible {
                msg: "Signature must not be empty".to_string(),
            })
        }
    };
    let recovery = get_recovery_param(*v)?;

    // Verification
    let calculated_pubkey = deps.api.secp256k1_recover_pubkey(&hash, rs, recovery)?;
    let calculated_address = ethereum_address_raw(&calculated_pubkey)?;
    let signer_address = decode_address(address.clone().as_str())?;
    if signer_address != calculated_address {
        return Err(ContractError::IsNotEligible {
            msg: "signer address is not calculated addr".to_string(),
        });
    }
    deps.api
        .secp256k1_verify(&hash, rs, &calculated_pubkey)
        .map_err(|err| ContractError::IsNotEligible {
            msg: err.to_string(),
        })
}

fn get_recovery_param(v: u8) -> StdResult<u8> {
    match v {
        27 => Ok(0),
        28 => Ok(1),
        _ => Err(StdError::generic_err("Values of v other than 27 and 28 not supported. Replay protection (EIP-155) cannot be used here."))
    }
}

/// Returns a raw 20 byte Ethereum address
fn ethereum_address_raw(pubkey: &[u8]) -> StdResult<[u8; 20]> {
    let (tag, data) = match pubkey.split_first() {
        Some(pair) => pair,
        None => return Err(StdError::generic_err("Public key must not be empty")),
    };
    if *tag != 0x04 {
        return Err(StdError::generic_err("Public key must start with 0x04"));
    }
    if data.len() != 64 {
        return Err(StdError::generic_err("Public key must be 65 bytes long"));
    }

    let hash = Keccak256::digest(data);
    Ok(hash[hash.len() - 20..].try_into().unwrap())
}

/// Returns a raw 20 byte Ethereum address from hex
pub fn decode_address(input: &str) -> StdResult<[u8; 20]> {
    if input.len() != 42 {
        return Err(StdError::generic_err(
            "Ethereum address must be 42 characters long",
        ));
    }
    if !input.starts_with("0x") {
        return Err(StdError::generic_err("Ethereum address must start wit 0x"));
    }
    let data = hex::decode(&input[2..]).map_err(|_| StdError::generic_err("hex decoding error"))?;
    Ok(data.try_into().unwrap())
}

/// Returns a raw 65 byte Ethereum signature from hex
pub fn decode_signature(input: &str) -> StdResult<[u8; 65]> {
    if input.len() != 132 {
        return Err(StdError::generic_err(
            "Ethereum signature must be 132 characters long",
        ));
    }
    if !input.starts_with("0x") {
        return Err(StdError::generic_err(
            "Ethereum signature must start wit 0x",
        ));
    }
    let data = hex::decode(&input[2..]).map_err(|_| StdError::generic_err("hex decoding error"))?;
    Ok(data.try_into().unwrap())
}

pub fn proof_address_cosmos(
    deps: Deps,
    address: String,
    passport_owner: String,
    message: String,
    signature: Binary,
) -> Result<bool, ContractError> {
    // TODO clean up before release
    // let obj = json!(
    //     {
    //         "account_number":"0",
    //         "chain_id":"",
    //         "fee":{"amount":[],"gas":"0"},
    //         "memo":"",
    //         "msgs":[
    //             {
    //                 "type":"sign/MsgSignData",
    //                 "value":{
    //                     "data": base64::encode(message),
    //                     "signer": address
    //                 }
    //             }],
    //         "sequence":"0"
    //     });

    let msg = passport_owner.add(":").add(&message);
    let mut msg_adr36:Vec<u8> = vec![123,34,97,99,99,111,117,110,116,95,110,117,109,98,101,114,34,58,34,48,34,44,34,99,104,97,105,110,95,105,100,34,58,34,34,44,34,102,101,101,34,58,123,34,97,109,111,117,110,116,34,58,91,93,44,34,103,97,115,34,58,34,48,34,125,44,34,109,101,109,111,34,58,34,34,44,34,109,115,103,115,34,58,91,123,34,116,121,112,101,34,58,34,115,105,103,110,47,77,115,103,83,105,103,110,68,97,116,97,34,44,34,118,97,108,117,101,34,58,123,34,100,97,116,97,34,58,34];
    msg_adr36.append(&mut base64::encode(msg).as_bytes().to_vec());
    msg_adr36.append(&mut vec![34,44,34,115,105,103,110,101,114,34,58,34]);
    msg_adr36.append(&mut address.clone().as_bytes().to_vec());
    msg_adr36.append(&mut vec![34,125,125,93,44,34,115,101,113,117,101,110,99,101,34,58,34,48,34,125]);

    let hash = Sha256::digest(&msg_adr36);
    let sig: CosmosSignature = from_binary(&signature).unwrap();

    // deps.api.addr_validate(&address.clone())?;
    let (prefix, _, _) = bech32::decode(&address.clone()).unwrap();

    let address_sig = pub_key_to_address(&deps, &sig.pub_key, &prefix)?;

    if address != address_sig.to_string() {
        return Err(ContractError::Unauthorized {})
    }

    let result = deps
        .api
        .secp256k1_verify(
            hash.as_ref(),
            &sig.signature.as_slice(),
            &sig.pub_key.as_slice(),
        )
        .map_err(|err| ContractError::IsNotEligible {
            msg: err.to_string(),
        });
    return result;
}

/// Converts user pubkey into Addr with given prefix
fn pub_key_to_address(deps: &Deps, pub_key: &[u8], prefix: &str) -> StdResult<Addr> {
    let compressed_pub_key = to_compressed_pub_key(pub_key)?;
    let mut ripemd160_hasher = Ripemd160::new();
    ripemd160_hasher.update(Sha256::digest(&compressed_pub_key));
    let addr_bytes = ripemd160_hasher.finalize().to_vec();
    let addr_str = bech32::encode(prefix, addr_bytes.to_base32(), Variant::Bech32).unwrap();
    // deps.api.addr_validate(&addr_str)
    Ok(Addr::unchecked(&addr_str))
}

/// Converts uncompressed pub key into compressed one
fn to_compressed_pub_key(pub_key: &[u8]) -> StdResult<Vec<u8>> {
    match pub_key.len() {
        // compressed
        33 => Ok(pub_key.to_vec()),
        // uncompressed
        65 => {
            let y = H256::from_slice(&pub_key[33..]);
            let mut pub_key_compressed = pub_key[1..33].to_vec();

            // Check whether even or odd
            if y & H256::from_low_u64_be(1) == H256::zero() {
                // 0x02
                pub_key_compressed.insert(0, 2);
            } else {
                // 0x03
                pub_key_compressed.insert(0, 3);
            }

            Ok(pub_key_compressed)
        }
        _ => Err(StdError::generic_err("PubKeyLengthIsNotValid" ))
    }
}

// {
//     pub_key: "A+MXFp7YeLMvoVlAU66Uu0z3Wtc9Cuwq0eocUhtNOmnw",
//     signature: "9O89CUdRRZj011BphnTs5JnYM9/0O0ch+XLG2DNiWqtYnA4xA5B0wmFQDOQogOxL5xKWILVMnv1IA/7s05QsIA=="
// };

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CosmosSignature {
    pub_key: Binary,
    signature: Binary,
}
