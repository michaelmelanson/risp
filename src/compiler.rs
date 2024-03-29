mod error;
pub mod stack_frame;

use crate::{
    codegen::{self, Function},
    ir::{self, AssignmentTarget, Instruction, Slot},
    parser::{
        Assignment, BinaryOperator, Block, Condition, Expression, Identifier, Literal, Loop,
        LoopPredicatePosition, Statement, VariableDeclaration,
    },
    value::Value,
};

pub use self::{
    error::{CompileError, CompilerError},
    stack_frame::{StackFrame, Symbol},
};

pub type CompileResult<T = ir::Slot> = Result<T, CompileError>;

pub fn compile<'a>(
    stack_frame: &'a mut StackFrame<'_>,
    block: &Block,
) -> Result<Function, CompilerError> {
    println!("AST:\n{:?}\n", block);

    let mut ir_block = ir::Block::new(stack_frame);
    compile_function_body(&mut ir_block, block)?;

    println!("IR:\n{}", ir_block);

    let function = codegen::codegen(ir_block)?;
    Ok(function)
}

fn compile_statement(block: &mut ir::Block, statement: &Statement) -> CompileResult {
    match statement {
        Statement::Expression(expression) => compile_expression(block, expression),
        Statement::FunctionDefinition(_definition) => compile_literal(block, &Literal::Integer(0)),
        Statement::VariableDeclaration(declaration) => {
            compile_variable_declaration(block, declaration)
        }
        Statement::Condition(condition) => compile_condition_statement(block, condition),
        Statement::Return(result) => compile_return_statement(block, result),
        Statement::Assignment(assignment) => compile_assignment_statement(block, assignment),
        Statement::Loop(loop_statement) => compile_loop_statement(block, loop_statement),
    }
}

fn compile_loop_statement(block: &mut ir::Block, loop_statement: &Loop) -> CompileResult {
    let start_label = ir::Label::new();
    let test_label = ir::Label::new();

    if loop_statement.predicate_position == LoopPredicatePosition::BeforeBlock {
        block.push_op(ir::Opcode::Jump(
            ir::JumpCondition::Unconditional,
            test_label,
        ));
    }

    block.push(Instruction::Label(start_label));
    compile_block(block, &loop_statement.block)?;

    block.push(Instruction::Label(test_label));
    let test_result = compile_expression(block, &loop_statement.predicate)?;

    block.push_op(ir::Opcode::Jump(
        ir::JumpCondition::IfNotZero(test_result),
        start_label,
    ));

    Ok(Slot::new())
}

fn compile_assignment_statement(block: &mut ir::Block, assignment: &Assignment) -> CompileResult {
    let rhs = compile_expression(block, &assignment.rhs)?;
    let lhs = assignment.lhs.clone();
    let lhs = block
        .resolve(&lhs)
        .ok_or(CompileError::UnresolvedSymbol(lhs))?;

    let target = match lhs {
        Symbol::Argument(index) => AssignmentTarget::FunctionArgument(index),
        Symbol::StackVariable(offset) => AssignmentTarget::StackVariable(offset),
        Symbol::Function(_func, _arity) => todo!(),
    };

    block.push(Instruction::Assign(target, rhs));
    Ok(rhs)
}

fn compile_return_statement(block: &mut ir::Block, result: &Expression) -> CompileResult {
    let result = compile_expression(block, result)?;
    block.push_op(ir::Opcode::SetReturnValue(result));
    block.push_op(ir::Opcode::Return);
    Ok(result)
}

fn compile_condition_statement(block: &mut ir::Block, condition: &Condition) -> CompileResult {
    let next_branch = ir::Label::new();
    let end_label = ir::Label::new();

    let branch_count = condition.branches.len();

    for (index, (predicate, branch_block)) in condition.branches.iter().enumerate() {
        let is_last_branch = index == branch_count - 1;

        if let Some(ref predicate) = predicate {
            let predicate_slot = compile_expression(block, predicate)?;
            block.push_op(ir::Opcode::Jump(
                ir::JumpCondition::IfZero(predicate_slot),
                if is_last_branch {
                    end_label
                } else {
                    next_branch
                },
            ));
        }

        compile_block(block, branch_block)?;

        if !is_last_branch {
            block.push_op(ir::Opcode::Jump(
                ir::JumpCondition::Unconditional,
                end_label,
            ));

            block.set_label(next_branch);
        }
    }

    block.set_label(end_label);

    let result_slot = ir::Slot::new();
    Ok(result_slot)
}

fn compile_block(ir_block: &mut ir::Block, block: &Block) -> CompileResult<()> {
    for statement in &block.0 {
        compile_statement(ir_block, statement)?;
    }

    Ok(())
}

fn compile_function_body(ir_block: &mut ir::Block, block: &Block) -> CompileResult {
    let mut result = None;

    let mut returned = false;

    for statement in &block.0 {
        result = Some(compile_statement(ir_block, statement)?);

        if let Statement::Return(_) = statement {
            returned = true;
            break;
        }
    }

    let Some(result ) = result else {
        unimplemented!("empty block");
    };

    if !returned {
        ir_block.push_op(ir::Opcode::SetReturnValue(result));
    }

    Ok(result)
}

fn compile_variable_declaration(
    block: &mut ir::Block,
    declaration: &VariableDeclaration,
) -> CompileResult {
    let initial_value = compile_expression(block, &declaration.value)?;
    let variable = block.insert_stack_variable(&declaration.name, initial_value);
    Ok(variable)
}

fn compile_expression(block: &mut ir::Block, expression: &Expression) -> CompileResult {
    match expression {
        Expression::Identifier(identifier) => compile_identifier(block, identifier),
        Expression::FunctionCall(identifier, args) => {
            compile_function_call(block, identifier, args)
        }
        Expression::Literal(literal) => compile_literal(block, literal),
        Expression::BinaryExpression(lhs, operator, rhs) => {
            compile_binary_operator_expression(block, lhs, operator, rhs)
        }
    }
}

fn compile_identifier(
    block: &mut ir::Block,
    identifier: &Identifier,
) -> Result<ir::Slot, CompileError> {
    match block.resolve_to_slot(identifier) {
        Some(slot) => Ok(slot),
        None => unimplemented!("undefined symbol"),
    }
}

pub fn compile_binary_operator_expression(
    block: &mut ir::Block,
    lhs: &Expression,
    operator: &BinaryOperator,
    rhs: &Expression,
) -> CompileResult {
    let lhs_slot = compile_expression(block, lhs)?;
    let rhs_slot = compile_expression(block, rhs)?;

    let slot = block.push_op(ir::Opcode::BinaryOperator(lhs_slot, *operator, rhs_slot));
    Ok(slot)
}

fn compile_function_call(
    block: &mut ir::Block,
    identifier: &Identifier,
    args: &Vec<Expression>,
) -> CompileResult {
    let mut argument_slots = Vec::with_capacity(args.len());
    for arg in args.iter() {
        let argument_slot = compile_expression(block, arg)?;
        argument_slots.push(argument_slot);
    }

    let Some(identifier_symbol) = block.resolve(identifier) else {
                  return Err(CompileError::UnresolvedSymbol(identifier.clone()));
              };

    let Symbol::Function(function, arity) = identifier_symbol else {
                  return Err(CompileError::NotImplemented(format!("calling symbol {:?}", identifier_symbol)));
              };

    if argument_slots.len() != arity {
        return Err(CompileError::IncorrectArity(
            identifier.clone(),
            argument_slots.len(),
            arity,
        ));
    }

    let return_value_slot = block.push_op(ir::Opcode::CallFunction(function, argument_slots));
    Ok(return_value_slot)
}

fn compile_literal(block: &mut ir::Block, literal: &Literal) -> CompileResult {
    match literal {
        Literal::Integer(int) => Ok(block.push_op(ir::Opcode::Literal(Value::Integer(*int)))),
        Literal::String(string) => {
            Ok(block.push_op(ir::Opcode::Literal(Value::String(string.to_string()))))
        }
    }
}
