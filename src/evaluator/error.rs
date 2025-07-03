use std::fmt::Display;

use crate::{codegen, compiler, parser};

#[derive(Debug)]
pub enum EvaluationError<'a> {
    ParseError(nom::Err<(parser::Span<'a>, nom::error::ErrorKind)>),
    CompilerError(compiler::CompilerError),
}

impl<'a> From<nom::Err<(parser::Span<'a>, nom::error::ErrorKind)>> for EvaluationError<'a> {
    fn from(err: nom::Err<(parser::Span<'a>, nom::error::ErrorKind)>) -> Self {
        EvaluationError::ParseError(err)
    }
}

impl From<compiler::CompilerError> for EvaluationError<'_> {
    fn from(err: compiler::CompilerError) -> Self {
        EvaluationError::CompilerError(err)
    }
}

impl<'a> Display for EvaluationError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::ParseError(error) => match error {
                nom::Err::Incomplete(needed) => write!(f, "expected {:?}", needed),
                nom::Err::Error(error) => write!(f, "error {:?}", error),
                nom::Err::Failure(failure) => write!(f, "failure {:?}", failure),
            },
            EvaluationError::CompilerError(error) => match error {
                compiler::CompilerError::CompileError(error) => match error {
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
                compiler::CompilerError::CodegenError(error) => match error {
                    codegen::CodegenError::MmapError(error) => {
                        write!(f, "could not create memory map: {}", error)
                    }
                    codegen::CodegenError::NotImplemented(message) => {
                        write!(f, "not yet implemented: {}", message)
                    }
                    codegen::CodegenError::InternalError(message) => {
                        write!(f, "internal error: {}", message)
                    }

                    codegen::CodegenError::ValueEncodeError(err) => {
                        write!(f, "value encoding error: {:?}", err)
                    }
                    // codegen::CodegenError::ValueDecodeError(err) => {
                    //     write!(f, "value decoding error: {:?}", err)
                    // }
                    codegen::CodegenError::IcedError(err) => {
                        write!(f, "assembly error: {:?}", err)
                    }
                },
            },
        }
    }
}
