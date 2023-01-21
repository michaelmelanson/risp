mod abi;
mod codegen_state;
mod error;
mod function;
mod instruction;
mod register_allocation;
mod slot;

use iced_x86::{code_asm::CodeAssembler, BlockEncoderOptions, DecoderOptions};

use crate::{codegen::x86_64::codegen_state::CodegenState, ir, value::EncodedValue};

use self::{
    abi::{emit_function_epilogue, emit_function_prelude},
    instruction::codegen_instruction,
};
pub use self::{error::CodegenError, function::Function, register_allocation::Register};

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
    let mut epilogue_label = assembler.create_label();

    emit_function_prelude(assembler, &block)?;

    for instruction in block.instructions() {
        codegen_instruction(state, assembler, instruction, &epilogue_label)?;
    }

    assembler.set_label(&mut epilogue_label)?;
    emit_function_epilogue(assembler, block)?;

    Ok(())
}
