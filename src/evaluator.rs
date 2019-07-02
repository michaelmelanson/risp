use crate::parser;
use crate::parser::{
    Term
};

use crate::compiler;

use crate::stack_frame::{
    StackFrame,
    Symbol
};


#[derive(Debug)]
pub enum EvaluationError<'a> {
    ParseError(nom::Err<(&'a str, nom::error::ErrorKind)>),
    CompileError(compiler::Error)
}

pub struct Evaluator<'a> {
    stack_frame: StackFrame<'a>,
}

impl <'a> Evaluator<'a> {
    pub fn new() -> Evaluator<'a> {
        Evaluator {
            stack_frame: StackFrame::new()
        }
    }

    pub fn evaluate<'b>(&mut self, line: &'b str) -> Result<parser::Literal, EvaluationError<'b>> {
        let (remainder, term) = parser::term(line).map_err(EvaluationError::ParseError)?;
        println!("Parsed: {:?}", term);

        if let Term::Definition(ref definition) = term {
            let mut stack_frame = self.stack_frame.push();

            for (index, arg) in definition.args.iter().enumerate() {
                stack_frame.insert(arg.clone(), Symbol::Argument(index));
            }

            let function = compiler::compile(&stack_frame, &definition.body).map_err(EvaluationError::CompileError)?;

            let symbol = Symbol::Function(function, definition.args.len());
            println!("{:?}: {:?}", definition.name, symbol);
            self.stack_frame.insert(definition.name.clone(), symbol);

        }

        let function = compiler::compile(&self.stack_frame, &term).map_err(EvaluationError::CompileError)?;
        let result = function.call();

        if remainder == "" {
            Ok(result)
        } else {
            self.evaluate(remainder)
        }
    }
}