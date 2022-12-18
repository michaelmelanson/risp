mod codegen;
mod compiler;
mod evaluator;
mod ir;
mod parser;
mod stack_frame;
mod tests;

use crate::{evaluator::Evaluator, parser::Literal};

fn main() {
    let mut readline = rustyline::Editor::<()>::new();
    let _ = readline.load_history("~/.risp-history");

    let mut evaluator = Evaluator::new();

    loop {
        let line = readline.readline("risp> ");

        match line {
            Ok(line) => {
                let result = evaluator.evaluate(&line);

                match result {
                    Ok(value) => match value {
                        Literal::String(value) => println!("(string) {:?}", value),
                        Literal::Integer(value) => println!("(integer) {:?}", value),
                    },
                    Err(error) => eprintln!("Evaluation error: {}", error),
                }
            }

            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(rustyline::error::ReadlineError::Interrupted) => break,
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
