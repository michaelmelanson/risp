use std::{
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use iced_x86::code_asm::CodeAssembler;

use crate::{codegen::CodegenResult, ir};

use super::{
    register_allocation::{Register, RegisterLease},
    slot::SlotValue,
    CodegenError, ReserveMode,
};

pub struct CodegenState {
    pub slot_values: HashMap<ir::Slot, SlotValue>,
    available_registers: BTreeSet<Register>,
    labels: HashMap<ir::Label, iced_x86::code_asm::CodeLabel>,
}

impl CodegenState {
    pub fn new() -> Self {
        let available_registers = BTreeSet::from([
            // Register::RAX,
            // Register::RBX,
            Register::RCX, // Register::RDX,
            // Register::RBP,
            Register::RSI,
            Register::RDI,
            Register::R8,
            Register::R9,
            Register::R10,
            Register::R11,
            // Register::R12,
            // Register::R13,
            // Register::R14,
            // Register::R15,
        ]);

        Self {
            slot_values: HashMap::new(),
            available_registers,
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

    pub fn reserve_register(&mut self) -> CodegenResult<Rc<RegisterLease>> {
        let Some(&register) = self.available_registers.iter().next() else {
            return Err(CodegenError::NotImplemented("register spilling — no available registers".to_owned()));
        };

        self.reserve_specific_register(register, ReserveMode::DenyReuse)
    }

    pub fn reserve_specific_register(
        &mut self,
        register: Register,
        mode: ReserveMode,
    ) -> CodegenResult<Rc<RegisterLease>> {
        if mode == ReserveMode::DenyReuse && !self.available_registers.contains(&register) {
            return Err(CodegenError::RegisterNotAvailable(register));
        }

        self.available_registers.remove(&register);

        let lease = RegisterLease(register);
        Ok(Rc::new(lease))
    }
}

impl Default for CodegenState {
    fn default() -> Self {
        Self::new()
    }
}
