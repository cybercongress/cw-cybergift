use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw_cyber_passport::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, PortidResponse, AddressResponse, SignatureResponse};
use cw_cyber_passport::state::{Config, PassportMetadata};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(PassportMetadata), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(PortidResponse), &out_dir);
    export_schema(&schema_for!(AddressResponse), &out_dir);
    export_schema(&schema_for!(SignatureResponse), &out_dir);
}
