use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use cw_cyber_gift::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, StateResponse, IsClaimedResponse, ClaimResponse, ReleaseStageStateResponse, AllReleaseStageStateResponse, MerkleRootResponse};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MerkleRootResponse), &out_dir);
    export_schema(&schema_for!(IsClaimedResponse), &out_dir);
    export_schema(&schema_for!(ClaimResponse), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(StateResponse), &out_dir);
    export_schema(&schema_for!(ReleaseStageStateResponse), &out_dir);
    export_schema(&schema_for!(AllReleaseStageStateResponse), &out_dir);
}
