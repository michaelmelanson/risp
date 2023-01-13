#[cfg(test)]
use nom::Slice;

use crate::parser::{
    parse_expression, parse_identifier,
    tokens::{assignment_token, let_keyword},
    Expression, Identifier, ParseResult, Span, Token,
};

#[cfg(test)]
use crate::{
    parser::{BinaryOperator, Literal},
    tests::parse_test,
};

use super::Statement;

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDeclaration {
    pub name: Identifier,
    pub value: Expression,
}

pub fn parse_variable_declaration(input: Span) -> ParseResult<VariableDeclaration> {
    let (input, let_token) = let_keyword(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = assignment_token(input)?;
    let (input, initial_value) = parse_expression(input)?;

    let value = VariableDeclaration {
        name: name.value,
        value: initial_value.value,
    };

    println!("Variable declaration: {:?}", value);
    Ok((
        input,
        Token {
            position: let_token.position,
            value,
        },
    ))
}

pub fn parse_variable_declaration_statement(input: Span) -> ParseResult<Statement> {
    let (input, declaration) = parse_variable_declaration(input)?;

    Ok((
        input,
        Token {
            position: declaration.position,
            value: Statement::VariableDeclaration(declaration.value),
        },
    ))
}

#[test]
fn test_parse_variable_declaration() {
    parse_test(parse_variable_declaration, "let x = 55 + 42", |input| {
        (
            input.slice(15..),
            Token {
                position: input.slice(0..0),
                value: VariableDeclaration {
                    name: Identifier::new("x"),
                    value: Expression::BinaryExpression(
                        Box::new(Expression::Literal(Literal::Integer(55))),
                        BinaryOperator::Add,
                        Box::new(Expression::Literal(Literal::Integer(42))),
                    ),
                },
            },
        )
    });
}

#[test]
fn test_parse_variable_declaration_2() {
    parse_test(parse_variable_declaration, "let result = x * x", |input| {
        (
            input.slice(18..),
            Token {
                position: input.slice(0..0),
                value: VariableDeclaration {
                    name: Identifier::new("result"),
                    value: Expression::BinaryExpression(
                        Box::new(Expression::Identifier(Identifier::new("x"))),
                        BinaryOperator::Multiply,
                        Box::new(Expression::Identifier(Identifier::new("x"))),
                    ),
                },
            },
        )
    });
}
