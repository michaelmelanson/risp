use iced_x86::code_asm::{
    dword_ptr, r8, r9, rbp, rcx, rdi, rdx, rsi, rsp, AsmMemoryOperand, AsmRegister64, CodeAssembler,
};

use crate::{codegen::CodegenResult, ir};

use super::CodegenError;

pub fn parameter_register(index: usize) -> Result<AsmRegister64, CodegenError> {
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

pub fn stack_variable(offset: usize) -> AsmMemoryOperand {
    dword_ptr(rbp - (8 + offset))
}

pub fn emit_function_prelude(
    assembler: &mut CodeAssembler,
    block: &ir::Block,
) -> CodegenResult<()> {
    if block.requires_stack_frame() {
        assembler.push(rbp)?;
        assembler.mov(rbp, rsp)?;

        let stack_size_bytes = block.stack_slots() * 8;
        assembler.sub(rsp, stack_size_bytes as i32)?;
    }

    Ok(())
}

pub fn emit_function_epilogue(
    assembler: &mut CodeAssembler,
    block: ir::Block,
) -> CodegenResult<()> {
    if block.requires_stack_frame() {
        assembler.mov(rsp, rbp)?;
        assembler.pop(rbp)?;
    }

    assembler.ret()?;

    Ok(())
}
