use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw721_namespace::state::Metadata;
use crate::state::Route;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub namespaces: Vec<Route>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RouteData {
    pub namespace: String,
    pub data: Metadata,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateNamespaces { namespaces: Vec<Route> },
    CreatePassport {
        citizen: String,
        routes: Vec<RouteData>,
    },
    SetGift { gift: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    NamespacesList {}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NamespacesListResponse {
    pub namespaces: Vec<Route>
}


