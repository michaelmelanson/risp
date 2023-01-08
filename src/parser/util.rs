use std::ops::{RangeFrom, RangeTo};

use nom::{
    character::complete::{char, multispace0},
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

pub fn ignore_whitespace<'a, O: std::fmt::Debug + std::cmp::PartialEq>(
    mut parser: impl FnMut(Span<'a>) -> IResult<Span<'a>, O, (Span<'a>, nom::error::ErrorKind)>,
) -> impl FnMut(Span<'a>) -> ParseResult<'a, O> {
    move |input| {
        let (input, _) = multispace0(input)?;
        let (input, position) = position(input)?;
        let (input, value) = parser(input)?;
        Ok((input, Token { position, value }))
    }
}
