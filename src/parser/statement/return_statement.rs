use crate::parser::{parse_expression, tokens::return_keyword, ParseResult, Span, Token};

use super::Statement;

pub fn parse_return_statement(input: Span) -> ParseResult<Statement> {
    let (input, return_token) = return_keyword(input)?;
    let (input, result) = parse_expression(input)?;

    Ok((
        input,
        Token {
            position: return_token.position,
            value: Statement::Return(result.value),
        },
    ))
}
