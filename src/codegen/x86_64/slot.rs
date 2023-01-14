use std::rc::Rc;

use iced_x86::code_asm::CodeAssembler;

use crate::{
    codegen::CodegenResult,
    ir,
    value::{EncodedValue, Value},
};

use super::{
    abi::{parameter_register, stack_variable},
    codegen_state::CodegenState,
    register_allocation::{RegisterLease, ReserveMode},
    CodegenError,
};

#[derive(Clone)]
pub enum SlotValue {
    Literal(Value),
    FunctionArgument(usize),
    Register(Rc<RegisterLease>),
    StackOffset(usize),
}

pub fn slot_to_register(
    state: &mut CodegenState,
    assembler: &mut CodeAssembler,
    slot: &ir::Slot,
) -> CodegenResult<Rc<RegisterLease>> {
    let slot_value = state.slot_values.get(slot);
    match slot_value {
        Some(SlotValue::Register(register)) => Ok(register.clone()),
        Some(SlotValue::Literal(literal)) => {
            let value: EncodedValue = literal.try_into().map_err(CodegenError::ValueEncodeError)?;
            let reg = state.reserve_register()?;

            let value = unsafe { value.encoded_value() };
            assembler.mov(reg.to_gpr64(), value)?;

            Ok(reg)
        }

        Some(SlotValue::FunctionArgument(index)) => {
            let register = state.reserve_specific_register(
                parameter_register(*index)?.into(),
                ReserveMode::AllowReuse,
            )?;
            Ok(register)
        }

        Some(SlotValue::StackOffset(offset)) => {
            let offset = *offset;
            let reg = state.reserve_register()?;
            assembler.mov(reg.to_gpr64(), stack_variable(offset))?;
            Ok(reg)
        }

        None => Err(CodegenError::InternalError(format!(
            "slot {} has no value",
            slot
        ))),
    }
}
