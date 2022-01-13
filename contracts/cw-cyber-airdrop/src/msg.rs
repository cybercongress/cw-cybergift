use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, Decimal, Uint128, Uint64};
use cw_utils::Expiration;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    /// Owner if none set to info.sender.
    pub owner: Option<String>,
    pub passport: String,
    pub allowed_native: String,
    pub initial_balance: Uint128,
    pub coefficient_up: Uint128,
    pub coefficient_down: Uint128,
    pub coefficient: Uint128,
    pub target_claim: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner {
        /// NewOwner if non sent, contract gets locked. Recipients can receive airdrops
        /// but owner cannot register new stages.
        new_owner: Option<String>,
    },
    /// Allows to easily debug
    UpdatePassportAddr {
        new_passport_addr: String,
    },
    UpdateTarget {
        new_target: Uint64,
    },
    RegisterMerkleRoot {
        /// MerkleRoot is hex-encoded merkle root.
        merkle_root: String,
    },
    /// Claim does not check if contract has enough funds, owner must ensure it.
    Claim {
        claim_msg: ClaimMsg,
        signature: Binary,
        claim_amount: Uint128,
        /// Proof is hex-encoded merkle proof.
        proof: Vec<String>,
    },
    Release {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimMsg {
    pub nickname: String,
    pub avatar_cid: String,
    pub gift_claiming_address_type: ClaimerType,
    pub gift_claiming_address: String,
    pub target_address: String,
    pub relay_reward: Decimal,
}

/*
impl ToString for ClaimMsg {
    fn to_string(&self) -> String {
        format!("\{{}\}", self.target_address, )
    }
}

 */

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimerType {
    Ethereum,
    Cosmos,
}

impl ToString for ClaimerType {
    fn to_string(&self) -> String {
        match self {
            ClaimerType::Ethereum => String::from("ethereum"),
            ClaimerType::Cosmos => String::from("cosmos"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    MerkleRoot {},
    IsClaimed { address: String },
    ReleaseState { address: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Option<String>,
    pub passport: String,
    pub target_claim: Uint64,
    pub allowed_native: String,
    pub current_balance: Uint128,
    pub initial_balance: Uint128,
    pub coefficient_up: Uint128,
    pub coefficient_down: Uint128,
    pub coefficient: Decimal,
    pub claims: Uint64,
    pub releases: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MerkleRootResponse {
    /// MerkleRoot is hex-encoded merkle root.
    pub merkle_root: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsClaimedResponse {
    pub is_claimed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReleaseStateResponse {
    pub balance_claim: Uint128,
    pub stage: Uint64,
    pub stage_expiration: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
