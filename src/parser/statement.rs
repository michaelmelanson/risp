mod assignment;
mod condition;
mod expression;
mod function_definition;
mod loop_statement;
mod return_statement;
mod variable_declaration;

use nom::branch::alt;

use super::{Expression, ParseResult, Span};

use self::{
    assignment::parse_assignment_statement, condition::parse_condition_statement,
    expression::parse_expression_statement,
    function_definition::parse_function_definition_statement, loop_statement::parse_loop_statement,
    return_statement::parse_return_statement,
    variable_declaration::parse_variable_declaration_statement,
};

pub use self::{
    assignment::Assignment,
    condition::Condition,
    function_definition::{parse_function_definition, FunctionDefinition},
    loop_statement::{Loop, LoopPredicatePosition},
    variable_declaration::VariableDeclaration,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Expression(Expression),
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
    Condition(Condition),
    Return(Expression),
    Assignment(Assignment),
    Loop(Loop),
}

pub fn parse_statement(input: Span) -> ParseResult<Statement> {
    alt((
        parse_function_definition_statement,
        parse_variable_declaration_statement,
        parse_condition_statement,
        parse_loop_statement,
        parse_return_statement,
        parse_assignment_statement,
        parse_expression_statement,
    ))(input)
}
