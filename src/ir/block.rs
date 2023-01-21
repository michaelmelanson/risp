use std::collections::HashMap;

use crate::{
    compiler::{StackFrame, Symbol},
    ir,
    parser::Identifier,
};

use super::{instruction::Instruction, opcode::Opcode, slot::Slot, AssignmentTarget, Label};

#[derive(Debug)]
pub struct Block<'a, 'b> {
    stack_frame: &'a mut StackFrame<'b>,
    instructions: Vec<Instruction>,
    cache: HashMap<Symbol, Slot>,
}

impl<'a, 'b> Block<'a, 'b> {
    pub(crate) fn new(stack_frame: &'a mut StackFrame<'b>) -> Block<'a, 'b> {
        Self {
            stack_frame,
            instructions: Vec::new(),
            cache: HashMap::new(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction)
    }

    pub fn push_op(&mut self, opcode: Opcode) -> Slot {
        let destination = self.slot();
        self.push(Instruction::opcode(destination, opcode));
        destination
    }

    pub fn set_label(&mut self, label: Label) {
        self.instructions.push(Instruction::label(label));
    }

    fn slot(&mut self) -> Slot {
        Slot::new()
    }

    pub(crate) fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    pub(crate) fn insert_stack_variable(&mut self, name: &Identifier, initial_value: Slot) -> Slot {
        let Symbol::StackVariable(offset) = self.stack_frame.insert_stack_variable(name) else {
            panic!("expected stack variable");
        };

        let slot = self.push_op(ir::Opcode::StackVariable(offset));

        self.push(Instruction::Assign(
            AssignmentTarget::StackVariable(offset),
            initial_value,
        ));

        slot
    }

    pub(crate) fn resolve(&self, identifier: &Identifier) -> Option<Symbol> {
        self.stack_frame.resolve(identifier)
    }

    pub(crate) fn resolve_to_slot(&mut self, identifier: &Identifier) -> Option<Slot> {
        match self.resolve(identifier) {
            Some(symbol) => {
                if let Some(slot) = self.cache.get(&symbol) {
                    return Some(*slot);
                }

                match symbol {
                    Symbol::Argument(index) => {
                        let slot = self.push_op(ir::Opcode::FunctionArgument(index));
                        // self.cache.insert(symbol, slot);
                        Some(slot)
                    }
                    Symbol::Function(_function, _arity) => todo!("resolve function to slot"),
                    Symbol::StackVariable(offset) => {
                        let slot = self.push_op(ir::Opcode::StackVariable(offset));
                        // self.cache.insert(symbol, slot);
                        Some(slot)
                    }
                }
            }
            None => unimplemented!("undefined symbol"),
        }
    }

    pub(crate) fn stack_slots(&self) -> usize {
        self.stack_frame.stack_slots()
    }

    pub(crate) fn requires_stack_frame(&self) -> bool {
        self.stack_slots() > 0
    }
}

impl<'a, 'b> std::fmt::Display for Block<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for instruction in &self.instructions {
            match instruction {
                Instruction::Label(label) => {
                    writeln!(f, "{}:", label)?;
                }
                Instruction::Opcode {
                    destination,
                    opcode,
                } => {
                    writeln!(f, "  {} = {}", destination, opcode)?;
                }
                Instruction::Assign(target, rhs) => {
                    writeln!(f, "  {} = {}", target, rhs)?;
                }
            }
        }

        Ok(())
    }
}
