use tanitc_crate::{CompileOptions, Unit};

fn main() {
    let mut source_file = "main.tt".to_string();
    let mut compile_options = CompileOptions::default();

    let argv = std::env::args().collect::<Vec<String>>();
    #[allow(clippy::needless_range_loop)]
    for i in 1..argv.len() {
        if argv[i] == "-i" || argv[i] == "--input" {
            source_file = argv[i + 1].clone();
        } else if argv[i] == "--dump-tokens" {
            compile_options.verbose_tokens = true;
        } else if argv[i] == "--dump-ast" {
            compile_options.dump_ast = true;
        } else if argv[i] == "--dump-symtable" {
            compile_options.dump_symbol_table = true;
        }
    }

    tanitc_crate::set_compile_options(compile_options);

    let mut main_unit_name = source_file
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

    let main_unit = Unit::builder()
        .set_name(main_unit_name)
        .set_path(source_file)
        .build();

    tanitc_crate::register_unit(main_unit);

    if let Err(e) = Unit::process() {
        eprintln!("{e}");
    }
}
