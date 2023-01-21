use nom::branch::alt;

use crate::parser::{
    parse_block, parse_expression, tokens::while_keyword, Block, Expression, ParseResult, Span,
    Token,
};

use super::Statement;

#[derive(Clone, Debug, PartialEq)]
pub enum LoopPredicatePosition {
    // i.e, a "while" loop
    BeforeBlock,
    // i.e, a "do-while" loop
    //AfterBlock,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Loop {
    pub predicate_position: LoopPredicatePosition,
    pub predicate: Expression,
    pub block: Block,
}

pub fn parse_loop_statement(input: Span) -> ParseResult<Statement> {
    alt((parse_while_loop_statement,))(input)
}

pub fn parse_while_loop_statement(input: Span) -> ParseResult<Statement> {
    let (input, while_keyword) = while_keyword(input)?;
    let (input, predicate) = parse_expression(input)?;
    let (input, block) = parse_block(input)?;

    Ok((
        input,
        Token {
            position: while_keyword.position,
            value: Statement::Loop(Loop {
                predicate_position: LoopPredicatePosition::BeforeBlock,
                predicate: predicate.value,
                block: block.value,
            }),
        },
    ))
}
