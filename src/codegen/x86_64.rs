use std::{
    borrow::Borrow,
    collections::{BTreeSet, HashMap},
    convert::TryInto,
    rc::Rc,
};

use memmap2::Mmap;

use iced_x86::{
    code_asm::{
        get_gpr64, ptr, qword_ptr, r8, r9, rax, rbp, rcx, rdi, rdx, rsi, AsmRegister64,
        CodeAssembler,
    },
    BlockEncoderOptions, DecoderOptions,
};

use crate::{
    ir,
    value::{EncodedValue, Value, ValueDecodeError, ValueEncodeError},
};

use super::CodegenResult;

#[derive(Debug)]
pub enum CodegenError {
    IcedError(iced_x86::IcedError),
    MmapError(std::io::Error),
    NotImplemented(String),
    InternalError(String),
    RegisterNotAvailable(Register),
    ValueEncodeError(ValueEncodeError),
    ValueDecodeError(ValueDecodeError),
}

impl From<iced_x86::IcedError> for CodegenError {
    fn from(err: iced_x86::IcedError) -> Self {
        CodegenError::IcedError(err)
    }
}

pub type FuncPointer = unsafe extern "C" fn() -> EncodedValue;

fn parameter_register(index: usize) -> Result<AsmRegister64, CodegenError> {
    match index {
        0 => Ok(rdi),
        1 => Ok(rsi),
        2 => Ok(rdx),
        3 => Ok(rcx),
        4 => Ok(r8),
        5 => Ok(r9),
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
    available_registers: BTreeSet<Register>,
}

impl CodegenState {
    fn new() -> Self {
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
        register: Register,
    ) -> CodegenResult<Rc<RegisterLease>> {
        let Some(_) = self.available_registers.take(&register) else {
            return Err(CodegenError::RegisterNotAvailable(register));
        };

        let lease = RegisterLease(register);
        Ok(Rc::new(lease))
    }
}

impl Default for CodegenState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn codegen(block: ir::Block) -> CodegenResult<(Mmap, FuncPointer)> {
    let mut state = CodegenState::new();
    let mut assembler = CodeAssembler::new(64)?;
    let mut start_label = assembler.create_label();

    assembler.set_label(&mut start_label)?;
    codegen_block(&mut state, &mut assembler, block)?;

    let mut memory_map = memmap2::MmapOptions::new()
        .len(4096)
        .map_anon()
        .map_err(CodegenError::MmapError)?;
    let result = assembler
        .assemble_options(
            memory_map.as_ptr() as u64,
            BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS,
        )
        .map_err(CodegenError::IcedError)?;

    let func_addr = result.label_ip(&start_label)?;

    let mut generated_code = result.inner.code_buffer;
    let decoder = iced_x86::Decoder::with_ip(
        64,
        &generated_code,
        memory_map.as_ptr() as u64,
        DecoderOptions::NONE,
    );

    println!("Generated assembly:");
    for instruction in decoder {
        println!("  {:#X}: {}", instruction.ip(), instruction);
    }

    generated_code.resize(memory_map.len(), 0xcc);
    memory_map.copy_from_slice(&generated_code);
    let memory_map = memory_map.make_exec().map_err(CodegenError::MmapError)?;

    let func = unsafe { std::mem::transmute::<u64, FuncPointer>(func_addr) };
    Ok((memory_map, func))
}

fn codegen_block(
    state: &mut CodegenState,
    assembler: &mut CodeAssembler,
    block: ir::Block,
) -> CodegenResult<()> {
    assembler.push(rbp)?;

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
                let lhs = slot_to_register(state, assembler, lhs)?;
                let rhs = slot_to_register(state, assembler, rhs)?;

                match op {
                    ir::BinaryOperator::Add => {
                        assembler
                            .add::<AsmRegister64, AsmRegister64>(lhs.to_gpr64(), rhs.to_gpr64())?;
                    }
                    ir::BinaryOperator::Multiply => {
                        assembler.imul_2::<AsmRegister64, AsmRegister64>(
                            lhs.to_gpr64(),
                            rhs.to_gpr64(),
                        )?;
                    }
                }

                state
                    .slot_values
                    .insert(*destination, SlotValue::Register(lhs));
            }

            ir::Opcode::CallFunction(func, args) => {
                for (index, arg) in args.iter().enumerate() {
                    let dest_register = parameter_register(index)?;
                    let arg_register = slot_to_register(state, assembler, arg)?;
                    assembler.mov::<AsmRegister64, AsmRegister64>(
                        dest_register,
                        arg_register.to_gpr64(),
                    )?;
                }

                assembler.call(func.address() as u64)?;
                state.slot_values.insert(
                    *destination,
                    SlotValue::Register(Rc::new(RegisterLease(Register::RAX))),
                );
            }

            ir::Opcode::FunctionArgument(index) => {
                state
                    .slot_values
                    .insert(*destination, SlotValue::FunctionArgument(*index));
            }
            ir::Opcode::Return(slot) => {
                let value = slot_to_register(state, assembler, slot)?;
                assembler.mov(
                    rax,
                    get_gpr64(value.0).expect("register is not a General-Purpose Register"),
                )?;
            }
        };
    }

    assembler.pop(rbp)?;
    assembler.ret()?;

    Ok(())
}

type Register = iced_x86::Register;

#[derive(Clone)]
struct RegisterLease(pub Register);

impl Into<Register> for RegisterLease {
    fn into(self) -> Register {
        self.0
    }
}

impl RegisterLease {
    fn to_gpr64(&self) -> AsmRegister64 {
        get_gpr64(self.0).expect("not a general-purpose register")
    }
}

fn slot_to_register(
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

            let value = unsafe { value.as_u64() };
            assembler.mov(reg.to_gpr64(), value)?;

            Ok(reg)
        }

        Some(SlotValue::FunctionArgument(index)) => {
            let register = state.reserve_specific_register(parameter_register(*index)?.into())?;
            Ok(register)
        }

        None => Err(CodegenError::InternalError(format!(
            "slot {} has no value",
            slot
        ))),
    }
}
