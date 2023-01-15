use crate::parser::{
    parse_expression, parse_identifier, tokens::assignment_token, Expression, Identifier,
    ParseResult, Span, Token,
};

use super::Statement;

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub lhs: Identifier,
    pub rhs: Expression,
}

pub fn parse_assignment_statement(input: Span) -> ParseResult<Statement> {
    let (input, lhs) = parse_identifier(input)?;
    let (input, _) = assignment_token(input)?;
    let (input, rhs) = parse_expression(input)?;

    Ok((
        input,
        Token {
            position: lhs.position,
            value: Statement::Assignment(Assignment {
                lhs: lhs.value,
                rhs: rhs.value,
            }),
        },
    ))
}
