mod binary_operator;
mod function_call;
mod identifier;
mod literal;

use nom::branch::alt;

use self::{
    binary_operator::parse_binary_operator_expression,
    function_call::parse_function_call_expression, identifier::parse_identifier_expression,
    literal::parse_literal_expression,
};

pub use self::binary_operator::BinaryOperator;
use super::{util::bracketed, Identifier, Literal, ParseResult, Span};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    FunctionCall(Identifier, Vec<Expression>),
    Literal(Literal),
    BinaryExpression(Box<Expression>, BinaryOperator, Box<Expression>),
}

pub fn parse_expression(input: Span) -> ParseResult<Expression> {
    alt((parse_binary_operator_expression, parse_factor_expression))(input)
}

pub fn parse_factor_expression(input: Span) -> ParseResult<Expression> {
    alt((
        bracketed(parse_expression),
        parse_function_call_expression,
        parse_identifier_expression,
        parse_literal_expression,
    ))(input)
}
