use std::rc::Rc;

use iced_x86::code_asm::{get_gpr64, rax, AsmRegister64, CodeAssembler, CodeLabel};

use crate::{
    codegen::CodegenResult,
    ir::{self, AssignmentTarget},
    parser::{ArithmeticOperator, BinaryOperator},
};

use super::{
    abi::{parameter_register, stack_variable_ref},
    codegen_state::CodegenState,
    register_allocation::RegisterLease,
    slot::{slot_to_register, SlotValue},
    Register,
};

pub fn codegen_instruction(
    state: &mut CodegenState,
    assembler: &mut CodeAssembler,
    instruction: &ir::Instruction,
    epilogue_label: &CodeLabel,
) -> CodegenResult<()> {
    match instruction {
        ir::Instruction::Label(label) => {
            let label = state.label(assembler, label);
            assembler.set_label(label)?;
        }
        ir::Instruction::Opcode {
            destination,
            opcode,
        } => {
            match opcode {
                ir::Opcode::Literal(literal) => {
                    state
                        .slot_values
                        .insert(*destination, SlotValue::Literal(literal.clone()));
                }

                ir::Opcode::BinaryOperator(lhs, BinaryOperator::ArithmeticOperator(op), rhs) => {
                    let lhs = slot_to_register(state, assembler, lhs)?;
                    let rhs = slot_to_register(state, assembler, rhs)?;

                    match op {
                        ArithmeticOperator::Add => {
                            assembler.add::<AsmRegister64, AsmRegister64>(
                                lhs.to_gpr64(),
                                rhs.to_gpr64(),
                            )?;
                        }
                        ArithmeticOperator::Multiply => {
                            assembler.imul_2::<AsmRegister64, AsmRegister64>(
                                lhs.to_gpr64(),
                                rhs.to_gpr64(),
                            )?;
                        }
                        ArithmeticOperator::Subtract => {
                            assembler.sub::<AsmRegister64, AsmRegister64>(
                                lhs.to_gpr64(),
                                rhs.to_gpr64(),
                            )?;
                        }
                        ArithmeticOperator::Divide => {
                            todo!("division operator");
                        }
                    }

                    state
                        .slot_values
                        .insert(*destination, SlotValue::Register(lhs));
                }

                ir::Opcode::BinaryOperator(_lhs, BinaryOperator::ComparisonOperator(_op), _rhs) => {
                    todo!("comparison operators")
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

                ir::Opcode::SetReturnValue(slot) => {
                    let value = slot_to_register(state, assembler, slot)?;
                    assembler.mov(
                        rax,
                        get_gpr64(value.0).expect("register is not a General-Purpose Register"),
                    )?;
                }

                ir::Opcode::Return => assembler.jmp(*epilogue_label)?,

                ir::Opcode::StackVariable(offset) => {
                    state
                        .slot_values
                        .insert(*destination, SlotValue::StackOffset(*offset));
                }

                ir::Opcode::Jump(condition, label) => {
                    let label = *state.label(assembler, label);
                    match condition {
                        ir::JumpCondition::Unconditional => {
                            assembler.jmp(label)?;
                        }
                        ir::JumpCondition::IfZero(slot) => {
                            let slot_register = slot_to_register(state, assembler, slot)?;
                            assembler.cmp(slot_register.to_gpr64(), 0)?;
                            assembler.je(label)?;
                        }
                        ir::JumpCondition::IfNotZero(slot) => {
                            let slot_register = slot_to_register(state, assembler, slot)?;
                            assembler.cmp(slot_register.to_gpr64(), 0)?;
                            assembler.jne(label)?;
                        }
                    };
                }
            };
        }
        ir::Instruction::Assign(target, rhs) => {
            let value = slot_to_register(state, assembler, rhs)?;

            match target {
                AssignmentTarget::StackVariable(offset) => {
                    assembler.mov(stack_variable_ref(*offset), value.to_gpr64())?;
                }
                AssignmentTarget::FunctionArgument(index) => {
                    assembler.mov(parameter_register(*index)?, value.to_gpr64())?;
                }
            }
        }
    }

    Ok(())
}
