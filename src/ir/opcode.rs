use std::rc::Rc;

use crate::{compiler::Function, parser::BinaryOperator, value::Value};

use super::slot::Slot;

#[derive(Debug)]
pub enum Opcode {
    Literal(Value),
    FunctionArgument(usize),
    Return(Slot),

    BinaryOperator(Slot, BinaryOperator, Slot),
    CallFunction(Rc<Function>, Vec<Slot>),
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Literal(Value::Integer(value)) => write!(f, "{:X}", value),
            Opcode::Literal(Value::String(value)) => write!(f, "{}", value),
            Opcode::BinaryOperator(lhs, op, rhs) => write!(f, "{} {} {}", lhs, op, rhs),
            Opcode::CallFunction(func, args) => {
                write!(f, "call {} (", func)?;
                for (index, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;

                    if index < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
            Opcode::FunctionArgument(index) => write!(f, "arg {}", index),
            Opcode::Return(slot) => write!(f, "return {}", slot),
        }
    }
}
