use std::{
    cell::RefCell,
    collections::{BTreeSet, HashSet},
    rc::{Rc, Weak},
};

use iced_x86::code_asm::{get_gpr64, AsmRegister64};

use crate::codegen::CodegenResult;

use super::CodegenError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReserveMode {
    AllowReuse,
    DenyReuse,
}

#[derive(Clone)]
pub struct RegisterLease(pub Register, Weak<RegisterAllocator>);

impl From<RegisterLease> for Register {
    fn from(lease: RegisterLease) -> Self {
        lease.0
    }
}

impl RegisterLease {
    pub fn to_gpr64(&self) -> AsmRegister64 {
        get_gpr64(self.0).expect("not a general-purpose register")
    }
}

impl Drop for RegisterLease {
    fn drop(&mut self) {
        if let Some(allocator) = self.1.upgrade() {
            allocator.free_register(&self.0)
        }
    }
}

pub type Register = iced_x86::Register;

fn all_registers() -> BTreeSet<Register> {
    BTreeSet::from([
        // Register::RAX,
        // Register::RBX,
        // Register::RCX,
        // Register::RDX,
        // Register::RBP,
        // Register::RSI,
        // Register::RDI,
        Register::R8,
        Register::R9,
        Register::R10,
        Register::R11,
        // Register::R12,
        // Register::R13,
        // Register::R14,
        // Register::R15,
    ])
}

pub struct RegisterAllocator {
    used_registers: RefCell<HashSet<Register>>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            used_registers: RefCell::new(HashSet::new()),
        }
    }

    fn is_register_available(&self, register: Register) -> bool {
        !self.used_registers.borrow().contains(&register)
    }

    fn next_available_register(&self) -> Option<Register> {
        for register in all_registers() {
            if self.is_register_available(register) {
                return Some(register);
            }
        }

        None
    }

    pub fn reserve_register(self: &Rc<RegisterAllocator>) -> CodegenResult<Rc<RegisterLease>> {
        let Some(register) = self.next_available_register() else {
            return Err(CodegenError::NotImplemented("register spilling — no available registers".to_owned()));
        };

        self.reserve_specific_register(register, ReserveMode::DenyReuse)
    }

    pub fn reserve_specific_register(
        self: &Rc<RegisterAllocator>,
        register: Register,
        mode: ReserveMode,
    ) -> CodegenResult<Rc<RegisterLease>> {
        if mode == ReserveMode::DenyReuse && !self.is_register_available(register) {
            return Err(CodegenError::RegisterNotAvailable(register));
        }

        println!("Allocated register {:?}", register);
        self.used_registers.borrow_mut().insert(register);
        let lease = RegisterLease(register, Rc::<RegisterAllocator>::downgrade(self));
        Ok(Rc::new(lease))
    }

    pub fn free_register(self: &Rc<RegisterAllocator>, register: &Register) {
        println!("Freed register {:?}", register);
        self.used_registers.borrow_mut().remove(register);
    }
}
