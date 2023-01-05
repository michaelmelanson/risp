use crate::parser::{parse_expression, ParseResult, Span, Token};

use super::Statement;

pub fn parse_expression_statement(input: Span) -> ParseResult<Statement> {
    let (input, expression) = parse_expression(input)?;

    Ok((
        input,
        Token {
            position: expression.position,
            value: Statement::Expression(expression.value),
        },
    ))
}
