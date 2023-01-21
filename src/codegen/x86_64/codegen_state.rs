use std::collections::HashMap;

use iced_x86::code_asm::CodeAssembler;

use crate::ir;

use super::{register_allocation::RegisterAllocator, slot::SlotValue};

pub struct CodegenState {
    pub slot_values: HashMap<ir::Slot, SlotValue>,
    pub registers: RegisterAllocator,
    labels: HashMap<ir::Label, iced_x86::code_asm::CodeLabel>,
}

impl CodegenState {
    pub fn new() -> Self {
        Self {
            slot_values: HashMap::new(),
            registers: RegisterAllocator::new(),
            labels: HashMap::new(),
        }
    }

    pub fn label(
        &mut self,
        assembler: &mut CodeAssembler,
        label: &ir::Label,
    ) -> &mut iced_x86::code_asm::CodeLabel {
        self.labels
            .entry(*label)
            .or_insert_with(|| assembler.create_label())
    }
}

impl Default for CodegenState {
    fn default() -> Self {
        Self::new()
    }
}
