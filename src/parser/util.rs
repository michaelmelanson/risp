use std::ops::{RangeFrom, RangeTo};

use nom::{
    character::complete::{char, space0},
    error::ParseError,
    sequence::{delimited, preceded, terminated},
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
        let (input, position) = position(input)?;
        let (input, value) = delimited(space0, char(c), space0)(input)?;
        Ok((input, Token { position, value }))
    }
}
