use crate::parser::{Identifier, Literal, Operator, Term};

use crate::stack_frame::{StackFrame, Symbol};

use assembler::{
    mnemonic_parameter_types::{immediates::*, registers::*},
    ExecutableAnonymousMemoryMap, InstructionStream,
};

#[derive(Debug)]
pub struct Function {
    _memory_map: Box<ExecutableAnonymousMemoryMap>,
    func: unsafe extern "C" fn() -> i64,
}

impl Function {
    pub fn call(&self) -> Literal {
        let result = unsafe { (self.func)() };

        Literal::Integer(result)
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
    MmapError(assembler::ExecutableAnonymousMemoryMapCreationError),
    CompileError(CompileError),
}

pub type CompileResult = Result<(), CompileError>;

pub fn compile(stack_frame: &StackFrame, term: &Term) -> Result<Function, Error> {
    let mut memory_map =
        Box::new(ExecutableAnonymousMemoryMap::new(4096, false, false).map_err(Error::MmapError)?);

    let hints = assembler::InstructionStreamHints::default();
    let mut stream = memory_map.instruction_stream(&hints);

    let func = stream.nullary_function_pointer::<i64>();
    stream.push_stack_frame();
    compile_term(stack_frame, &mut stream, Register64Bit::RAX, term)
        .map_err(Error::CompileError)?;
    stream.pop_stack_frame_and_return();

    stream.finish();

    Ok(Function {
        _memory_map: memory_map,
        func,
    })
}

fn compile_term(
    stack_frame: &StackFrame,
    stream: &mut InstructionStream,
    destination: Register64Bit,
    term: &Term,
) -> CompileResult {
    match term {
        Term::Expression(operator, args) => {
            compile_expression(stack_frame, stream, destination, operator, args)
        }
        Term::Literal(literal) => compile_literal(stream, destination, literal),
        Term::Identifier(_identifier) => Err(CompileError::NotImplemented(
            "compile identifier term".to_owned(),
        )),
        Term::Definition(_definition) => compile_literal(
            stream,
            destination,
            &Literal::String("<function>".to_owned()),
        ),
    }
}

fn compile_expression(
    stack_frame: &StackFrame,
    stream: &mut InstructionStream,
    destination: Register64Bit,
    operator: &Operator,
    args: &Vec<Term>,
) -> CompileResult {
    let intermediate_register = Register64Bit::R10;

    stream.push_Register64Bit_r64(intermediate_register);

    // Clear the destination register
    match operator {
        Operator::Add => stream.mov_Register64Bit_Immediate64Bit(destination, Immediate64Bit(0)),

        Operator::Multiply => {
            stream.mov_Register64Bit_Immediate64Bit(destination, Immediate64Bit(1))
        }

        Operator::CallFunction(_name) => {}
    }

    for (index, arg) in args.iter().enumerate() {
        match operator {
            Operator::Add => {
                compile_term_argument(stream, stack_frame, intermediate_register, arg)?;
                stream.add_Register64Bit_Register64Bit(destination, intermediate_register);
            }

            Operator::Multiply => {
                compile_term_argument(stream, stack_frame, intermediate_register, arg)?;
                stream.imul_Register64Bit_Register64Bit(destination, intermediate_register);
            }

            Operator::CallFunction(_identifier) => {
                let register = parameter_register(&index)?;
                compile_term_argument(stream, stack_frame, register, arg)?;
            }
        }
    }

    if let Operator::CallFunction(identifier) = operator {
        match stack_frame.resolve(&identifier) {
            Some(Symbol::Function(function, arity)) => {
                if args.len() != *arity {
                    return Err(CompileError::IncorrectArity(
                        identifier.clone(),
                        *arity,
                        args.len(),
                    ));
                }
                stream.mov_Register64Bit_Immediate64Bit(
                    Register64Bit::RAX,
                    Immediate64Bit(function.func as i64),
                );
                stream.call_Register64Bit(Register64Bit::RAX);
            }
            Some(Symbol::Argument(_)) => {
                return Err(CompileError::NotImplemented(
                    "calling function arguments".to_owned(),
                ));
            }
            None => {
                return Err(CompileError::UnresolvedSymbol(identifier.clone()));
            }
        }
    }
    stream.pop_Register64Bit_r64(intermediate_register);
    Ok(())
}

fn compile_literal(
    stream: &mut InstructionStream,
    destination: Register64Bit,
    literal: &Literal,
) -> CompileResult {
    match literal {
        Literal::Integer(int) => {
            stream.mov_Register64Bit_Immediate64Bit(destination, Immediate64Bit(*int));
        }

        Literal::String(string) => {
            stream.mov_Register64Bit_Immediate64Bit(
                destination,
                Immediate64Bit(string.as_str().as_ptr() as i64),
            );
        }
    }

    Ok(())
}
fn compile_term_argument(
    stream: &mut InstructionStream,
    stack_frame: &StackFrame,
    destination: Register64Bit,
    term: &Term,
) -> Result<(), CompileError> {
    let scratch_register = Register64Bit::R11;

    match term {
        Term::Literal(literal) => {
            compile_literal(stream, destination, literal)?;
        }

        Term::Expression(operator, args) => {
            stream.push_Register64Bit_r64(scratch_register);
            compile_expression(stack_frame, stream, scratch_register, operator, args)?;
            stream.mov_Register64Bit_Register64Bit_r64_rm64(destination, scratch_register);
            stream.pop_Register64Bit_r64(scratch_register);
        }

        Term::Identifier(identifier) => match stack_frame.resolve(&identifier) {
            Some(Symbol::Argument(index)) => {
                let register = parameter_register(&index)?;
                stream.mov_Register64Bit_Register64Bit_r64_rm64(destination, register);
            }
            Some(Symbol::Function(_function, _arity)) => {
                return Err(CompileError::NotImplemented(
                    "functions as expression arguments".to_owned(),
                ));
            }
            None => {
                return Err(CompileError::UnresolvedSymbol(identifier.clone()));
            }
        },

        Term::Definition(_definition) => {
            return Err(CompileError::NotImplemented(
                "function definition as function argument".to_owned(),
            ))
        }
    }

    Ok(())
}

fn parameter_register(index: &usize) -> Result<Register64Bit, CompileError> {
    match index {
        0 => Ok(Register64Bit::RDI),
        1 => Ok(Register64Bit::RSI),
        2 => Ok(Register64Bit::RDX),
        3 => Ok(Register64Bit::RCX),
        4 => Ok(Register64Bit::R8),
        5 => Ok(Register64Bit::R9),
        _ => Err(CompileError::NotImplemented(
            "functions with arity greater than 6".to_owned(),
        )),
    }
}
