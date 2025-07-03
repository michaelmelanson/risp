use std::collections::HashMap;

use iced_x86::code_asm::CodeAssembler;

use crate::ir;

use super::slot::SlotValue;

pub struct CodegenState {
    pub slot_values: HashMap<ir::Slot, SlotValue>,
    labels: HashMap<ir::Label, iced_x86::code_asm::CodeLabel>,
}

impl CodegenState {
    pub fn new() -> Self {
        Self {
            slot_values: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    pub fn label(
        &mut self,
        assembler: &mut CodeAssembler,
        label: &ir::Label,
    ) -> &mut iced_x86::code_asm::CodeLabel {
        self.labels
            .entry(label.clone())
            .or_insert_with(|| assembler.create_label())
    }
}

impl Default for CodegenState {
    fn default() -> Self {
        Self::new()
    }
}
