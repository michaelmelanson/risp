use crate::parser::{parse_literal, ParseResult, Span, Token};

use super::Expression;

pub fn parse_literal_expression(input: Span) -> ParseResult<Expression> {
    let (input, literal) = parse_literal(input)?;

    Ok((
        input,
        Token {
            position: literal.position,
            value: Expression::Literal(literal.value),
        },
    ))
}
