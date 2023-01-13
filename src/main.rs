mod codegen;
mod compiler;
mod evaluator;
mod ir;
mod parser;
mod tests;
mod value;

use value::Value;

use crate::evaluator::Evaluator;

fn main() {
    let mut readline = rustyline::Editor::<()>::new().expect("readline error");
    let _ = readline.load_history("~/.risp-history");

    let mut evaluator = Evaluator::default();

    loop {
        let line = readline.readline("risp> ");

        match line {
            Ok(line) => {
                let result = evaluator.evaluate(&line);

                match result {
                    Ok(value) => match value {
                        Value::Integer(value) => println!("(integer) {:?}", value),
                        Value::String(value) => println!("(string) {:?}", value),
                        Value::Boolean(value) => println!("(boolean) {:?}", value),
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
