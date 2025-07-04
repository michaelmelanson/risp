use nom::{multi::many1, sequence::delimited};
use nom_locate::position;

use super::{
    parse_statement,
    tokens::{close_brace_token, open_brace_token},
    ParseResult, Span, Statement, Token,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Block(pub Vec<Statement>);

pub fn parse_block(input: Span) -> ParseResult<Block> {
    delimited(open_brace_token, parse_block_inner, close_brace_token)(input)
}

pub fn parse_block_inner(input: Span) -> ParseResult<Block> {
    let (input, position) = position(input)?;
    let (input, statements) = many1(parse_statement)(input)?;

    let statements = statements.iter().map(|t| t.value.clone()).collect();
    Ok((
        input,
        Token {
            position,
            value: Block(statements),
        },
    ))
}
