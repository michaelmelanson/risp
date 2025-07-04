use std::collections::HashMap;

use iced_x86::code_asm::{rax, AsmRegister64, CodeAssembler, CodeLabel};

use crate::{
    codegen::CodegenResult,
    ir::{self, AssignmentTarget, Slot},
    parser::{ArithmeticOperator, BinaryOperator},
};

use super::{
    abi::{parameter_register, stack_variable_ref},
    codegen_state::CodegenState,
    slot::{slot_to_register, SlotValue},
};

pub fn codegen_instruction(
    state: &mut CodegenState,
    register_map: &HashMap<Slot, AsmRegister64>,
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
                    let lhs = slot_to_register(state, register_map, assembler, lhs)?;
                    let rhs = slot_to_register(state, register_map, assembler, rhs)?;

                    match op {
                        ArithmeticOperator::Add => {
                            assembler.add::<AsmRegister64, AsmRegister64>(lhs, rhs)?;
                        }
                        ArithmeticOperator::Multiply => {
                            assembler.imul_2::<AsmRegister64, AsmRegister64>(lhs, rhs)?;
                        }
                        ArithmeticOperator::Subtract => {
                            assembler.sub::<AsmRegister64, AsmRegister64>(lhs, rhs)?;
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
                    unimplemented!("comparison operators in expressions")
                }
                ir::Opcode::CallFunction(func, args) => {
                    for arg in args {
                        slot_to_register(state, register_map, assembler, arg)?;
                    }

                    assembler.call(func.address() as u64)?;
                    state
                        .slot_values
                        .insert(*destination, SlotValue::Register(rax));
                }
                ir::Opcode::FunctionArgument(index) => {
                    state
                        .slot_values
                        .insert(*destination, SlotValue::FunctionArgument(*index));
                }
                ir::Opcode::SetReturnValue(slot) => {
                    let value = slot_to_register(state, register_map, assembler, slot)?;
                    if value != rax {
                        assembler.mov(rax, value)?;
                    }
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
                        ir::JumpCondition::Zero(identifier) => {
                            let register =
                                slot_to_register(state, register_map, assembler, identifier)?;
                            assembler.test(register, register)?;
                            assembler.jz(label)?;
                        }
                        ir::JumpCondition::NotZero(identifier) => {
                            let register =
                                slot_to_register(state, register_map, assembler, identifier)?;
                            assembler.test(register, register)?;
                            assembler.jnz(label)?;
                        }
                        ir::JumpCondition::Equal(lhs, rhs) => {
                            let lhs_register =
                                slot_to_register(state, register_map, assembler, lhs)?;
                            let rhs_register =
                                slot_to_register(state, register_map, assembler, rhs)?;
                            assembler.cmp(lhs_register, rhs_register)?;
                            assembler.je(label)?;
                        }
                        ir::JumpCondition::NotEqual(lhs, rhs) => {
                            let lhs_register =
                                slot_to_register(state, register_map, assembler, lhs)?;
                            let rhs_register =
                                slot_to_register(state, register_map, assembler, rhs)?;
                            assembler.cmp(lhs_register, rhs_register)?;
                            assembler.jne(label)?;
                        }
                        ir::JumpCondition::Greater(lhs, rhs) => {
                            let lhs_register =
                                slot_to_register(state, register_map, assembler, lhs)?;
                            let rhs_register =
                                slot_to_register(state, register_map, assembler, rhs)?;
                            assembler.cmp(lhs_register, rhs_register)?;
                            assembler.jg(label)?;
                        }
                        ir::JumpCondition::GreaterOrEqual(lhs, rhs) => {
                            let lhs_register =
                                slot_to_register(state, register_map, assembler, lhs)?;
                            let rhs_register =
                                slot_to_register(state, register_map, assembler, rhs)?;
                            assembler.cmp(lhs_register, rhs_register)?;
                            assembler.jge(label)?;
                        }
                        ir::JumpCondition::Less(lhs, rhs) => {
                            let lhs_register =
                                slot_to_register(state, register_map, assembler, lhs)?;
                            let rhs_register =
                                slot_to_register(state, register_map, assembler, rhs)?;
                            assembler.cmp(lhs_register, rhs_register)?;
                            assembler.jl(label)?;
                        }
                        ir::JumpCondition::LessOrEqual(lhs, rhs) => {
                            let lhs_register =
                                slot_to_register(state, register_map, assembler, lhs)?;
                            let rhs_register =
                                slot_to_register(state, register_map, assembler, rhs)?;
                            assembler.cmp(lhs_register, rhs_register)?;
                            assembler.jle(label)?;
                        }
                    };
                }
                ir::Opcode::PhiStart(slot) => {
                    let _ = slot_to_register(state, register_map, assembler, slot)?;
                }
                ir::Opcode::PhiEnd(_slots) => {
                    state.slot_values.insert(*destination, SlotValue::Phi);
                }
            };
        }
        ir::Instruction::Assign(target, rhs) => {
            let value = slot_to_register(state, register_map, assembler, rhs)?;

            match target {
                AssignmentTarget::StackVariable(offset) => {
                    assembler.mov(stack_variable_ref(*offset), value)?;
                }
                AssignmentTarget::FunctionArgument(index) => {
                    assembler.mov(parameter_register(*index)?, value)?;
                }
            }
        }
    }

    Ok(())
}
