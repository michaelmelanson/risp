mod block;
mod expression;
mod identifier;
mod literal;
mod statement;
mod tokens;
mod util;

use crate::parser::block::parse_block_inner;

pub use self::{
    block::{parse_block, Block},
    expression::{parse_expression, BinaryOperator, Expression},
    identifier::{parse_identifier, Identifier},
    literal::{parse_literal, Literal},
    statement::{parse_statement, Statement},
};

pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;
pub type ParseResult<'a, O, E = (Span<'a>, nom::error::ErrorKind), I = Span<'a>> =
    nom::IResult<I, Token<'a, O>, E>;

#[derive(Clone, Debug, PartialEq)]
pub struct Token<'a, T>
where
    T: std::fmt::Debug + PartialEq,
{
    pub position: Span<'a>,
    pub value: T,
}

pub fn parse(line: &str) -> ParseResult<Block> {
    // println!("Parsing: {}", line);
    parse_block_inner(Span::new(line))
}
