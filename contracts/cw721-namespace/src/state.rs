use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Empty, Coin};
use cw_storage_plus::Item;

pub type NamespaceContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type Extension = Metadata;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub fee: Coin,
    pub owner: Option<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub parent: String,
    pub target: String,
}
