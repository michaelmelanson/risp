use super::{util::ignore_whitespace, ParseResult, Span, Token};
use nom::bytes::complete::tag;

fn token<'a>(c: &'a str) -> impl FnMut(Span<'a>) -> ParseResult<String> {
    move |input| {
        let (input, token) = ignore_whitespace(tag(c))(input)?;
        Ok((
            input,
            Token {
                position: token.position,
                value: token.value.to_string(),
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

pub fn comma_token(input: Span<'_>) -> ParseResult<'_, String> {
    token(",")(input)
}

pub fn open_brace_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("{")(input)
}

pub fn close_brace_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("}")(input)
}

pub fn add_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("+")(input)
}

pub fn subtract_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("-")(input)
}

pub fn multiply_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("*")(input)
}

pub fn divide_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("/")(input)
}

pub fn equality_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("==")(input)
}

pub fn inequality_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("!=")(input)
}

pub fn less_than_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("<")(input)
}

pub fn less_or_equal_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("<=")(input)
}

pub fn greater_than_token(input: Span<'_>) -> ParseResult<'_, String> {
    token(">")(input)
}

pub fn greater_or_equal_token(input: Span<'_>) -> ParseResult<'_, String> {
    token(">=")(input)
}

pub fn assignment_token(input: Span<'_>) -> ParseResult<'_, String> {
    token("=")(input)
}

pub fn def_keyword(input: Span<'_>) -> ParseResult<'_, Span<'_>> {
    keyword("def")(input)
}

pub fn let_keyword(input: Span<'_>) -> ParseResult<'_, Span<'_>> {
    keyword("let")(input)
}

pub fn if_keyword(input: Span<'_>) -> ParseResult<'_, Span<'_>> {
    keyword("if")(input)
}

pub fn else_keyword(input: Span<'_>) -> ParseResult<'_, Span<'_>> {
    keyword("else")(input)
}

pub fn return_keyword(input: Span<'_>) -> ParseResult<'_, Span<'_>> {
    keyword("return")(input)
}
