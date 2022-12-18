use assembler::mnemonic_parameter_types::FunctionPointer;

use crate::codegen::{self, CodegenError};
use crate::ir;
use crate::parser::{Identifier, Literal, Operator, Term};

use crate::stack_frame::{StackFrame, Symbol};

#[derive(Debug)]
pub struct Function {
    _memory_map: Box<assembler::ExecutableAnonymousMemoryMap>,
    func: unsafe extern "C" fn() -> i64,
}

impl Function {
    pub fn call(&self) -> Literal {
        // TODO other return types
        let result = unsafe { (self.func)() };

        Literal::Integer(result)
    }

    pub fn address(&self) -> impl FunctionPointer {
        self.func
    }
}

#[derive(Debug)]
pub enum CompileError {
    IncorrectArity(Identifier, usize, usize),
    NotImplemented(String),
    UnresolvedSymbol(Identifier),
}

#[derive(Debug)]
pub enum Error {
    CompileError(CompileError),
    CodegenError(CodegenError),
}

pub type CompileResult = Result<ir::Slot, CompileError>;

pub fn compile(stack_frame: &StackFrame, term: &Term) -> Result<Function, Error> {
    let mut block = ir::Block::default();
    let result = compile_term(stack_frame, &mut block, term).map_err(Error::CompileError)?;
    block.push(ir::Opcode::Return(result));

    println!("IR:\n{}", block);

    let (memory_map, func) = codegen::codegen(block).map_err(Error::CodegenError)?;

    Ok(Function {
        _memory_map: memory_map,
        func,
    })
}

fn compile_term(stack_frame: &StackFrame, block: &mut ir::Block, term: &Term) -> CompileResult {
    match term {
        Term::Expression(operator, args) => compile_expression(stack_frame, block, operator, args),
        Term::Literal(literal) => compile_literal(block, literal),
        Term::Identifier(_identifier) => Err(CompileError::NotImplemented(
            "compile identifier term".to_owned(),
        )),
        Term::Definition(_definition) => {
            compile_literal(block, &Literal::String("<function>".to_owned()))
        }
    }
}

fn compile_expression(
    stack_frame: &StackFrame,
    block: &mut ir::Block,
    operator: &Operator,
    args: &Vec<Term>,
) -> CompileResult {
    match operator {
        Operator::Add | Operator::Multiply => {
            let mut args_iter = args.iter();
            let Some(arg) = args_iter.next() else {
                return Err(CompileError::NotImplemented(
                    "arithmetic operator with no arguments".to_owned(),
                ));
            };

            let mut slot = compile_term_argument(block, stack_frame, arg)?;

            for arg in args_iter {
                let arg_slot = compile_term_argument(block, stack_frame, arg)?;

                let operator = match operator {
                    Operator::Add => ir::BinaryOperator::Add,
                    Operator::Multiply => ir::BinaryOperator::Multiply,
                    _ => panic!(),
                };

                slot = block.push(ir::Opcode::BinaryOperator(slot, operator, arg_slot));
            }

            Ok(slot)
        }

        Operator::CallFunction(identifier) => {
            let mut argument_slots = Vec::with_capacity(args.len());
            for arg in args.iter() {
                let argument_slot = compile_term_argument(block, stack_frame, arg)?;
                argument_slots.push(argument_slot);
            }

            let Some(identifier_symbol) = stack_frame.resolve(&identifier) else {
                return Err(CompileError::UnresolvedSymbol(identifier.clone()));
            };

            let Symbol::Function(function, arity) = identifier_symbol else {
                return Err(CompileError::NotImplemented(format!("calling symbol {:?}", identifier_symbol)));
            };

            if argument_slots.len() != *arity {
                return Err(CompileError::IncorrectArity(
                    identifier.clone(),
                    argument_slots.len(),
                    *arity,
                ));
            }

            let return_value_slot =
                block.push(ir::Opcode::CallFunction(function.clone(), argument_slots));
            Ok(return_value_slot)
        }
    }
}

fn compile_literal(block: &mut ir::Block, literal: &Literal) -> CompileResult {
    match literal {
        Literal::Integer(int) => Ok(block.push(ir::Opcode::Literal(ir::Literal::Int(*int)))),
        Literal::String(string) => {
            Ok(block.push(ir::Opcode::Literal(ir::Literal::String(string.to_string()))))
        }
    }
}

fn compile_term_argument(
    block: &mut ir::Block,
    stack_frame: &StackFrame,
    term: &Term,
) -> CompileResult {
    match term {
        Term::Literal(literal) => compile_literal(block, literal),
        Term::Expression(operator, args) => compile_expression(stack_frame, block, operator, args),
        Term::Identifier(identifier) => match stack_frame.resolve(&identifier) {
            Some(Symbol::Argument(index)) => {
                let slot = block.push(ir::Opcode::FunctionArgument(*index));
                Ok(slot)
            }
            Some(Symbol::Function(_function, _arity)) => Err(CompileError::NotImplemented(
                "function term argument".to_string(),
            )),
            None => Err(CompileError::UnresolvedSymbol(identifier.clone())),
        },

        Term::Definition(_definition) => Err(CompileError::NotImplemented(
            "function definition as function argument".to_owned(),
        )),
    }
}
