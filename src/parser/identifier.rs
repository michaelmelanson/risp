use nom::{
    character::complete::multispace0,
    combinator::fail,
    error::{ErrorKind, ParseError},
    AsChar, IResult, InputTakeAtPosition,
};

// we use this but Rust Analyzer doesn't notice it...?
#[allow(unused_imports)]
use nom::Slice;

use nom_locate::position;

use super::{ParseResult, Span, Token};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Identifier(pub String);
impl Identifier {
    #[allow(unused)]
    pub(crate) fn new(name: impl ToString) -> Identifier {
        Self(name.to_string())
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(&self.0)
    }
}

pub fn identifier_name<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position1_complete(
        |item| {
            let item = item.as_char();
            !item.is_alphanum() && item.as_char() != '_'
        },
        ErrorKind::AlphaNumeric,
    )
}

pub fn parse_identifier(input: Span) -> ParseResult<Identifier> {
    let (input, _) = multispace0(input)?;
    let (before_token_input, position) = position(input)?;
    let (input, value) = identifier_name(before_token_input)?;

    let (input, _) = match *value.fragment() {
        "def" | "let" | "if" | "else" => fail(before_token_input)?,
        _ => (input, ()),
    };

    println!("Identifier: {:?}", value);
    Ok((
        input,
        Token {
            position,
            value: Identifier(value.to_string()),
        },
    ))
}

#[test]
fn test_identifier() {
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

    let input = Span::new("foo_bar");
    assert_eq!(
        parse_identifier(input),
        Ok((
            input.slice(7..),
            Token {
                position: input.slice(0..0),
                value: Identifier("foo_bar".to_owned())
            }
        ))
    );
}
