use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

// TODO move coefficient, current_balance, claims and releases out of config
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Option<Addr>,
    pub passport_addr: Addr,
    pub treasury_addr: Addr,
    pub target_claim: Uint64,
    pub allowed_native: String,
    pub initial_balance: Uint128,
    pub coefficient_up: Uint128,
    pub coefficient_down: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub current_balance: Uint128,
    pub coefficient: Decimal,
    pub claims: Uint64,
    pub releases: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReleaseState {
    pub address: Addr,
    pub balance_claim: Uint128,
    pub stage: Uint64,
    pub stage_expiration: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimState {
    pub claim: Uint128,
    pub multiplier: Decimal,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const STATE_KEY: &str = "state";
pub const STATE: Item<State> = Item::new(STATE_KEY);

pub const MERKLE_ROOT_PREFIX: &str = "merkle_root";
pub const MERKLE_ROOT: Item<String> = Item::new(MERKLE_ROOT_PREFIX);

pub const CLAIM_PREFIX: &str = "claim";
pub const CLAIM: Map<String, ClaimState> = Map::new(CLAIM_PREFIX);

pub const RELEASE_PREFIX: &str = "release";
pub const RELEASE: Map<String, ReleaseState> = Map::new(RELEASE_PREFIX);

pub const RELEASE_INFO_PREFIX: &str = "release_info";
pub const RELEASE_INFO: Map<u64, Uint64> = Map::new(RELEASE_INFO_PREFIX);
