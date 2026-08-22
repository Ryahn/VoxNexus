//! Writes `packages/api-client/openapi.json` from the live utoipa document.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let dest = openapi_path();
    if let Some(parent) = dest.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            eprintln!("voxnexus-codegen: create {}: {error}", parent.display());
            return ExitCode::from(1);
        }
    }
    if let Err(error) = std::fs::write(&dest, voxnexus::openapi::spec_json()) {
        eprintln!("voxnexus-codegen: write {}: {error}", dest.display());
        return ExitCode::from(1);
    }
    println!("wrote {}", dest.display());
    ExitCode::SUCCESS
}

fn openapi_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/api-client/openapi.json")
}
