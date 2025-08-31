pub mod options;

fn main() {
    let compile_options = match options::CommandLineParser::new(std::env::args().collect()).parse()
    {
        Err(err) => {
            eprintln!("{err}");
            return;
        }
        Ok(options) => options,
    };

    let mut c = match tanitc_crate::Crate::new(compile_options) {
        Err(err) => {
            eprintln!("{err}");
            return;
        }
        Ok(c) => c,
    };

    match c.process() {
        Err(err) => eprintln!("{err}"),
        Ok(_) => println!("Compilation finished!"),
    }
}
