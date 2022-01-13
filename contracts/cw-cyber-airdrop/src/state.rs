use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

// TODO move coefficient, current_balance, claims and releases out of config
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Option<Addr>,
    pub passport_addr: Addr,
    /// target_claim amount of claimed accounts to start release (activate gift)
    pub target_claim: Uint64,
    pub allowed_native: String,
    pub current_balance: Uint128,
    pub initial_balance: Uint128,
    pub coefficient_up: Uint128,
    pub coefficient_down: Uint128,
    pub coefficient: Decimal,
    /// amount of claimed accounts
    pub claims: Uint64,
    /// amount of total releases by all accounts
    pub releases: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReleaseState {
    pub balance_claim: Uint128,
    pub stage: Uint64,
    pub stage_expiration: Expiration,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const MERKLE_ROOT_PREFIX: &str = "merkle_root";
pub const MERKLE_ROOT: Item<String> = Item::new(MERKLE_ROOT_PREFIX);

pub const CLAIM_PREFIX: &str = "claim";
pub const CLAIM: Map<String, bool> = Map::new(CLAIM_PREFIX);

pub const RELEASE_PREFIX: &str = "release";
pub const RELEASE: Map<String, ReleaseState> = Map::new(RELEASE_PREFIX);
