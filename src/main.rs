mod compiler;
mod evaluator;
mod parser;
mod stack_frame;
mod tests;

use crate::evaluator::Evaluator;

fn main() {
    let mut readline = rustyline::Editor::<()>::new();
    let _ = readline.load_history("~/.risp-history");

    let mut evaluator = Evaluator::new();

    loop {
        let line = readline.readline("risp> ");

        match line {
            Ok(line) => println!("{:?}", evaluator.evaluate(&line)),

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
