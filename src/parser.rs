use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::{char, digit1, one_of, space0, space1},
    combinator::{map, map_res, opt},
    error::{ErrorKind, ParseError},
    multi::{many1, separated_list0},
    sequence::{delimited, preceded, terminated},
    IResult, Slice,
};
use nom_locate::position;

use std::{
    ops::{RangeFrom, RangeTo},
    str,
};

pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type ParseResult<'a, O, E = (Span<'a>, ErrorKind), I = Span<'a>> = IResult<I, Token<'a, O>, E>;

#[derive(Debug, PartialEq)]
pub struct Token<'a, T>
where
    T: std::fmt::Debug + PartialEq,
{
    pub position: Span<'a>,
    pub value: T,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Identifier(String);

impl std::fmt::Display for Identifier {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(&self.0)
    }
}

fn identifier(input: Span) -> ParseResult<Identifier> {
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
        identifier(input),
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
        identifier(input),
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
        identifier(input),
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
        identifier(input),
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
        identifier(input),
        Ok((
            input.slice(7..),
            Token {
                position: input.slice(0..0),
                value: Identifier("foo-bar".to_owned())
            }
        ))
    );
}

#[derive(Clone, Debug, PartialEq)]
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
    let (input, _) = space0(input)?;
    let (input, position) = position(input)?;
    let (input, value) = map_res(digit1, |s: Span| s.parse::<i64>())(input)?;
    Ok((input, Token { position, value }))
}

fn literal_int(input: Span) -> ParseResult<Literal> {
    let (input, _) = space0(input)?;
    let (input, position) = position(input)?;
    let (input, value) = map(parse_int, |i| Literal::Integer(i.value))(input)?;

    Ok((input, Token { position, value }))
}

fn literal(input: Span) -> ParseResult<Literal> {
    alt((literal_string, literal_int))(input)
}

#[test]
fn test_literal() {
    let input = Span::new("\"foo\"");
    assert_eq!(
        literal(input),
        Ok((
            input.slice(5..),
            Token {
                position: input.slice(0..0),
                value: Literal::String("foo".to_owned())
            }
        ))
    );

    let input = Span::new("1234");
    assert_eq!(
        literal(input),
        Ok((
            input.slice(4..),
            Token {
                position: input.slice(0..0),
                value: Literal::Integer(1234)
            }
        ))
    );

    let input = Span::new("\"foo\"blah");
    assert_eq!(
        literal(input),
        Ok((
            input.slice(5..),
            Token {
                position: input.slice(0..0),
                value: Literal::String("foo".to_owned())
            }
        ))
    );
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Multiply,
}

pub type ArgumentsList = Vec<Identifier>;

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    pub name: Identifier,
    pub args: ArgumentsList,
    pub body: Box<Term>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    Identifier(Identifier),
    Literal(Literal),
    Expression(Operator, Vec<Term>),
    CallFunction(Identifier, Vec<Term>),
    Definition(Definition),
}

pub fn bracketed<I, O, E: ParseError<I>, F>(f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: FnMut(I) -> IResult<I, O, E>,
    I: nom::InputIter,
    I: nom::Slice<RangeFrom<usize>>,
    I: nom::Slice<RangeTo<usize>>,
    <I as nom::InputIter>::Item: nom::AsChar,
    I: nom::Offset,
    I: std::clone::Clone,
{
    preceded(char('('), terminated(f, char(')')))
}

fn term_factor(input: Span) -> ParseResult<Term> {
    alt((
        map(literal, |x| Token {
            position: x.position,
            value: Term::Literal(x.value),
        }),
        map(identifier, |x| Token {
            position: x.position,
            value: Term::Identifier(x.value),
        }),
    ))(input)
}

fn expression_multiply(input: Span) -> ParseResult<Term> {
    let (input, position) = position(input)?;
    let (input, lhs) = term_factor(input)?;
    let (input, _) = delimited(space0, tag("*"), space0)(input)?;
    let (input, rhs) = term(input)?;

    Ok((
        input,
        Token {
            position,
            value: Term::Expression(Operator::Multiply, vec![lhs.value, rhs.value]),
        },
    ))
}

fn expression_add(input: Span) -> ParseResult<Term> {
    let (input, position) = position(input)?;
    let (input, lhs) = term_factor(input)?;
    let (input, _) = delimited(space0, tag("+"), space0)(input)?;
    let (input, rhs) = term(input)?;

    Ok((
        input,
        Token {
            position,
            value: Term::Expression(Operator::Add, vec![lhs.value, rhs.value]),
        },
    ))
}

// e.g.:
//
// def add_one (x) { 1 + x }
//
fn term_definition(input: Span) -> ParseResult<Term> {
    let (input, _) = space0(input)?;
    let (input, position) = position(input)?;
    let (input, _) = tag("def")(input)?;
    let (input, _) = space0(input)?;
    let (input, name) = identifier(input)?;
    let (input, args) = opt(arguments_list)(input)?;
    let (input, body) = delimited(
        delimited(space0, char('{'), space0),
        term,
        delimited(space0, char('}'), space0),
    )(input)?;

    let args = args.map(|t| t.value).unwrap_or_default();
    let body = Box::new(body.value);

    Ok((
        input,
        Token {
            position,
            value: Term::Definition(Definition {
                name: name.value,
                args,
                body,
            }),
        },
    ))
}

fn arguments_list(input: Span) -> ParseResult<ArgumentsList> {
    let (input, _) = space0(input)?;
    let (input, position) = position(input)?;
    let (input, value) = bracketed(separated_list0(
        delimited(space0, char(','), space0),
        identifier,
    ))(input)?;

    let value = value.iter().map(|t| t.value.clone()).collect();
    Ok((input, Token { position, value }))
}

pub fn term(input: Span) -> ParseResult<Term> {
    alt((
        bracketed(term),
        expression_multiply,
        expression_add,
        term_function_call,
        term_definition,
        term_factor,
    ))(input)
}

fn term_function_call(input: Span) -> ParseResult<Term> {
    let (input, position) = position(input)?;
    let (input, identifier) = identifier(input)?;

    let (input, _) = delimited(space0, char('('), space0)(input)?;
    let (input, args) = separated_list0(delimited(space0, char(','), space0), term)(input)?;
    let (input, _) = delimited(space0, char(')'), space0)(input)?;

    let args = args.iter().map(|t| t.value.clone()).collect();
    Ok((
        input,
        Token {
            position,
            value: Term::CallFunction(identifier.value, args),
        },
    ))
}

#[test]
fn test_term_identifier() {
    let input = Span::new("foo");
    assert_eq!(
        term(input),
        Ok((
            input.slice(3..),
            Token {
                position: input.slice(0..0),
                value: Term::Identifier(Identifier("foo".to_owned()))
            }
        ))
    );

    let input = Span::new("   foo");
    assert_eq!(
        term(input),
        Ok((
            input.slice(6..),
            Token {
                position: input.slice(3..3),
                value: Term::Identifier(Identifier("foo".to_owned()))
            }
        ))
    );
}

#[test]
fn test_term_literal() {
    let input = Span::new("123");
    assert_eq!(
        term(input),
        Ok((
            input.slice(3..),
            Token {
                position: input.slice(0..0),
                value: Term::Literal(Literal::Integer(123))
            }
        ))
    );

    let input = Span::new("\"blah blah\"");
    assert_eq!(
        term(input),
        Ok((
            input.slice(11..),
            Token {
                position: input.slice(0..0),
                value: Term::Literal(Literal::String("blah blah".to_owned()))
            }
        ))
    );
}

#[test]
fn test_term_expression() {
    let input = Span::new("1 + 2");
    assert_eq!(
        term(input),
        Ok((
            input.slice(5..),
            Token {
                position: input.slice(0..0),
                value: Term::Expression(
                    Operator::Add,
                    vec![
                        Term::Literal(Literal::Integer(1)),
                        Term::Literal(Literal::Integer(2)),
                    ]
                )
            }
        ))
    );
}

#[test]
pub fn test_nested_expressions_1() {
    let input = Span::new("1+(2*3)");
    assert_eq!(
        term(input),
        Ok((
            input.slice(7..),
            Token {
                position: input.slice(0..0),
                value: Term::Expression(
                    Operator::Add,
                    vec![
                        Term::Literal(Literal::Integer(1)),
                        Term::Expression(
                            Operator::Multiply,
                            vec![
                                Term::Literal(Literal::Integer(2)),
                                Term::Literal(Literal::Integer(3))
                            ],
                        ),
                    ]
                )
            }
        ))
    );
}
#[test]
pub fn test_nested_expressions_2() {
    let input = Span::new("(2*3)+(3*4)");
    assert_eq!(
        term(input),
        Ok((
            input.slice(11..),
            Token {
                position: input.slice(0..0),
                value: Term::Expression(
                    Operator::Add,
                    vec![
                        Term::Expression(
                            Operator::Multiply,
                            vec![
                                Term::Literal(Literal::Integer(2)),
                                Term::Literal(Literal::Integer(3))
                            ],
                        ),
                        Term::Expression(
                            Operator::Multiply,
                            vec![
                                Term::Literal(Literal::Integer(3)),
                                Term::Literal(Literal::Integer(4))
                            ],
                        ),
                    ]
                )
            }
        ))
    );
}

#[test]
pub fn test_term_definition() {
    let input = Span::new("def add (a, b) { a + b }");
    assert_eq!(
        term_definition(input),
        Ok((
            input.slice(24..),
            Token {
                position: input.slice(0..0),
                value: Term::Definition(Definition {
                    name: Identifier("add".to_owned()),
                    args: vec![Identifier("a".to_owned()), Identifier("b".to_owned())],
                    body: Box::new(Term::Expression(
                        Operator::Add,
                        vec![
                            Term::Identifier(Identifier("a".to_owned())),
                            Term::Identifier(Identifier("b".to_owned()))
                        ]
                    ))
                })
            }
        ))
    );

    let input = Span::new("def incr (x) { 1 + x }");
    assert_eq!(
        term(input),
        Ok((
            input.slice(22..),
            Token {
                position: input.slice(0..0),
                value: Term::Definition(Definition {
                    name: Identifier("incr".to_owned()),
                    args: vec![Identifier("x".to_owned())],
                    body: Box::new(Term::Expression(
                        Operator::Add,
                        vec![
                            Term::Literal(Literal::Integer(1)),
                            Term::Identifier(Identifier("x".to_owned()))
                        ]
                    ))
                })
            }
        ))
    );
}

#[test]
fn test_term_function_call() {
    let input = Span::new("some_function(1)");
    assert_eq!(
        term(input),
        Ok((
            input.slice(16..),
            Token {
                position: input.slice(0..0),
                value: Term::CallFunction(
                    Identifier("some_function".to_owned()),
                    vec![Term::Literal(Literal::Integer(1))]
                )
            }
        ))
    );
}

pub fn parse(line: &str) -> ParseResult<Term> {
    println!("Parsing: {}", line);
    term(Span::new(line))
}
