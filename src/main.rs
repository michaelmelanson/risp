mod parser;
mod compiler;
mod tests;

fn main() {
    let mut readline = rustyline::Editor::<()>::new();
    let _ = readline.load_history("~/.risp-history");

    loop {
        let line = readline.readline("risp> ");

        match line {
            Ok(line) => println!("{:?}", evaluate(&line)),

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

#[derive(Debug)]
enum EvalError<'a> {
    ParseError(nom::Err<(&'a str, nom::error::ErrorKind)>),
    CompileError(compiler::Error)
}

fn evaluate(line: &str) -> Result<parser::Literal, EvalError> {
    let (_remainder, term) = parser::term(&line).map_err(EvalError::ParseError)?;
    // println!("Parsed: {:?}", term);

    let function = compiler::compile(&term).map_err(EvalError::CompileError)?;
    let result = function.call();
    Ok(result)
}
