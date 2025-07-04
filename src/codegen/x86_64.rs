mod abi;
mod codegen_state;
mod error;
mod function;
mod instruction;
mod slot;

use std::collections::HashMap;

use iced_x86::{
    code_asm::{r10, r11, r8, r9, rax, rcx, rdi, rdx, rsi, AsmRegister64, CodeAssembler},
    BlockEncoderOptions, DecoderOptions, Register,
};

use crate::{
    codegen::x86_64::codegen_state::CodegenState,
    ir::{self, Slot},
    value::EncodedValue,
};

use self::{
    abi::{emit_function_epilogue, emit_function_prelude},
    instruction::codegen_instruction,
};
pub use self::{error::CodegenError, function::Function};

use super::CodegenResult;

pub type FuncPointer = unsafe extern "C" fn() -> EncodedValue;

pub fn codegen(block: ir::Block) -> CodegenResult<Function> {
    let mut state = CodegenState::new();
    let mut assembler = CodeAssembler::new(64)?;
    let mut start_label = assembler.create_label();

    assembler.set_label(&mut start_label)?;
    codegen_block(&mut state, &mut assembler, block)?;

    let code_length = 4096; // TODO calculate this

    let mut memory_map = memmap2::MmapOptions::new()
        .len(code_length)
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
    // assert!(generated_code.len() == code_length);

    print_generated_code(&generated_code, memory_map.as_ptr() as u64);

    generated_code.resize(memory_map.len(), 0xcc);
    memory_map.copy_from_slice(&generated_code);
    let memory_map = memory_map.make_exec().map_err(CodegenError::MmapError)?;

    let function_pointer = unsafe { std::mem::transmute::<u64, FuncPointer>(func_addr) };
    let function = Function::new(memory_map, function_pointer);
    Ok(function)
}

fn print_generated_code(generated_code: &Vec<u8>, ip: u64) {
    let decoder = iced_x86::Decoder::with_ip(64, generated_code, ip, DecoderOptions::NONE);

    println!("Generated assembly:");
    for instruction in decoder {
        println!("  {:#X}: {}", instruction.ip(), instruction);
    }
}

fn codegen_block(
    state: &mut CodegenState,
    assembler: &mut CodeAssembler,
    block: ir::Block,
) -> CodegenResult<()> {
    let register_map = allocate_registers(&block);

    let mut epilogue_label = assembler.create_label();

    emit_function_prelude(assembler, &block)?;

    for instruction in block.instructions() {
        codegen_instruction(
            state,
            &register_map,
            assembler,
            instruction,
            &epilogue_label,
        )?;
    }

    assembler.set_label(&mut epilogue_label)?;
    emit_function_epilogue(assembler, block)?;

    Ok(())
}

fn allocate_registers(block: &ir::Block) -> HashMap<Slot, AsmRegister64> {
    let mut free_registers = vec![r8, r9, r10, r11];

    let mut register_map = HashMap::new();

    fn assign_register(
        register_map: &mut HashMap<Slot, AsmRegister64>,
        slot: &Slot,
        register: AsmRegister64,
    ) {
        if !register_map.contains_key(slot) {
            register_map.insert(slot.clone(), register);
        }
    }

    fn choose_register(
        register_map: &mut HashMap<Slot, AsmRegister64>,
        free_registers: &mut Vec<AsmRegister64>,
        slot: &Slot,
    ) -> AsmRegister64 {
        if let Some(register) = register_map.get(slot) {
            *register
        } else {
            let register = free_registers.pop().unwrap();
            assign_register(register_map, slot, register);
            register
        }
    }

    fn release_register(
        register_map: &mut HashMap<Slot, AsmRegister64>,
        free_registers: &mut Vec<AsmRegister64>,
        slot: &Slot,
    ) {
        if let Some(register) = register_map.get(slot) {
            if !free_registers.contains(register) {
                free_registers.push(*register);
            }
        }
    }

    for instruction in block.instructions().iter().rev() {
        match instruction {
            ir::Instruction::Label(_) => {}
            ir::Instruction::Assign(_, slot) => {
                choose_register(&mut register_map, &mut free_registers, slot);
            }
            ir::Instruction::Opcode {
                destination,
                opcode,
            } => match opcode {
                ir::Opcode::Return => {}
                ir::Opcode::Jump(condition, _) => match condition {
                    ir::JumpCondition::Unconditional => {}
                    ir::JumpCondition::Equal(lhs, rhs)
                    | ir::JumpCondition::NotEqual(lhs, rhs)
                    | ir::JumpCondition::Less(lhs, rhs)
                    | ir::JumpCondition::Greater(lhs, rhs)
                    | ir::JumpCondition::LessOrEqual(lhs, rhs)
                    | ir::JumpCondition::GreaterOrEqual(lhs, rhs) => {
                        choose_register(&mut register_map, &mut free_registers, lhs);
                        choose_register(&mut register_map, &mut free_registers, rhs);
                    }
                    ir::JumpCondition::Zero(slot) | ir::JumpCondition::NotZero(slot) => {
                        choose_register(&mut register_map, &mut free_registers, slot);
                    }
                },

                ir::Opcode::SetReturnValue(slot) => {
                    assign_register(&mut register_map, slot, rax);
                }

                ir::Opcode::Literal(_)
                | ir::Opcode::FunctionArgument(_)
                | ir::Opcode::StackVariable(_) => {
                    release_register(&mut register_map, &mut free_registers, destination);
                }

                ir::Opcode::CallFunction(_, parameters) => {
                    for (index, parameter) in parameters.iter().enumerate() {
                        let register = match index {
                            0 => rdi,
                            1 => rsi,
                            2 => rdx,
                            3 => rcx,
                            4 => r8,
                            5 => r9,
                            _ => panic!("too many arguments"),
                        };
                        assign_register(&mut register_map, parameter, register);
                    }
                }
                ir::Opcode::BinaryOperator(lhs, _, rhs) => {
                    let register = register_map
                        .get(destination)
                        .expect(&format!(
                            "destination register not yet set for slot {destination}"
                        ))
                        .clone();

                    assign_register(&mut register_map, destination, register);
                    assign_register(&mut register_map, lhs, register);
                    choose_register(&mut register_map, &mut free_registers, rhs);
                }
                ir::Opcode::PhiStart(slot) => {
                    let register = register_map
                        .get(destination)
                        .expect(&format!(
                            "destination register not yet set for slot {destination}"
                        ))
                        .clone();

                    assign_register(&mut register_map, slot, register);
                }
                ir::Opcode::PhiEnd(slots) => {
                    let register =
                        choose_register(&mut register_map, &mut free_registers, destination);
                    for slot in slots {
                        assign_register(&mut register_map, slot, register);
                    }
                }
            },
        };
    }

    println!("Register map:");
    for (slot, register) in &register_map {
        println!("  {}: {:?}", slot, Register::from(*register));
    }
    println!();

    register_map
}
