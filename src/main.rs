mod parser;
mod compiler;
mod tests;

fn main() {
    let mut readline = rustyline::Editor::<()>::new();
    let _ = readline.load_history("~/.risp-history");

    loop {
        let line = readline.readline("risp> ");

        match line {
            Ok(line) => {
                match parser::term(&line) {
                    Ok((_remainder, term)) => {
                        println!("Parsed: {:?}", term);

                        match compiler::execute(&term) {
                            Ok(result) => {
                                println!("{}", result);
                            },
                            Err(err) => {
                                println!("Compilation error: {:?}", err);
                            }
                        }
                    },

                    Err(err) => {
                        println!("Parse error: {:?}", err);
                    }
                }
            },
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    if let Err(err) = readline.save_history("~/.risp-history") {
        eprintln!("Failed to save history: {}", err);
    }
}
