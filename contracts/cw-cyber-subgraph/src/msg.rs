use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cyber_std::Link;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub executers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Cyberlink {
        links: Vec<Link>
    },
    UpdateAdmins {
        new_admins: Vec<String>
    },
    UpdateExecutors {
        new_executors: Vec<String>
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}
