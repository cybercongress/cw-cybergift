use cosmwasm_std::{Deps, StdResult};
use crate::state::{CONFIG};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admins: cfg.admins.into_iter().map(|a| a.into()).collect(),
        executors: cfg.executors.into_iter().map(|a| a.into()).collect()
    })
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ConfigResponse {
    pub admins: Vec<String>,
    pub executors: Vec<String>,
}
