use tanitc_options::{Backend, CompileOptions, CrateType};

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

pub fn build_object_file(
    path: &Path,
    output: &Path,
    options: &CompileOptions,
) -> Result<(), String> {
    let utility = get_utility_name(options.backend);

    let res = Command::new(utility)
        .arg("-c")
        .arg(if options.crate_type == CrateType::DynamicLib {
            "-fPIC"
        } else {
            ""
        })
        .arg(path)
        .arg("-o")
        .arg(output)
        .output();

    if let Err(err) = res {
        return Err(format!("Error: {err}"));
    }

    Ok(())
}

fn build_executable(inputs: &[PathBuf], options: &CompileOptions) -> Result<(), String> {
    let utility = get_utility_name(options.backend);

    let res = Command::new(utility)
        .args(inputs)
        .arg("-o")
        .arg(&options.output_file)
        .output();

    if let Err(err) = res {
        return Err(format!("Error: {err}"));
    }

    Ok(())
}

fn build_static_lib(inputs: &[PathBuf], options: &CompileOptions) -> Result<(), String> {
    const UTILITY: &str = "ar";
    const OPTIONS: &str = "rcs";

    let res = Command::new(UTILITY)
        .arg(OPTIONS)
        .arg(&options.output_file)
        .args(inputs)
        .output();

    if let Err(err) = res {
        return Err(format!("Error: {err}"));
    }

    Ok(())
}

fn build_dynamic_lib(inputs: &[PathBuf], options: &CompileOptions) -> Result<(), String> {
    let utility = get_utility_name(options.backend);

    let res = Command::new(utility)
        .args(inputs)
        .arg("-shared")
        .arg("-o")
        .arg(&options.output_file)
        .output();

    if let Err(err) = res {
        return Err(format!("Error: {err}"));
    }

    Ok(())
}

pub fn link_crate_objects(inputs: &[PathBuf], options: &CompileOptions) -> Result<(), String> {
    match options.crate_type {
        CrateType::Bin => build_executable(inputs, options),
        CrateType::StaticLib => build_static_lib(inputs, options),
        CrateType::DynamicLib => build_dynamic_lib(inputs, options),
    }
}
