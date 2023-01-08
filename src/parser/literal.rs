use nom::{
    branch::alt,
    bytes::complete::escaped,
    character::complete::{char, digit1, multispace0, one_of, space0},
    combinator::{map, map_res},
    sequence::delimited,
};

// we use this but Rust Analyzer doesn't notice it...?
#[allow(unused_imports)]
use nom::Slice;

use nom_locate::position;

use super::{ParseResult, Span, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    String(String),
    Integer(i64),
}

fn literal_string(input: Span) -> ParseResult<Literal> {
    let (input, _) = space0(input)?;
    let (input, position) = position(input)?;
    let (input, chars) = delimited(
        char('"'),
        escaped(
            one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890 !@#$%^&*()"),
            '\\',
            one_of("\"n\\"),
        ),
        char('"'),
    )(input)?;

    let value = Literal::String(String::from(*chars));
    Ok((input, Token { position, value }))
}

fn parse_int(input: Span) -> ParseResult<i64> {
    let (input, _) = multispace0(input)?;
    let (input, position) = position(input)?;
    let (input, value) = map_res(digit1, |s: Span| s.parse::<i64>())(input)?;
    Ok((input, Token { position, value }))
}

fn literal_int(input: Span) -> ParseResult<Literal> {
    let (input, _) = multispace0(input)?;
    let (input, position) = position(input)?;
    let (input, value) = map(parse_int, |i| Literal::Integer(i.value))(input)?;

    Ok((input, Token { position, value }))
}

pub fn parse_literal(input: Span) -> ParseResult<Literal> {
    alt((literal_string, literal_int))(input)
}

#[test]
fn test_literal() {
    let input = Span::new("  \"foo\"  ");
    assert_eq!(
        parse_literal(input),
        Ok((
            input.slice(7..),
            Token {
                position: input.slice(2..2),
                value: Literal::String("foo".to_owned())
            }
        ))
    );

    let input = Span::new("  1234");
    assert_eq!(
        parse_literal(input),
        Ok((
            input.slice(6..),
            Token {
                position: input.slice(2..2),
                value: Literal::Integer(1234)
            }
        ))
    );

    let input = Span::new("\"foo\"blah");
    assert_eq!(
        parse_literal(input),
        Ok((
            input.slice(5..),
            Token {
                position: input.slice(0..0),
                value: Literal::String("foo".to_owned())
            }
        ))
    );
}
