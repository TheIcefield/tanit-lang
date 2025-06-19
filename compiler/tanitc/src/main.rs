use tanitc_crate::Unit;
use tanitc_options::CrateType;

pub mod options;

fn crate_type_suffix(t: CrateType) -> &'static str {
    match t {
        CrateType::Bin => "",
        CrateType::StaticLib => ".a",
        CrateType::DynamicLib => ".so",
    }
}

fn main() {
    let mut compile_options =
        match options::CommandLineParser::new(std::env::args().collect()).parse() {
            Ok(options) => options,
            Err(err) => {
                eprintln!("{err}");
                return;
            }
        };

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
