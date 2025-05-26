use tanitc_options::{Backend, CrateType};

use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn get_utility_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Gcc => "gcc",
        Backend::Clang => "clang",
    }
}

pub fn build_object_file(path: &Path, output: &Path, backend: Backend) -> Result<(), String> {
    let utility = get_utility_name(backend);

    let res = Command::new(utility)
        .arg("-c")
        .arg(path)
        .arg("-o")
        .arg(output)
        .output();

    if let Err(err) = res {
        return Err(format!("Error: {err}"));
    }

    Ok(())
}

fn build_static_lib(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    const UTILITY: &str = "ar";
    const OPTIONS: &str = "rcs";

    let res = Command::new(UTILITY)
        .arg(OPTIONS)
        .arg(output)
        .args(inputs)
        .output();

    if let Err(err) = res {
        return Err(format!("Error: {err}"));
    }

    Ok(())
}

pub fn link_crate_objects(
    inputs: &[PathBuf],
    output: &Path,
    crate_type: CrateType,
) -> Result<(), String> {
    match crate_type {
        CrateType::StaticLib => build_static_lib(inputs, output),
    }
}
