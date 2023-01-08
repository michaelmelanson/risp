use std::collections::HashMap;

use crate::{
    compiler::{StackFrame, Symbol},
    ir,
    parser::Identifier,
};

use super::{instruction::Instruction, opcode::Opcode, slot::Slot};

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

    pub fn push(&mut self, opcode: Opcode) -> Slot {
        let destination = self.slot();
        self.instructions
            .push(Instruction::new(destination, opcode));
        destination
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

        self.push(ir::Opcode::AssignToStackVariable(offset, initial_value))
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

                let slot = match symbol {
                    Symbol::Argument(index) => {
                        let slot = self.push(ir::Opcode::FunctionArgument(index));
                        self.cache.insert(symbol, slot);
                        Some(slot)
                    }
                    Symbol::Function(_function, _arity) => todo!("resolve function to slot"),
                    Symbol::StackVariable(offset) => {
                        let slot = self.push(ir::Opcode::StackVariable(offset));
                        self.cache.insert(symbol, slot);
                        Some(slot)
                    }
                };

                slot
            }
            None => unimplemented!("undefined symbol"),
        }
    }

    pub(crate) fn stack_slots(&self) -> usize {
        self.stack_frame.stack_slots()
    }
}

impl<'a, 'b> std::fmt::Display for Block<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for instruction in &self.instructions {
            write!(f, "{} = {}\n", instruction.destination, instruction.opcode)?;
        }

        Ok(())
    }
}
