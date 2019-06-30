use nom::{
  branch::alt,
  bytes::complete::{
    escaped,
    take_while1
  },
  character::complete::{
    alphanumeric1,
    char,
    digit1,
    one_of
  },
  combinator::{
    cut, map, map_res
  },

  error::{context, ErrorKind, ParseError},
  IResult,
  multi::{
    separated_list
  },
  sequence::{
    preceded,
    terminated
  }
};

use std::str;

type ParseResult<'a, O, E = (&'a str, ErrorKind), I=&'a str> = IResult<I, O, E>;

fn space(input: &str) -> ParseResult<&str> {
  let chars = " \t\n\r";

  take_while1(move |c| chars.contains(c))(input)
}

#[test]
fn test_space() {
  use nom::Err;
  assert_eq!(space("   "), Ok(("", "   ")));
  assert_eq!(space("\t(foo) "), Ok(("(foo) ", "\t")));
  assert_eq!(space("abc"), Err(Err::Error(("abc", ErrorKind::TakeWhile1))));
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier(String);

fn identifier(input: &str) -> ParseResult<Identifier> {
  let chars = "abcdefghijklmnopqrstuvwxyz-_+*/!@#$%^&*<>=";

  map(
    take_while1(move |c| chars.contains(c)),
    |s: &str| Identifier(s.to_owned())
  )(input)
}

#[test]
fn test_identifier() {
  use nom::Err;
  assert_eq!(identifier("+"), Ok(("", Identifier("+".to_owned()))));
  assert_eq!(identifier("foo"), Ok(("", Identifier("foo".to_owned()))));
  assert_eq!(identifier(" foo"), Err(Err::Error((" foo", ErrorKind::TakeWhile1))));
  assert_eq!(identifier("foo "), Ok((" ", Identifier("foo".to_owned()))));
  assert_eq!(identifier("foo-bar"), Ok(("", Identifier("foo-bar".to_owned()))));
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
  Str(String),
  Int(i64)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  escaped(alphanumeric1, '\\', one_of("\"n\\"))(i)
}

fn literal_string(input: &str) -> ParseResult<Literal> {
  context("literal string",
    preceded(
      char('"'),
      cut(
        terminated(
          map(parse_str, |s| Literal::Str(String::from(s))),
          char('"')
        )
      )
    )
  )(input)
}

fn parse_int(input: &str) -> ParseResult<i64> {
  map_res(
    digit1, 
    |s| i64::from_str_radix(s, 10)
  )(input)
}

fn literal_int(input: &str) -> ParseResult<Literal> {
  context("literal integer",
    map(parse_int, |i| Literal::Int(i))
  )(input)
}

fn literal(input: &str) -> ParseResult<Literal> {
  alt((
    literal_string,
    literal_int
  ))(input)
}

#[test]
fn test_literal() {
  assert_eq!(literal("\"foo\""), Ok(("", Literal::Str("foo".to_owned()))));
  assert_eq!(literal("1234"), Ok(("", Literal::Int(1234))));
  assert_eq!(literal("\"foo\"blah"), Ok(("blah", Literal::Str("foo".to_owned()))));
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
  Add,
  Multiply,
  CallFunction(Identifier)
}

impl Operator {
  pub fn from_identifier(i: Identifier) -> Operator {
    if i.0 == "+".to_owned() {
      Operator::Add
    } else if i.0 == "*".to_owned() {
      Operator::Multiply
    } else {
      Operator::CallFunction(i.clone())
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Term {
  Identifier(Identifier),
  Literal(Literal),
  Expression(Operator, Vec<Term>)
}


fn expression_terms(input: &str) -> ParseResult<Term> {
  let (input, operator) = identifier(input)?;
  let (input, _) = space(input)?;
  let (input, args) = separated_list(space, term)(input)?;

  let operator = Operator::from_identifier(operator);

  Ok((input, Term::Expression(operator, args)))
}

fn term_expression(input: &str) -> ParseResult<Term> {
  context("expression",
    preceded(
      char('('),
      cut(
        terminated(expression_terms, char(')'))
      )
    )
  )(input)
}

pub fn term(input: &str) -> ParseResult<Term> {
  alt((
    map(identifier, |x| Term::Identifier(x)),
    map(literal, |x| Term::Literal(x)),
    term_expression
  ))(input)
}

#[test]
fn test_term() {
  assert_eq!(term("foo"), Ok(("", Term::Identifier(Identifier("foo".to_owned())))));
  assert_eq!(term("123"), Ok(("", Term::Literal(Literal::Int(123)))));
  assert_eq!(term("\"blah\""), Ok(("", Term::Literal(Literal::Str("blah".to_owned())))));
  assert_eq!(term("(+ 1 2)"), Ok(("", Term::Expression(Operator::Add, vec![
    Term::Literal(Literal::Int(1)),
    Term::Literal(Literal::Int(2)),
  ]))));
}
