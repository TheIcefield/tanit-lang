use garnet_script::lexer;

fn main() {
    let mut source_file = "".to_string();
    let mut dump_tokens = false;

    let mut argv = std::env::args();
    argv.next();
    for arg in argv {
        if arg == "--dump-tokens" {
            dump_tokens = true;
        } else {
            source_file = arg;
        }
    }

    if source_file.len() == 0 {
        println!("Error: Source file not specified");
        return;
    }

    let mut lexer = {
        let lexer = lexer::Lexer::from_file(&source_file, dump_tokens);
        match lexer {
            Err(e) => {
                println!("Error: {}", e);
                return;
            },
            Ok(lexer) => lexer,
        }
    };

    let tokens = lexer.collect();

    lexer::dump_tokens(&tokens);
    
}
