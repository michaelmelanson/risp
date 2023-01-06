mod error;
mod function;
pub mod stack_frame;

use crate::{
    codegen, ir,
    parser::{BinaryOperator, Block, Expression, Identifier, Literal, Statement},
    value::Value,
};

pub use self::{
    error::{CompileError, CompilerError},
    function::Function,
    stack_frame::{StackFrame, Symbol},
};

pub type CompileResult = Result<ir::Slot, CompileError>;

pub fn compile<'a>(
    stack_frame: &'a mut StackFrame<'_>,
    block: &Block,
) -> Result<Function, CompilerError> {
    // println!("AST:\n{:?}\n", block);

    let mut ir_block = ir::Block::new(stack_frame);
    let mut result = None;

    for statement in &block.0 {
        result = Some(compile_statement(&mut ir_block, statement)?);
    }

    let Some(result ) = result else {
        unimplemented!("empty block");
    };

    ir_block.push(ir::Opcode::Return(result));

    let function = codegen::codegen(ir_block)?;
    Ok(function)
}

fn compile_statement(block: &mut ir::Block, statement: &Statement) -> CompileResult {
    match statement {
        Statement::Expression(expression) => compile_expression(block, expression),
        Statement::Definition(_definition) => compile_literal(block, &Literal::Integer(0)),
    }
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

    let slot = block.push(ir::Opcode::BinaryOperator(lhs_slot, *operator, rhs_slot));
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

    let Some(identifier_symbol) = block.resolve(&identifier) else {
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

    let return_value_slot = block.push(ir::Opcode::CallFunction(function.clone(), argument_slots));
    Ok(return_value_slot)
}

fn compile_literal(block: &mut ir::Block, literal: &Literal) -> CompileResult {
    match literal {
        Literal::Integer(int) => Ok(block.push(ir::Opcode::Literal(Value::Integer(*int)))),
        Literal::String(string) => {
            Ok(block.push(ir::Opcode::Literal(Value::String(string.to_string()))))
        }
    }
}
