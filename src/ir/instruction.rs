use super::{opcode::Opcode, slot::Slot, Label};

#[derive(Debug)]
pub enum Instruction {
    Label(Label),
    Opcode { destination: Slot, opcode: Opcode },
}

impl Instruction {
    pub fn opcode(destination: Slot, opcode: Opcode) -> Self {
        Self::Opcode {
            destination,
            opcode,
        }
    }

    pub fn label(label: Label) -> Self {
        Self::Label(label)
    }
}
