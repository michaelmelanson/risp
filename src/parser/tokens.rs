use super::{util::token, ParseResult, Span};

pub fn newline<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('\n')(input)
}

pub fn comma<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token(',')(input)
}

pub fn open_brace<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('{')(input)
}

pub fn close_brace<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('}')(input)
}

pub fn add<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('+')(input)
}

pub fn multiply<'a>(input: Span<'a>) -> ParseResult<'a, char> {
    token('*')(input)
}
