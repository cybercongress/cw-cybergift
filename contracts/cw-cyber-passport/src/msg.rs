use cosmwasm_std::{Binary, CosmosMsg};
use cw721::Expiration;
use cw721_base::{
    MintMsg as CW721MintMsg,
    msg::{
        ExecuteMsg as CW721ExecuteMsg, InstantiateMsg as CW721InstantiateMsg,
        QueryMsg as CW721QueryMsg,
    },
};
use cyber_std::CyberMsgWrapper;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Extension;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,

    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    pub minter: String,

    pub owner: String,

    pub name_subgraph: String,
    pub avatar_subgraph: String,
    pub proof_subgraph: String,
}

impl From<InstantiateMsg> for CW721InstantiateMsg {
    fn from(msg: InstantiateMsg) -> CW721InstantiateMsg {
        CW721InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            minter: msg.minter,
        }
    }
}

pub type MintMsg = CW721MintMsg<Extension>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Execute {
        msgs: Vec<CosmosMsg<CyberMsgWrapper>>,
    },
    CreatePassport {
        nickname: String,
        avatar: String,
        signature: Binary,
    },
    UpdateName{
        old_nickname: String,
        new_nickname: String
    },
    UpdateAvatar{
        nickname: String,
        new_avatar: String
    },
    UpdateData{
        nickname: String,
        data: Option<String>,
    },
    UpdateParticle{
        nickname: String,
        particle: Option<String>,
    },
    ProofAddress{
        nickname: String,
        address: String,
        signature: Binary,
    },
    RemoveAddress{
        nickname: String,
        address: String,
    },
    SetOwner { owner: String },
    SetActive { token_id: String },
    SetSubgraphs {
        name_subgraph: String,
        avatar_subgraph: String,
        proof_subgraph: String,
    },
    SetAddressLabel {
        nickname: String,
        address: String,
        label: Option<String>,
    },

    // Overwrite Standard CW721 ExecuteMsg

    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft {
        recipient: String,
        token_id: String,
    },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },

    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg),

    /// Burn an NFT the sender has access to
    Burn { token_id: String },

    // Standard CW721 ExecuteMsg

    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke {
        spender: String,
        token_id: String,
    },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll {
        operator: String,
    },
}

impl From<ExecuteMsg> for CW721ExecuteMsg<Extension> {
    fn from(msg: ExecuteMsg) -> CW721ExecuteMsg<Extension> {
        match msg {
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => CW721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::Revoke { spender, token_id } => {
                CW721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                CW721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::RevokeAll { operator } => CW721ExecuteMsg::RevokeAll { operator },
            _ => panic!("cannot covert {:?} to CW721ExecuteMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Portid {},
    AddressByNickname { nickname: String },
    PassportByNickname { nickname: String },
    MetadataByNickname { nickname: String },
    PassportSigned {
        nickname: String,
        address: String
    },
    ActivePassport { address: String },

    // Standard CW721 queries
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NumTokens {},
    ContractInfo {},
    NftInfo {
        token_id: String,
    },
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Minter {},
}

impl From<QueryMsg> for CW721QueryMsg {
    fn from(msg: QueryMsg) -> CW721QueryMsg {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => CW721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => CW721QueryMsg::Approval {
                token_id,
                spender,
                include_expired
            },
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => CW721QueryMsg::Approvals {
                token_id,
                include_expired
            },
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => CW721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::NumTokens {} => CW721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => CW721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => CW721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => CW721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => CW721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                CW721QueryMsg::AllTokens { start_after, limit }
            },
            QueryMsg::Minter {} => {
                CW721QueryMsg::Minter {}
            }
            _ => panic!("cannot covert {:?} to CW721QueryMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: String,
    pub name_subgraph: String,
    pub avatar_subgraph: String,
    pub proof_subgraph: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PortidResponse {
    pub portid: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AddressResponse {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SignatureResponse {
    pub signed: bool,
}


