//! Writes `packages/protocol/gateway.schema.json` from schemars gateway types.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let dest = schema_path();
    if let Some(parent) = dest.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            eprintln!("voxnexus-codegen: create {}: {error}", parent.display());
            return ExitCode::from(1);
        }
    }
    if let Err(error) = std::fs::write(&dest, voxnexus_protocol::gateway_schema_json()) {
        eprintln!("voxnexus-codegen: write {}: {error}", dest.display());
        return ExitCode::from(1);
    }
    println!("wrote {}", dest.display());
    ExitCode::SUCCESS
}

fn schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/protocol/gateway.schema.json")
}
