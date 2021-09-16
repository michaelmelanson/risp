use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while1},
    character::complete::{alphanumeric1, char, digit1, one_of},
    combinator::{cut, map, map_res, opt},
    error::{context, ErrorKind, ParseError},
    multi::{separated_list0, separated_list1},
    sequence::{preceded, terminated},
    IResult,
};

use std::str;

type ParseResult<'a, O, E = (&'a str, ErrorKind), I = &'a str> = IResult<I, O, E>;

fn space(input: &str) -> ParseResult<&str> {
    let chars = " \t\n\r";

    take_while1(move |c| chars.contains(c))(input)
}

#[test]
fn test_space() {
    use nom::Err;
    assert_eq!(space("   "), Ok(("", "   ")));
    assert_eq!(space("\t(foo) "), Ok(("(foo) ", "\t")));
    assert_eq!(
        space("abc"),
        Err(Err::Error(("abc", ErrorKind::TakeWhile1)))
    );
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Identifier(String);

impl std::fmt::Display for Identifier {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(&self.0)
    }
}

fn identifier(input: &str) -> ParseResult<Identifier> {
    let chars = "abcdefghijklmnopqrstuvwxyz-_+*/!@#$%^&*<>=";

    map(take_while1(move |c| chars.contains(c)), |s: &str| {
        Identifier(s.to_owned())
    })(input)
}

#[test]
fn test_identifier() {
    use nom::Err;
    assert_eq!(identifier("+"), Ok(("", Identifier("+".to_owned()))));
    assert_eq!(identifier("foo"), Ok(("", Identifier("foo".to_owned()))));
    assert_eq!(
        identifier(" foo"),
        Err(Err::Error((" foo", ErrorKind::TakeWhile1)))
    );
    assert_eq!(identifier("foo "), Ok((" ", Identifier("foo".to_owned()))));
    assert_eq!(
        identifier("foo-bar"),
        Ok(("", Identifier("foo-bar".to_owned())))
    );
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(alphanumeric1, '\\', one_of("\"n\\"))(i)
}

fn literal_string(input: &str) -> ParseResult<Literal> {
    context(
        "literal string",
        preceded(
            char('"'),
            cut(terminated(
                map(parse_str, |s| Literal::String(String::from(s))),
                char('"'),
            )),
        ),
    )(input)
}

fn parse_int(input: &str) -> ParseResult<i64> {
    map_res(digit1, |s| i64::from_str_radix(s, 10))(input)
}

fn literal_int(input: &str) -> ParseResult<Literal> {
    context("literal integer", map(parse_int, |i| Literal::Integer(i)))(input)
}

fn literal(input: &str) -> ParseResult<Literal> {
    alt((literal_string, literal_int))(input)
}

#[test]
fn test_literal() {
    assert_eq!(
        literal("\"foo\""),
        Ok(("", Literal::String("foo".to_owned())))
    );
    assert_eq!(literal("1234"), Ok(("", Literal::Integer(1234))));
    assert_eq!(
        literal("\"foo\"blah"),
        Ok(("blah", Literal::String("foo".to_owned())))
    );
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Multiply,
    CallFunction(Identifier),
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
    Definition(Definition),
}

fn expression_terms(input: &str) -> ParseResult<Term> {
    let (input, parts) = separated_list1(space, term)(input)?;

    if let Some((Term::Identifier(operator), args)) = parts.split_first() {
        let operator = Operator::from_identifier(operator.clone());
        Ok((input, Term::Expression(operator, Vec::from(args))))
    } else {
        Err(nom::Err::Error((
            input,
            nom::error::ErrorKind::SeparatedNonEmptyList,
        )))
    }
}

pub fn bracketed<I, O, E: ParseError<I>, F>(f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: FnMut(I) -> IResult<I, O, E>,
    I: nom::InputIter,
    I: nom::Slice<std::ops::RangeFrom<usize>>,
    I: nom::Slice<std::ops::RangeTo<usize>>,
    <I as nom::InputIter>::Item: nom::AsChar,
    I: nom::Offset,
    I: std::clone::Clone,
{
    preceded(char('('), terminated(f, char(')')))
}

fn term_expression(input: &str) -> ParseResult<Term> {
    context("expression", bracketed(expression_terms))(input)
}

fn term_definition(input: &str) -> ParseResult<Term> {
    context("definition", bracketed(definition_inner))(input)
}

fn definition_inner(input: &str) -> ParseResult<Term> {
    let (input, _) = tag("def")(input)?;
    let (input, _) = space(input)?;
    let (input, name) = identifier(input)?;
    let (input, args) = opt(arguments_list)(input)?;
    let (input, _) = opt(space)(input)?;
    let (input, body) = term(input)?;

    let args = args.unwrap_or_default();
    let body = Box::new(body);

    Ok((input, Term::Definition(Definition { name, args, body })))
}

fn arguments_list(input: &str) -> ParseResult<ArgumentsList> {
    let (input, _) = opt(space)(input)?;
    context(
        "arguments list",
        bracketed(separated_list0(space, identifier)),
    )(input)
}

#[test]
pub fn test_term_definition() {
    assert_eq!(
        term_definition("(def add (a b) (+ a b))"),
        Ok((
            "",
            Term::Definition(Definition {
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
        ))
    );
}

pub fn term(input: &str) -> ParseResult<Term> {
    let (input, _) = opt(space)(input)?;

    alt((
        map(identifier, |x| Term::Identifier(x)),
        map(literal, |x| Term::Literal(x)),
        term_definition,
        term_expression,
    ))(input)
}

// fn term_function_call(input: &str) -> ParseResult<Term> {
//   bracketed(function_call_inner)(input)
// }

// fn function_call_inner(input: &str) -> ParseResult<Term> {
//   let (input, parts) = separated_nonempty_list(space, term)(input)?;
//   let (name, args) = parts.split_first().expect("split_first on nonempty list");

//   if let Term::Identifier(name) = name {
//     Ok((input, Term::Expression(
//       Operator::CallFunction(name.clone()),
//       Vec::from(args)
//     )))
//   } else {
//     Err(nom::Err::Error((input, nom::error::ErrorKind::Verify)))
//   }
// }

#[test]
fn test_term() {
    assert_eq!(
        term("foo"),
        Ok(("", Term::Identifier(Identifier("foo".to_owned()))))
    );
    assert_eq!(
        term("   foo"),
        Ok(("", Term::Identifier(Identifier("foo".to_owned()))))
    );
    assert_eq!(term("123"), Ok(("", Term::Literal(Literal::Integer(123)))));
    assert_eq!(
        term("\"blah\""),
        Ok(("", Term::Literal(Literal::String("blah".to_owned()))))
    );
    assert_eq!(
        term("(+ 1 2)"),
        Ok((
            "",
            Term::Expression(
                Operator::Add,
                vec![
                    Term::Literal(Literal::Integer(1)),
                    Term::Literal(Literal::Integer(2)),
                ]
            )
        ))
    );
    assert_eq!(
        term("(def incr (x) (+ 1 x))"),
        Ok((
            "",
            Term::Definition(Definition {
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
        ))
    );
    assert_eq!(
        term("(some-function)"),
        Ok((
            "",
            Term::Expression(
                Operator::CallFunction(Identifier("some-function".to_owned())),
                vec![]
            )
        ))
    );
}
