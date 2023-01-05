use crate::parser::{parse_identifier, ParseResult, Span, Token};

use super::Expression;

pub fn parse_identifier_expression(input: Span) -> ParseResult<Expression> {
    let (input, identifier) = parse_identifier(input)?;

    Ok((
        input,
        Token {
            position: identifier.position,
            value: Expression::Identifier(identifier.value),
        },
    ))
}
