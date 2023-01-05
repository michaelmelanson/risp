use std::ops::{RangeFrom, RangeTo};

use nom::{
    bytes::complete::tag,
    character::complete::{char, space0},
    error::ParseError,
    sequence::{preceded, terminated},
    AsChar, IResult, InputIter, Offset, Slice,
};
use nom_locate::position;

use super::{ParseResult, Span, Token};

pub fn bracketed<I, O, E: ParseError<I>, F>(f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: FnMut(I) -> IResult<I, O, E>,
    I: InputIter,
    I: Slice<RangeFrom<usize>>,
    I: Slice<RangeTo<usize>>,
    <I as InputIter>::Item: AsChar,
    I: Offset,
    I: std::clone::Clone,
{
    preceded(char('('), terminated(f, char(')')))
}

pub fn token(c: char) -> impl FnMut(Span) -> ParseResult<char> {
    move |input| {
        let (input, token) = ignore_whitespace(char(c))(input)?;
        Ok((
            input,
            Token {
                position: token.position,
                value: token.value,
            },
        ))
    }
}

pub fn keyword(k: &str) -> impl FnMut(Span) -> ParseResult<Span> + '_ {
    move |input| {
        let (input, token) = ignore_whitespace(tag(k))(input)?;
        Ok((
            input,
            Token {
                position: token.position,
                value: token.value,
            },
        ))
    }
}

pub fn ignore_whitespace<'a, O: std::fmt::Debug + std::cmp::PartialEq>(
    mut parser: impl FnMut(Span<'a>) -> IResult<Span<'a>, O, (Span<'a>, nom::error::ErrorKind)>,
) -> impl FnMut(Span<'a>) -> ParseResult<'a, O> {
    move |input| {
        let (input, _) = space0(input)?;
        let (input, position) = position(input)?;
        let (input, value) = parser(input)?;
        let (input, _) = space0(input)?;
        Ok((input, Token { position, value }))
    }
}
