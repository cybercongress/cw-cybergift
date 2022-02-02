use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Empty, Coin};
use cw_storage_plus::Item;

pub type SpaceContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type Extension = SpaceMetadata;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct SpaceMetadata {
    pub source: String,
    pub target: String,
}
