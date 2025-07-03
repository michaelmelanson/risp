mod error;
pub mod stack_frame;

use crate::{
    codegen::{self, Function},
    ir::{self, AssignmentTarget, Instruction, Opcode, Slot},
    parser::{
        Assignment, BinaryOperator, Block, ComparisonOperator, Condition, Expression, Identifier,
        Literal, Loop, LoopPredicatePosition, Statement, VariableDeclaration,
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
    println!("AST:\n{:#?}\n", block);

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
    let start_label = ir::Label::new("loop start");
    let test_label = ir::Label::new("loop test");
    let after_label = ir::Label::new("loop after");

    if loop_statement.predicate_position == LoopPredicatePosition::BeforeBlock {
        block.push_op(ir::Opcode::Jump(
            ir::JumpCondition::Unconditional,
            test_label.clone(),
        ));
    }

    block.set_label(start_label.clone());
    compile_block(block, &loop_statement.block)?;

    block.set_label(test_label);
    compile_predicate(
        block,
        &loop_statement.predicate,
        start_label.clone(),
        Some(after_label.clone()),
    )?;

    block.set_label(after_label.clone());

    Ok(Slot::new())
}

fn compile_predicate(
    block: &mut ir::Block<'_, '_>,
    predicate: &Expression,
    true_target: ir::Label,
    false_target: Option<ir::Label>,
) -> CompileResult {
    match predicate {
        Expression::Identifier(identifier) => {
            let identifier = compile_identifier(block, identifier)?;
            block.push_op(Opcode::Jump(
                ir::JumpCondition::NotZero(identifier),
                true_target,
            ));
            if let Some(false_target) = false_target {
                block.push_op(Opcode::Jump(ir::JumpCondition::Unconditional, false_target));
            }
            Ok(Slot::new())
        }
        Expression::FunctionCall(_identifier, _expressions) => todo!("function call in predicate"),
        Expression::Literal(_literal) => todo!("literal in predicate"),
        Expression::BinaryExpression(_lhs, BinaryOperator::ArithmeticOperator(op), _rhs) => {
            match op {
                op => unimplemented!("predicate arithmetic operator {op:?}"),
            }
        }
        Expression::BinaryExpression(lhs, BinaryOperator::ComparisonOperator(op), rhs) => {
            let lhs = compile_expression(block, lhs)?;
            let rhs = compile_expression(block, rhs)?;

            let condition = match op {
                ComparisonOperator::GreaterThan => ir::JumpCondition::Greater(lhs, rhs),
                ComparisonOperator::LessThan => ir::JumpCondition::Less(lhs, rhs),
                ComparisonOperator::Equal => ir::JumpCondition::Equal(lhs, rhs),
                ComparisonOperator::NotEqual => ir::JumpCondition::NotEqual(lhs, rhs),
                ComparisonOperator::GreaterOrEqual => ir::JumpCondition::GreaterOrEqual(lhs, rhs),
                ComparisonOperator::LessOrEqual => ir::JumpCondition::LessOrEqual(lhs, rhs),
            };

            block.push_op(Opcode::Jump(condition, true_target));

            if let Some(false_target) = false_target {
                block.push_op(Opcode::Jump(ir::JumpCondition::Unconditional, false_target));
            }

            Ok(Slot::new())
        }
    }
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
    let next_branch = ir::Label::new("condition next");
    let end_label = ir::Label::new("condition end");

    let branch_count = condition.branches.len();
    let mut branch_results = Vec::<Slot>::with_capacity(branch_count);

    for (index, (predicate, branch_block)) in condition.branches.iter().enumerate() {
        let is_last_branch = index == branch_count - 1;

        let block_label = ir::Label::new("condition block");

        if let Some(ref predicate) = predicate {
            compile_predicate(
                block,
                predicate,
                block_label.clone(),
                if is_last_branch {
                    Some(end_label.clone())
                } else {
                    Some(next_branch.clone())
                },
            )?;
            block.set_label(block_label.clone());
        }

        let result = compile_block(block, branch_block)?;
        branch_results.push(result);

        if !is_last_branch {
            block.push_op(ir::Opcode::Jump(
                ir::JumpCondition::Unconditional,
                end_label.clone(),
            ));
            block.set_label(next_branch.clone());
        }
    }

    block.set_label(end_label.clone());

    let result = block.push_op(ir::Opcode::Phi(branch_results));
    Ok(result)
}

fn compile_block(ir_block: &mut ir::Block, block: &Block) -> CompileResult {
    let mut result = None;

    for statement in &block.0 {
        result = Some(compile_statement(ir_block, statement)?);
    }

    Ok(result.unwrap())
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

    let result = match result {
        Some(result) => result,
        None => compile_literal(ir_block, &Literal::Integer(0))?,
    };

    if !returned {
        ir_block.push_op(ir::Opcode::SetReturnValue(result));
    }

    ir_block.push_op(ir::Opcode::Return);

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
        return Err(CompileError::NotImplemented(format!(
            "calling symbol {:?}",
            identifier_symbol
        )));
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
