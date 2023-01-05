use nom::{character::complete::space0, multi::separated_list0};
use nom_locate::position;

use crate::parser::{
    parse_block, parse_identifier,
    tokens::{comma, def},
    util::bracketed,
    Block, Identifier, ParseResult, Span, Token,
};

#[cfg(test)]
use crate::tests::parse_test;

use super::Statement;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub args: Vec<Identifier>,
    pub body: Block,
}

pub fn parse_function_definition_statement(input: Span) -> ParseResult<Statement> {
    let (input, function_definition) = parse_function_definition(input)?;

    Ok((
        input,
        Token {
            position: function_definition.position,
            value: Statement::Definition(function_definition.value),
        },
    ))
}

pub fn parse_function_definition(input: Span) -> ParseResult<FunctionDefinition> {
    let (input, def_token) = def(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, args) = parse_arguments_list(input)?;
    let (input, body) = parse_block(input)?;

    Ok((
        input,
        Token {
            position: def_token.position,
            value: FunctionDefinition {
                name: name.value,
                args: args.value,
                body: body.value,
            },
        },
    ))
}

fn parse_arguments_list(input: Span) -> ParseResult<Vec<Identifier>> {
    let (input, _) = space0(input)?;
    let (input, position) = position(input)?;
    let (input, value) = bracketed(separated_list0(comma, parse_identifier))(input)?;

    let value = value.iter().map(|t| t.value.clone()).collect();
    Ok((input, Token { position, value }))
}

#[test]
fn test_function_definition() {
    use crate::parser::{BinaryOperator, Expression, Literal, Statement};
    use nom::Slice;

    parse_test(
        parse_function_definition,
        "def add_one(x) { 1 + x }",
        |input| {
            (
                input.slice(24..),
                Token {
                    position: input.slice(0..0),
                    value: FunctionDefinition {
                        name: Identifier("add_one".to_string()),
                        args: vec![Identifier("x".to_string())],
                        body: Block(vec![Statement::Expression(Expression::BinaryExpression(
                            Box::new(Expression::Literal(Literal::Integer(1))),
                            BinaryOperator::Add,
                            Box::new(Expression::Identifier(Identifier("x".to_string()))),
                        ))]),
                    },
                },
            )
        },
    )
}
