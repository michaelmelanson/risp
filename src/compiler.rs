mod error;
mod function;

use crate::{
    codegen, ir,
    parser::{BinaryOperator, Block, Expression, Identifier, Literal, Statement},
    stack_frame::{StackFrame, Symbol},
    value::Value,
};

pub use self::{
    error::{CompileError, CompilerError},
    function::Function,
};

pub type CompileResult = Result<ir::Slot, CompileError>;

pub fn compile(stack_frame: &StackFrame, block: &Block) -> Result<Function, CompilerError> {
    println!("AST:\n{:?}\n", block);

    let mut ir_block = ir::Block::default();
    let mut result = None;

    for statement in &block.0 {
        result = Some(compile_statement(stack_frame, &mut ir_block, statement)?);
    }

    let Some(result ) = result else {
        unimplemented!("empty block");
    };

    ir_block.push(ir::Opcode::Return(result));

    let function = codegen::codegen(ir_block)?;
    Ok(function)
}

fn compile_statement(
    stack_frame: &StackFrame,
    block: &mut ir::Block,
    statement: &Statement,
) -> CompileResult {
    match statement {
        Statement::Expression(expression) => compile_expression(stack_frame, block, expression),
        Statement::Definition(_definition) => compile_literal(block, &Literal::Integer(0)),
    }
}

fn compile_expression(
    stack_frame: &StackFrame,
    block: &mut ir::Block,
    expression: &Expression,
) -> CompileResult {
    match expression {
        Expression::Identifier(identifier) => compile_identifier(stack_frame, block, identifier),
        Expression::FunctionCall(identifier, args) => {
            compile_function_call(stack_frame, block, identifier, args)
        }
        Expression::Literal(literal) => compile_literal(block, literal),
        Expression::BinaryExpression(lhs, operator, rhs) => {
            compile_binary_operator_expression(stack_frame, block, lhs, operator, rhs)
        }
    }
}

fn compile_identifier(
    stack_frame: &StackFrame,
    block: &mut ir::Block,
    identifier: &Identifier,
) -> Result<ir::Slot, CompileError> {
    match stack_frame.resolve(identifier) {
        Some(symbol) => match symbol {
            Symbol::Argument(index) => {
                let slot = block.push(ir::Opcode::FunctionArgument(*index));
                Ok(slot)
            }
            Symbol::Function(_function, _arity) => todo!("compile function identifier"),
        },
        None => unimplemented!("undefined symbol"),
    }
}

pub fn compile_binary_operator_expression(
    stack_frame: &StackFrame,
    block: &mut ir::Block,
    lhs: &Expression,
    operator: &BinaryOperator,
    rhs: &Expression,
) -> CompileResult {
    let lhs_slot = compile_expression(stack_frame, block, lhs)?;
    let rhs_slot = compile_expression(stack_frame, block, rhs)?;

    let operator = match operator {
        BinaryOperator::Add => ir::BinaryOperator::Add,
        BinaryOperator::Multiply => ir::BinaryOperator::Multiply,
    };

    let slot = block.push(ir::Opcode::BinaryOperator(lhs_slot, operator, rhs_slot));
    Ok(slot)
}

fn compile_function_call(
    stack_frame: &StackFrame,
    block: &mut ir::Block,
    identifier: &Identifier,
    args: &Vec<Expression>,
) -> CompileResult {
    let mut argument_slots = Vec::with_capacity(args.len());
    for arg in args.iter() {
        let argument_slot = compile_expression(stack_frame, block, arg)?;
        argument_slots.push(argument_slot);
    }

    let Some(identifier_symbol) = stack_frame.resolve(&identifier) else {
                  return Err(CompileError::UnresolvedSymbol(identifier.clone()));
              };

    let Symbol::Function(function, arity) = identifier_symbol else {
                  return Err(CompileError::NotImplemented(format!("calling symbol {:?}", identifier_symbol)));
              };

    if argument_slots.len() != *arity {
        return Err(CompileError::IncorrectArity(
            identifier.clone(),
            argument_slots.len(),
            *arity,
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
