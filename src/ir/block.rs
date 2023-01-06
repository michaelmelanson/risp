use std::collections::HashMap;

use crate::{
    compiler::{StackFrame, Symbol},
    ir,
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

    pub(crate) fn resolve(&self, identifier: &crate::parser::Identifier) -> Option<Symbol> {
        self.stack_frame.resolve(identifier).cloned()
    }

    pub(crate) fn resolve_to_slot(
        &mut self,
        identifier: &crate::parser::Identifier,
    ) -> Option<Slot> {
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
                    Symbol::Function(_function, _arity) => todo!("compile function identifier"),
                };

                slot
            }
            None => unimplemented!("undefined symbol"),
        }
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
