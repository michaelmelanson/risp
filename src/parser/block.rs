use nom::{
    character::complete::{char, space0},
    multi::separated_list0,
    sequence::delimited,
};
use nom_locate::position;

use super::{parse_statement, util::token, ParseResult, Span, Statement, Token};

#[derive(Clone, Debug, PartialEq)]
pub struct Block(pub Vec<Statement>);

pub fn parse_block(input: Span) -> ParseResult<Block> {
    delimited(
        delimited(space0, char('{'), space0),
        parse_block_inner,
        delimited(space0, char('}'), space0),
    )(input)
}

pub fn parse_block_inner(input: Span) -> ParseResult<Block> {
    let (input, position) = position(input)?;
    let (input, statements) = separated_list0(token('\n'), parse_statement)(input)?;

    let statements = statements.iter().map(|t| t.value.clone()).collect();
    Ok((
        input,
        Token {
            position,
            value: Block(statements),
        },
    ))
}
