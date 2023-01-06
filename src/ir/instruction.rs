use super::{opcode::Opcode, slot::Slot};

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
