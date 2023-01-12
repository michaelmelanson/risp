use crate::parser::{
    parse_block, parse_expression, tokens::if_keyword, Block, Expression, ParseResult, Span, Token,
};

use super::Statement;

#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    // an if x { 1 } else if y { 2 } else { 3 } statement would have three
    // branches:
    //  - (Some(x), 1)
    //  - (Some(y), 2)
    //  - (None, 3)
    pub branches: Vec<(Option<Expression>, Block)>,
}

pub fn parse_condition_statement(input: Span) -> ParseResult<Statement> {
    let (input, condition) = parse_condition(input)?;

    Ok((
        input,
        Token {
            position: condition.position,
            value: Statement::Condition(condition.value),
        },
    ))
}

fn parse_condition(input: Span) -> ParseResult<Condition> {
    let mut branches = Vec::new();

    let (input, if_keyword) = if_keyword(input)?;
    let (input, if_predicate) = parse_expression(input)?;
    let (input, if_block) = parse_block(input)?;
    branches.push((Some(if_predicate.value), if_block.value));

    Ok((
        input,
        Token {
            position: if_keyword.position,
            value: Condition { branches },
        },
    ))
}
