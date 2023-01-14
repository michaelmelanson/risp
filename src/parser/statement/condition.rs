use nom::{combinator::opt, multi::many0};

use crate::parser::{
    parse_block, parse_expression,
    tokens::{else_keyword, if_keyword},
    Block, Expression, ParseResult, Span, Token,
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

    let (input, else_if_branches) = many0(parse_else_if_branch)(input)?;
    for branch in else_if_branches {
        branches.push(branch.value);
    }

    let (input, else_branch) = opt(parse_else_branch)(input)?;
    if let Some(else_branch) = else_branch {
        branches.push(else_branch.value);
    }

    Ok((
        input,
        Token {
            position: if_keyword.position,
            value: Condition { branches },
        },
    ))
}

fn parse_else_if_branch(input: Span) -> ParseResult<(Option<Expression>, Block)> {
    let (input, else_keyword) = else_keyword(input)?;
    let (input, _if_keyword) = if_keyword(input)?;
    let (input, predicate) = parse_expression(input)?;
    let (input, block) = parse_block(input)?;

    Ok((
        input,
        Token {
            position: else_keyword.position,
            value: (Some(predicate.value), block.value),
        },
    ))
}

fn parse_else_branch(input: Span) -> ParseResult<(Option<Expression>, Block)> {
    let (input, else_keyword) = else_keyword(input)?;
    let (input, block) = parse_block(input)?;

    Ok((
        input,
        Token {
            position: else_keyword.position,
            value: (None, block.value),
        },
    ))
}
