use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use sienna_rpt::msg as rpt;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("rpt");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(rpt::Init), &out_dir);
    export_schema(&schema_for!(rpt::Handle), &out_dir);
    export_schema(&schema_for!(rpt::Query), &out_dir);
    export_schema(&schema_for!(rpt::Response), &out_dir);
}
