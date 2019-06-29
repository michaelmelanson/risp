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
  Err,
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
  assert_eq!(space("   "), Ok(("", "   ")));
  assert_eq!(space("\t(foo) "), Ok(("(foo) ", "\t")));
  assert_eq!(space("abc"), Err(Err::Error(("abc", ErrorKind::TakeWhile1))));
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier<'a>(&'a str);

fn identifier<'a>(input: &'a str) -> ParseResult<Identifier<'a>> {
  let chars = "abcdefghijklmnopqrstuvwxyz-_+*/!@#$%^&*<>=";

  map(
    take_while1(move |c| chars.contains(c)),
    Identifier
  )(input)
}

#[test]
fn test_identifier() {
  assert_eq!(identifier("+"), Ok(("", Identifier("+"))));
  assert_eq!(identifier("foo"), Ok(("", Identifier("foo"))));
  assert_eq!(identifier(" foo"), Err(Err::Error((" foo", ErrorKind::TakeWhile1))));
  assert_eq!(identifier("foo "), Ok((" ", Identifier("foo"))));
  assert_eq!(identifier("foo-bar"), Ok(("", Identifier("foo-bar"))));
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal<'a> {
  Str(&'a str),
  Int(i64)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  escaped(alphanumeric1, '\\', one_of("\"n\\"))(i)
}

fn literal_string<'a>(input: &'a str) -> ParseResult<Literal<'a>> {
  context("literal string",
    preceded(
      char('"'),
      cut(
        terminated(
          map(parse_str, |s| Literal::Str(s)),
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

fn literal_int<'a>(input: &'a str) -> ParseResult<Literal<'a>> {
  context("literal integer",
    map(parse_int, |i| Literal::Int(i))
  )(input)
}

fn literal<'a>(input: &'a str) -> ParseResult<Literal<'a>> {
  alt((
    literal_string,
    literal_int
  ))(input)
}

#[test]
fn test_literal() {
  assert_eq!(literal("\"foo\""), Ok(("", Literal::Str("foo"))));
  assert_eq!(literal("1234"), Ok(("", Literal::Int(1234))));
  assert_eq!(literal("\"foo\"blah"), Ok(("blah", Literal::Str("foo"))));
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator<'a> {
  Add,
  Multiply,
  CallFunction(Identifier<'a>)
}

impl <'a> Operator<'a> {
  pub fn from_identifier(i: Identifier<'a>) -> Operator<'a> {
    match i {
      Identifier("+") => Operator::Add,
      Identifier("*") => Operator::Multiply,
      _ => Operator::CallFunction(i.clone())
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Term<'a> {
  Identifier(Identifier<'a>),
  Literal(Literal<'a>),
  Expression(Operator<'a>, Vec<Term<'a>>)
}


fn expression_terms<'a>(input: &'a str) -> ParseResult<Term<'a>> {
  let (input, operator) = identifier(input)?;
  let (input, _) = space(input)?;
  let (input, args) = separated_list(space, term)(input)?;

  let operator = Operator::from_identifier(operator);

  Ok((input, Term::Expression(operator, args)))
}

fn term_expression<'a>(input: &'a str) -> ParseResult<Term<'a>> {
  context("expression",
    preceded(
      char('('),
      cut(
        terminated(expression_terms, char(')'))
      )
    )
  )(input)
}

pub fn term<'a>(input: &'a str) -> ParseResult<Term<'a>> {
  alt((
    map(identifier, |x| Term::Identifier(x)),
    map(literal, |x| Term::Literal(x)),
    term_expression
  ))(input)
}

#[test]
fn test_term() {
  assert_eq!(term("foo"), Ok(("", Term::Identifier(Identifier("foo")))));
  assert_eq!(term("123"), Ok(("", Term::Literal(Literal::Int(123)))));
  assert_eq!(term("\"blah\""), Ok(("", Term::Literal(Literal::Str("blah")))));
  assert_eq!(term("(+ 1 2)"), Ok(("", Term::Expression(Operator::Add, vec![
    Term::Literal(Literal::Int(1)),
    Term::Literal(Literal::Int(2)),
  ]))));
}
