use crate::{codegen, parser::Identifier};

#[derive(Debug)]
pub enum CompileError {
    IncorrectArity(Identifier, usize, usize),
    NotImplemented(String),
    UnresolvedSymbol(Identifier),
}

#[derive(Debug)]
pub enum CompilerError {
    CompileError(CompileError),
    CodegenError(codegen::CodegenError),
}

impl From<CompileError> for CompilerError {
    fn from(err: CompileError) -> Self {
        CompilerError::CompileError(err)
    }
}

impl From<codegen::CodegenError> for CompilerError {
    fn from(err: codegen::CodegenError) -> Self {
        CompilerError::CodegenError(err)
    }
}
