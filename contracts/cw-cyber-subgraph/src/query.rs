use cosmwasm_std::{Deps, StdResult};
use crate::state::{CONFIG, Config};

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(Config {
        owner: cfg.owner,
        executer: cfg.executer
    })
}
