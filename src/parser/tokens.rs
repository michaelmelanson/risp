use super::{util::ignore_whitespace, ParseResult, Span, Token};
use nom::{bytes::complete::tag, character::complete::char};

fn token(c: char) -> impl FnMut(Span) -> ParseResult<char> {
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

fn keyword(k: &str) -> impl FnMut(Span) -> ParseResult<Span> + '_ {
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

pub fn comma_token(input: Span<'_>) -> ParseResult<'_, char> {
    token(',')(input)
}

pub fn open_brace_token(input: Span<'_>) -> ParseResult<'_, char> {
    token('{')(input)
}

pub fn close_brace_token(input: Span<'_>) -> ParseResult<'_, char> {
    token('}')(input)
}

pub fn add_token(input: Span<'_>) -> ParseResult<'_, char> {
    token('+')(input)
}

pub fn multiply_token(input: Span<'_>) -> ParseResult<'_, char> {
    token('*')(input)
}

pub fn equal_token(input: Span<'_>) -> ParseResult<'_, char> {
    token('=')(input)
}

pub fn def_keyword(input: Span<'_>) -> ParseResult<'_, Span<'_>> {
    keyword("def")(input)
}

pub fn let_keyword(input: Span<'_>) -> ParseResult<'_, Span<'_>> {
    keyword("let")(input)
}
