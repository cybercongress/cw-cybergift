use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cyber_std::CyberMsgWrapper;

pub type PassportContract<'a> = cw721_base::Cw721Contract<'a, Extension, CyberMsgWrapper>;
pub type Extension = PassportMetadata;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub name_subspace: Addr,
    pub avatar_subspace: Addr,
    pub proof_subspace: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddressPortID {
    pub address: Addr,
    pub portid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LabeledAddress {
    pub label: Option<String>,
    pub address: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PORTID: Item<u64> = Item::new("portid");

pub const ACTIVE: Map<&Addr, String> = Map::new("active");
pub const NICKNAMES: Map<&str, AddressPortID> = Map::new("nicknames");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct PassportMetadata {
    pub addresses: Option<Vec<LabeledAddress>>,
    pub avatar: String,
    pub nickname: String,
    pub data: Option<String>,
}
