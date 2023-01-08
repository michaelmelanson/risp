mod expression;
mod function_definition;
mod variable_declaration;

use nom::branch::alt;

use super::{expression::Expression, ParseResult, Span};

use self::{
    expression::parse_expression_statement,
    function_definition::parse_function_definition_statement,
    variable_declaration::parse_variable_declaration_statement,
};
pub use self::{
    function_definition::{parse_function_definition, FunctionDefinition},
    variable_declaration::VariableDeclaration,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    FunctionDefinition(FunctionDefinition),
    Expression(Expression),
    VariableDeclaration(VariableDeclaration),
}

pub fn parse_statement(input: Span) -> ParseResult<Statement> {
    alt((
        parse_function_definition_statement,
        parse_variable_declaration_statement,
        parse_expression_statement,
    ))(input)
}
