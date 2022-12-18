use std::fmt::Display;
use std::rc::Rc;

use crate::{
    codegen, compiler, parser,
    stack_frame::{StackFrame, Symbol},
};

#[derive(Debug)]
pub enum EvaluationError<'a> {
    ParseError(nom::Err<(&'a str, nom::error::ErrorKind)>),
    CompileError(compiler::Error),
}

impl<'a> Display for EvaluationError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::ParseError(error) => match error {
                nom::Err::Incomplete(needed) => write!(f, "expected {:?}", needed),
                nom::Err::Error(error) => write!(f, "error {:?}", error),
                nom::Err::Failure(failure) => write!(f, "failure {:?}", failure),
            },
            EvaluationError::CompileError(error) => match error {
                compiler::Error::CompileError(error) => match error {
                    compiler::CompileError::IncorrectArity(identifier, expected, actual) => write!(
                        f,
                        "function '{}' expects {} parameters but {} were given",
                        identifier, expected, actual
                    ),
                    compiler::CompileError::NotImplemented(message) => {
                        write!(f, "not yet implemented: {}", message)
                    }
                    compiler::CompileError::UnresolvedSymbol(identifier) => {
                        write!(f, "{} is not defined", identifier)
                    }
                },
                compiler::Error::CodegenError(error) => match error {
                    codegen::CodegenError::MmapError(error) => {
                        write!(f, "could not create memory map: {}", error)
                    }
                    codegen::CodegenError::NotImplemented(message) => {
                        write!(f, "not yet implemented: {}", message)
                    }
                    codegen::CodegenError::InternalError(message) => {
                        write!(f, "internal error: {}", message)
                    }
                    codegen::CodegenError::RegisterNotAvailable(register) => {
                        write!(f, "register not available: {:?}", register)
                    }
                },
            },
        }
    }
}

pub struct Evaluator<'a> {
    stack_frame: StackFrame<'a>,
}

impl<'a> Evaluator<'a> {
    pub fn new() -> Evaluator<'a> {
        Evaluator {
            stack_frame: StackFrame::new(),
        }
    }

    pub fn evaluate<'b>(&mut self, line: &'b str) -> Result<parser::Literal, EvaluationError<'b>> {
        let (remainder, term) = parser::term(line).map_err(EvaluationError::ParseError)?;

        if let parser::Term::Definition(ref definition) = term {
            let mut stack_frame = self.stack_frame.push();

            for (index, arg) in definition.args.iter().enumerate() {
                stack_frame.insert(arg.clone(), Symbol::Argument(index));
            }

            let function = compiler::compile(&stack_frame, &definition.body)
                .map_err(EvaluationError::CompileError)?;
            let symbol = Symbol::Function(Rc::new(function), definition.args.len());
            println!("Function {} defined", definition.name);
            self.stack_frame.insert(definition.name.clone(), symbol);
        }

        let function =
            compiler::compile(&self.stack_frame, &term).map_err(EvaluationError::CompileError)?;
        let result = function.call();

        if remainder == "" {
            Ok(result)
        } else {
            self.evaluate(remainder)
        }
    }
}
