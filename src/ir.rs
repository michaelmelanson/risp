use std::{
    fmt::Display,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::compiler::Function;

#[derive(Debug, Default)]
pub struct Block {
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn push(&mut self, opcode: Opcode) -> Slot {
        let destination = self.slot();
        self.instructions
            .push(Instruction::new(destination, opcode));
        destination
    }

    fn slot(&mut self) -> Slot {
        Slot::new()
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for instruction in &self.instructions {
            write!(f, "{} = {}\n", instruction.destination, instruction.opcode)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot(u64);

impl Slot {
    pub fn new() -> Slot {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Slot(id)
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub destination: Slot,
    pub opcode: Opcode,
}

impl Instruction {
    pub fn new(destination: Slot, opcode: Opcode) -> Self {
        Self {
            destination,
            opcode,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    String(String),
}

#[derive(Debug)]
pub enum Opcode {
    Literal(Literal),
    FunctionArgument(usize),
    Return(Slot),

    BinaryOperator(Slot, BinaryOperator, Slot),
    CallFunction(Rc<Function>, Vec<Slot>),
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Literal(Literal::Int(value)) => write!(f, "{:X}", value),
            Opcode::Literal(Literal::String(value)) => write!(f, "{}", value),
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

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Multiply,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Multiply => write!(f, "*"),
        }
    }
}
