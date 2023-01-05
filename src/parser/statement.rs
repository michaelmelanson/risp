mod expression;
mod function_definition;

use nom::branch::alt;

use super::{expression::Expression, ParseResult, Span};

pub use self::function_definition::{parse_function_definition, FunctionDefinition};
use self::{
    expression::parse_expression_statement,
    function_definition::parse_function_definition_statement,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Definition(FunctionDefinition),
    Expression(Expression),
}

pub fn parse_statement(input: Span) -> ParseResult<Statement> {
    alt((
        parse_function_definition_statement,
        parse_expression_statement,
    ))(input)
}
