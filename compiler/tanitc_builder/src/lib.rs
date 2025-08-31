use tanitc_options::{Backend, CompileOptions, CrateType};

use std::{path::Path, process::Command};

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

    let mut cmd = Command::new(utility);

    cmd.arg("-c");

    if options.crate_type == CrateType::DynamicLib {
        cmd.arg("-fPIC");
    }

    cmd.arg("-o");
    cmd.arg(output);
    cmd.arg(path);

    execute_command(&mut cmd)
}

fn build_executable(inputs: &[&Path], options: &CompileOptions) -> Result<(), String> {
    let utility = get_utility_name(options.backend);

    let mut cmd = Command::new(utility);
    cmd.args(inputs);
    cmd.arg("-o");
    cmd.arg(&options.output_file);

    for dir in options.libraries_paths.iter() {
        cmd.arg(format!("-L{}", dir.to_str().unwrap()));
    }

    for lib in options.libraries.iter() {
        cmd.arg(format!("-l{lib}"));
    }

    execute_command(&mut cmd)
}

fn build_static_lib(inputs: &[&Path], options: &CompileOptions) -> Result<(), String> {
    const UTILITY: &str = "ar";
    const OPTIONS: &str = "rcs";

    let mut cmd = Command::new(UTILITY);
    cmd.arg(OPTIONS);
    cmd.arg(&options.output_file);
    cmd.args(inputs);

    execute_command(&mut cmd)
}

fn build_dynamic_lib(inputs: &[&Path], options: &CompileOptions) -> Result<(), String> {
    let utility = get_utility_name(options.backend);

    let mut cmd = Command::new(utility);
    cmd.args(inputs);
    cmd.arg("-shared");
    cmd.arg("-o");
    cmd.arg(&options.output_file);

    execute_command(&mut cmd)
}

pub fn link_crate_objects(inputs: &[&Path], options: &CompileOptions) -> Result<(), String> {
    match options.crate_type {
        CrateType::Bin => build_executable(inputs, options),
        CrateType::StaticLib => build_static_lib(inputs, options),
        CrateType::DynamicLib => build_dynamic_lib(inputs, options),
    }
}

fn execute_command(cmd: &mut Command) -> Result<(), String> {
    match cmd.output() {
        Ok(out) => {
            if !out.status.success() {
                Err(String::from_utf8(out.stderr).unwrap())
            } else {
                Ok(())
            }
        }
        Err(err) => Err(format!("Error: {err}")),
    }
}
