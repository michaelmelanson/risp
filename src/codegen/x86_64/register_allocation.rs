use iced_x86::code_asm::{get_gpr64, AsmRegister64};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReserveMode {
    AllowReuse,
    DenyReuse,
}

#[derive(Clone, PartialEq, Eq)]
pub struct RegisterLease(pub Register);

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

pub type Register = iced_x86::Register;
