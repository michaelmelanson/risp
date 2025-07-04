use std::rc::Rc;

use crate::{codegen::Function, parser::BinaryOperator, value::Value};

use super::{jump_condition::JumpCondition, slot::Slot, Label};

#[derive(Debug)]
pub enum AssignmentTarget {
    StackVariable(usize),
    FunctionArgument(usize),
}

impl std::fmt::Display for AssignmentTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignmentTarget::StackVariable(offset) => write!(f, "stack@{}", offset),
            AssignmentTarget::FunctionArgument(index) => write!(f, "arg@{}", index),
        }
    }
}

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
    PhiStart(Slot),
    PhiEnd(Vec<Slot>),
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Literal(Value::Integer(value)) => write!(f, "literal {value}"),
            Opcode::Literal(Value::String(value)) => write!(f, "literal {value}"),
            Opcode::Literal(Value::Boolean(value)) => write!(f, "literal {value}"),
            Opcode::BinaryOperator(lhs, op, rhs) => write!(f, "{lhs} {op} {rhs}"),
            Opcode::CallFunction(func, args) => {
                write!(f, "call {func} (")?;
                for (index, arg) in args.iter().enumerate() {
                    write!(f, "{arg}")?;

                    if index < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
            Opcode::FunctionArgument(index) => write!(f, "arg {index}"),
            Opcode::SetReturnValue(slot) => write!(f, "return_value = {slot}"),
            Opcode::Return => write!(f, "return"),
            Opcode::StackVariable(offset) => write!(f, "stack@{offset}"),
            Opcode::Jump(condition, label) => {
                write!(f, "jump to {label} if {condition}")
            }
            Opcode::PhiStart(slot) => {
                write!(f, "start phi({slot})")
            }
            Opcode::PhiEnd(slots) => {
                write!(f, "end phi(")?;
                for (index, slot) in slots.iter().enumerate() {
                    write!(f, "{slot}")?;

                    if index < slots.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
        }
    }
}
