use nom::multi::separated_list0;
use nom_locate::position;

use crate::parser::{
    parse_identifier,
    tokens::comma,
    util::{bracketed, token},
    ParseResult, Span, Token,
};

use super::{parse_expression, Expression};

pub fn parse_function_call_expression(input: Span) -> ParseResult<Expression> {
    let (input, position) = position(input)?;
    let (input, identifier) = parse_identifier(input)?;
    let (input, args) = bracketed(separated_list0(comma, parse_expression))(input)?;

    let args = args.iter().map(|token| token.value.clone()).collect();
    Ok((
        input,
        Token {
            position,
            value: Expression::FunctionCall(identifier.value, args),
        },
    ))
}
