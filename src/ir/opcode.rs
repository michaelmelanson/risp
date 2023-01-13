use std::rc::Rc;

use crate::{compiler::Function, parser::BinaryOperator, value::Value};

use super::{jump_condition::JumpCondition, slot::Slot, Label};

#[derive(Debug)]
pub enum Opcode {
    Literal(Value),
    FunctionArgument(usize),
    SetReturnValue(Slot),
    Return,
    Jump(JumpCondition, Label),

    BinaryOperator(Slot, BinaryOperator, Slot),
    CallFunction(Rc<Function>, Vec<Slot>),
    StackVariable(usize),

    AssignToStackVariable(usize, Slot),
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Literal(Value::Integer(value)) => write!(f, "{:X}", value),
            Opcode::Literal(Value::String(value)) => write!(f, "{}", value),
            Opcode::Literal(Value::Boolean(value)) => write!(f, "{}", value),
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
            Opcode::SetReturnValue(slot) => write!(f, "return value = {}", slot),
            Opcode::Return => write!(f, "return"),
            Opcode::StackVariable(offset) => write!(f, "stack@{}", offset),
            Opcode::AssignToStackVariable(offset, slot) => write!(f, "stack@{} = {}", offset, slot),
            Opcode::Jump(condition, label) => write!(f, "jump {} to {}", condition, label),
        }
    }
}
