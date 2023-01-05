use nom::{
    character::complete::{one_of, space0},
    combinator::map,
    multi::many1,
};

// we use this but Rust Analyzer doesn't notice it...?
#[allow(unused_imports)]
use nom::Slice;

use nom_locate::position;

use super::{ParseResult, Span, Token};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Identifier(pub String);

impl std::fmt::Display for Identifier {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(&self.0)
    }
}

pub fn parse_identifier(input: Span) -> ParseResult<Identifier> {
    let (input, _) = space0(input)?;
    let chars = "abcdefghijklmnopqrstuvwxyz-_+*/!@#$%^&*<>=";
    let (input, position) = position(input)?;
    let (input, value) = map(many1(one_of(chars)), move |v| {
        Identifier(v.into_iter().collect())
    })(input)?;

    Ok((input, Token { position, value }))
}

#[test]
fn test_identifier() {
    let input = Span::new("+");
    assert_eq!(
        parse_identifier(input),
        Ok((
            input.slice(1..),
            Token {
                position: input.slice(0..0),
                value: Identifier("+".to_owned())
            }
        ))
    );

    let input = Span::new("foo");
    assert_eq!(
        parse_identifier(input),
        Ok((
            input.slice(3..),
            Token {
                position: input.slice(0..0),
                value: Identifier("foo".to_owned())
            }
        ))
    );

    let input = Span::new(" foo");
    assert_eq!(
        parse_identifier(input),
        Ok((
            input.slice(4..),
            Token {
                position: input.slice(1..1),
                value: Identifier("foo".to_owned())
            }
        ))
    );

    let input = Span::new("foo ");
    assert_eq!(
        parse_identifier(input),
        Ok((
            input.slice(3..),
            Token {
                position: input.slice(0..0),
                value: Identifier("foo".to_owned())
            }
        ))
    );

    let input = Span::new("foo-bar");
    assert_eq!(
        parse_identifier(input),
        Ok((
            input.slice(7..),
            Token {
                position: input.slice(0..0),
                value: Identifier("foo-bar".to_owned())
            }
        ))
    );
}
