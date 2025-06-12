use std::path::PathBuf;

use tanitc_crate::Unit;
use tanitc_options::{AstSerializeMode, Backend, CompileOptions, CrateType};

fn crate_type_suffix(t: CrateType) -> &'static str {
    match t {
        CrateType::Bin => "",
        CrateType::StaticLib => ".a",
        CrateType::DynamicLib => ".so",
    }
}

fn parse_crate_type(s: &str) -> CrateType {
    match s {
        "bin" => CrateType::Bin,
        "static-lib" => CrateType::StaticLib,
        "dynamic-lib" => CrateType::DynamicLib,
        _ => {
            eprintln!("Error: unknown crate type: {s}");
            CrateType::StaticLib
        }
    }
}

fn parse_backend(s: &str) -> Backend {
    match s {
        "gcc" => Backend::Gcc,
        "clang" => Backend::Clang,
        _ => {
            eprintln!("Error: unknown backend: {s}");
            Backend::Gcc
        }
    }
}

fn parse_options() -> CompileOptions {
    let mut compile_options = CompileOptions::default();

    let argv = std::env::args().collect::<Vec<String>>();
    #[allow(clippy::needless_range_loop)]
    for i in 1..argv.len() {
        if argv[i] == "-i" || argv[i] == "--input" {
            compile_options.input_file = argv[i + 1].clone();
        } else if argv[i] == "-o" || argv[i] == "--output" {
            compile_options.output_file = argv[i + 1].clone();
        } else if argv[i] == "-l" || argv[i] == "--library" {
            compile_options.libraries.push(argv[i + 1].clone());
        } else if argv[i] == "-L" || argv[i] == "--library-path" {
            compile_options
                .libraries_paths
                .push(PathBuf::from(argv[i + 1].clone()));
        } else if argv[i] == "--dump-tokens" {
            compile_options.verbose_tokens = true;
        } else if argv[i] == "--dump-ast" || argv[i] == "--dump-ast-ron" {
            compile_options.dump_ast_mode = AstSerializeMode::Ron;
        } else if argv[i] == "--dump-ast-xml" {
            compile_options.dump_ast_mode = AstSerializeMode::Xml;
        } else if argv[i] == "--dump-ast-json" {
            compile_options.dump_ast_mode = AstSerializeMode::Json;
        } else if argv[i] == "--dump-symtable" {
            compile_options.dump_symbol_table = true;
        } else if argv[i] == "--backend" {
            compile_options.backend = parse_backend(&argv[i + 1]);
        } else if argv[i] == "--crate-type" {
            compile_options.crate_type = parse_crate_type(&argv[i + 1]);
        } else if argv[i] == "--crate-name" {
            compile_options.crate_name = argv[i + 1].clone();
        } else if argv[i] == "--allow-variants" {
            compile_options.allow_variants = true;
        }
    }

    compile_options
}

fn main() {
    let mut compile_options = parse_options();

    let input_file = compile_options.input_file.clone();

    let mut main_unit_name = input_file
        .chars()
        .rev()
        .collect::<String>()
        .splitn(2, '/')
        .collect::<Vec<&str>>()[0]
        .chars()
        .rev()
        .collect::<String>();

    if main_unit_name.ends_with(".tt") {
        for _ in 0..".tt".len() {
            main_unit_name.pop();
        }
    }

    if compile_options.crate_name.is_empty() {
        compile_options.crate_name = main_unit_name.clone();
    }

    if compile_options.output_file.is_empty() {
        compile_options.output_file = format!(
            "{}{}",
            compile_options.crate_name,
            crate_type_suffix(compile_options.crate_type)
        );
    }

    tanitc_crate::set_compile_options(compile_options.clone());

    let main_unit = Unit::builder()
        .set_name(main_unit_name)
        .set_path(input_file)
        .build();

    tanitc_crate::register_unit(main_unit);

    if let Err(e) = Unit::process() {
        eprintln!("{e}");
    }
}
