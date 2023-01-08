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

pub fn comma_token<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token(',')(input)
}

pub fn open_brace_token<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('{')(input)
}

pub fn close_brace_token<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('}')(input)
}

pub fn add_token<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('+')(input)
}

pub fn multiply_token<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('*')(input)
}

pub fn equal_token<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('=')(input)
}

pub fn def_keyword<'a>(input: Span<'a>) -> ParseResult<'a, Span<'a>> {
    keyword("def")(input)
}

pub fn let_keyword<'a>(input: Span<'a>) -> ParseResult<'a, Span<'a>> {
    keyword("let")(input)
}
