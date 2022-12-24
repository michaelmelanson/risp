use std::{
    collections::{BTreeSet, HashMap},
    convert::TryInto,
    rc::Rc,
};

use assembler::{
    mnemonic_parameter_types::{immediates::Immediate64Bit, registers::Register64Bit},
    ExecutableAnonymousMemoryMap, ExecutableAnonymousMemoryMapCreationError, InstructionStream,
};

use crate::{
    ir,
    value::{EncodedValue, Value, ValueDecodeError, ValueEncodeError, ValueType},
};

use super::CodegenResult;

#[derive(Debug)]
pub enum CodegenError {
    MmapError(ExecutableAnonymousMemoryMapCreationError),
    NotImplemented(String),
    InternalError(String),
    RegisterNotAvailable(Register64Bit),
    ValueEncodeError(ValueEncodeError),
    ValueDecodeError(ValueDecodeError),
}

pub type FuncPointer = unsafe extern "C" fn() -> EncodedValue;

fn parameter_register(index: usize) -> Result<Register64Bit, CodegenError> {
    match index {
        0 => Ok(Register64Bit::RDI),
        1 => Ok(Register64Bit::RSI),
        2 => Ok(Register64Bit::RDX),
        3 => Ok(Register64Bit::RCX),
        4 => Ok(Register64Bit::R8),
        5 => Ok(Register64Bit::R9),
        _ => Err(CodegenError::NotImplemented(
            "functions with arity greater than 6".to_owned(),
        )),
    }
}

#[derive(Clone)]
enum SlotValue {
    Literal(Value),
    FunctionArgument(usize),
    Register(Rc<RegisterLease>),
}

struct CodegenState {
    slot_values: HashMap<ir::Slot, SlotValue>,
    available_registers: BTreeSet<Register64Bit>,
}

impl CodegenState {
    fn new() -> Self {
        let available_registers = BTreeSet::from([
            // Register64Bit::RAX,
            // Register64Bit::RBX,
            Register64Bit::RCX,
            // Register64Bit::RDX,
            // Register64Bit::RBP,
            Register64Bit::RSI,
            Register64Bit::RDI,
            Register64Bit::R8,
            Register64Bit::R9,
            Register64Bit::R10,
            Register64Bit::R11,
            // Register64Bit::R12,
            // Register64Bit::R13,
            // Register64Bit::R14,
            // Register64Bit::R15,
        ]);

        Self {
            slot_values: HashMap::new(),
            available_registers,
        }
    }

    fn reserve_register(&mut self) -> CodegenResult<Rc<RegisterLease>> {
        let Some(&register) = self.available_registers.iter().next() else {
            return Err(CodegenError::NotImplemented("register spilling — no available registers".to_owned()));
        };

        self.reserve_specific_register(register)
    }

    fn reserve_specific_register(
        &mut self,
        register: Register64Bit,
    ) -> CodegenResult<Rc<RegisterLease>> {
        let Some(_) = self.available_registers.take(&register) else {
            return Err(CodegenError::RegisterNotAvailable(register));
        };

        let lease = RegisterLease(register);
        Ok(Rc::new(lease))
    }
}

pub fn codegen(
    block: ir::Block,
) -> CodegenResult<(Box<ExecutableAnonymousMemoryMap>, FuncPointer)> {
    let mut state = CodegenState::new();

    let memory_map =
        ExecutableAnonymousMemoryMap::new(4096, false, false).map_err(CodegenError::MmapError)?;
    let mut memory_map = Box::new(memory_map);

    let hints = assembler::InstructionStreamHints::default();
    let mut stream = memory_map.instruction_stream(&hints);

    println!("Assembly:");
    let func = stream.nullary_function_pointer::<EncodedValue>();
    codegen_block(&mut state, &mut stream, block)?;
    stream.finish();

    Ok((memory_map, func))
}

fn codegen_block(
    state: &mut CodegenState,
    stream: &mut InstructionStream,
    block: ir::Block,
) -> CodegenResult<()> {
    stream.push_stack_frame();

    for instruction in &block.instructions {
        let ir::Instruction {
            destination,
            opcode,
        } = instruction;

        match opcode {
            ir::Opcode::Literal(literal) => {
                state
                    .slot_values
                    .insert(*destination, SlotValue::Literal(literal.clone()));
            }

            ir::Opcode::BinaryOperator(lhs, op, rhs) => {
                let lhs = slot_to_register(state, stream, lhs)?;
                let rhs = slot_to_register(state, stream, rhs)?;

                match op {
                    ir::BinaryOperator::Add => {
                        println!("ADD {:?}, {:?}", lhs.0, rhs.0);
                        stream.add_Register64Bit_Register64Bit(lhs.0, rhs.0);
                    }
                    ir::BinaryOperator::Multiply => {
                        println!("MUL {:?}, {:?}", lhs.0, rhs.0);
                        stream.imul_Register64Bit_Register64Bit(lhs.0, rhs.0);
                    }
                }

                state
                    .slot_values
                    .insert(*destination, SlotValue::Register(lhs));
            }

            ir::Opcode::CallFunction(func, args) => {
                for (index, arg) in args.iter().enumerate() {
                    let arg_register = slot_to_register(state, stream, arg)?;
                    stream.mov_Register64Bit_Register64Bit_rm64_r64(
                        parameter_register(index)?,
                        arg_register.0,
                    );
                }

                stream.call_function(func.address());
                state.slot_values.insert(
                    *destination,
                    SlotValue::Register(Rc::new(RegisterLease(Register64Bit::RAX))),
                );
            }
            ir::Opcode::FunctionArgument(index) => {
                state
                    .slot_values
                    .insert(*destination, SlotValue::FunctionArgument(*index));
            }
            ir::Opcode::Return(slot) => {
                let value = slot_to_register(state, stream, slot)?;

                println!("MOV {:?}, {:?}", Register64Bit::RAX, value.0);
                stream.mov_Register64Bit_Register64Bit_rm64_r64(Register64Bit::RAX, value.0);

                // state
                //     .slot_values
                //     .insert(*destination, SlotValue::Register(callee_register));
            }
        };
    }

    stream.pop_stack_frame_and_return();

    Ok(())
}

#[derive(Clone)]
struct RegisterLease(pub Register64Bit);

impl Into<Register64Bit> for RegisterLease {
    fn into(self) -> Register64Bit {
        self.0
    }
}

fn slot_to_register(
    state: &mut CodegenState,
    stream: &mut InstructionStream,
    slot: &ir::Slot,
) -> CodegenResult<Rc<RegisterLease>> {
    let slot_value = state.slot_values.get(slot);
    match slot_value {
        Some(SlotValue::Register(register)) => Ok(register.clone()),
        Some(SlotValue::Literal(literal)) => {
            let value: EncodedValue = literal.try_into().map_err(CodegenError::ValueEncodeError)?;
            let reg = state.reserve_register()?;

            let value = unsafe { value.as_u64() };
            println!("MOV {:?}, {:#X}", reg.0, value);
            stream.mov_Register64Bit_Immediate64Bit(reg.0, Immediate64Bit(value as i64));

            Ok(reg)
        }

        Some(SlotValue::FunctionArgument(index)) => {
            let register = state.reserve_specific_register(parameter_register(*index)?)?;
            Ok(register)
        }

        None => Err(CodegenError::InternalError(format!(
            "slot {} has no value",
            slot
        ))),
    }
}
