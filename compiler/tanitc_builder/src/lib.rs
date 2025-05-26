use tanitc_options::Backend;

use std::{path::Path, process::Command};

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
