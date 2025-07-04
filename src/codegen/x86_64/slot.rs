use std::collections::HashMap;

use iced_x86::code_asm::{AsmRegister64, CodeAssembler};

use crate::{
    codegen::CodegenResult,
    ir::{self, Slot},
    value::{EncodedValue, Value},
};

use super::{
    abi::{parameter_register, stack_variable_ref},
    codegen_state::CodegenState,
    CodegenError,
};

#[derive(Clone)]
pub enum SlotValue {
    Literal(Value),
    FunctionArgument(usize),
    Register(AsmRegister64),
    StackOffset(usize),
    Phi,
}

pub fn slot_to_register(
    state: &mut CodegenState,
    register_map: &HashMap<Slot, AsmRegister64>,
    assembler: &mut CodeAssembler,
    slot: &ir::Slot,
) -> CodegenResult<AsmRegister64> {
    let slot_value = state.slot_values.get(slot);
    match slot_value {
        Some(SlotValue::Register(register)) => Ok(register.clone()),
        Some(SlotValue::Literal(literal)) => {
            let value: EncodedValue = literal.try_into().map_err(CodegenError::ValueEncodeError)?;
            let reg = register_map
                .get(slot)
                .expect(&format!("no register mapped for slot {slot}"));

            let value = unsafe { value.encoded_value() };
            assembler.mov(*reg, value)?;

            Ok(*reg)
        }

        Some(SlotValue::FunctionArgument(index)) => Ok(parameter_register(*index)?.into()),

        Some(SlotValue::StackOffset(offset)) => {
            let offset = *offset;
            let reg = register_map
                .get(slot)
                .expect("no register mapped for slot {slot}");
            assembler.mov(*reg, stack_variable_ref(offset))?;
            Ok(*reg)
        }

        Some(SlotValue::Phi) => {
            let reg = register_map
                .get(slot)
                .expect("no register mapped for slot {slot}");
            Ok(*reg)
        }

        None => Err(CodegenError::InternalError(format!(
            "slot {} has no value",
            slot
        ))),
    }
}
