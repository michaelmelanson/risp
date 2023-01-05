mod block;
mod expression;
mod identifier;
mod literal;
mod statement;
mod util;

use nom::{error::ErrorKind, IResult};

use std::str;

use crate::parser::block::parse_block_inner;

pub use self::{
    block::{parse_block, Block},
    expression::{parse_expression, BinaryOperator, Expression},
    identifier::{parse_identifier, Identifier},
    literal::{parse_literal, Literal},
    statement::{parse_statement, Statement},
};

pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;
pub(crate) type ParseResult<'a, O, E = (Span<'a>, ErrorKind), I = Span<'a>> =
    IResult<I, Token<'a, O>, E>;

#[derive(Clone, Debug, PartialEq)]
pub struct Token<'a, T>
where
    T: std::fmt::Debug + PartialEq,
{
    pub position: Span<'a>,
    pub value: T,
}

// fn term_factor(input: Span) -> ParseResult<Term> {
//     alt((
//         map(literal, |x| Token {
//             position: x.position,
//             value: Term::Literal(x.value),
//         }),
//         map(parse_identifier, |x| Token {
//             position: x.position,
//             value: Term::Identifier(x.value),
//         }),
//     ))(input)
// }

// fn expression_multiply(input: Span) -> ParseResult<Term> {
//     let (input, position) = position(input)?;
//     let (input, lhs) = term_factor(input)?;
//     let (input, _) = delimited(space0, tag("*"), space0)(input)?;
//     let (input, rhs) = term(input)?;

//     Ok((
//         input,
//         Token {
//             position,
//             value: Term::Expression(Operator::Multiply, vec![lhs.value, rhs.value]),
//         },
//     ))
// }

// fn expression_add(input: Span) -> ParseResult<Term> {
//     let (input, position) = position(input)?;
//     let (input, lhs) = term_factor(input)?;
//     let (input, _) = delimited(space0, tag("+"), space0)(input)?;
//     let (input, rhs) = term(input)?;

//     Ok((
//         input,
//         Token {
//             position,
//             value: Term::Expression(Operator::Add, vec![lhs.value, rhs.value]),
//         },
//     ))
// }

// // e.g.:
// //
// // def add_one (x) { 1 + x }
// //
// fn term_definition(input: Span) -> ParseResult<Term> {}

// fn term_function_call(input: Span) -> ParseResult<Term> {
//     let (input, _) = space0(input)?;
//     let (input, position) = position(input)?;
//     let (input, identifier) = parse_identifier(input)?;
//     let (input, _) = space0(input)?;
//     let (input, args) =
//         bracketed(separated_list0(delimited(space0, char(','), space0), term))(input)?;

//     let args = args.iter().map(|t| t.value.clone()).collect();
//     Ok((
//         input,
//         Token {
//             position,
//             value: Term::CallFunction(identifier.value, args),
//         },
//     ))
// }

// #[test]
// fn test_term_identifier() {
//     let input = Span::new("foo");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(3..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::Identifier(Identifier("foo".to_owned()))
//             }
//         ))
//     );

//     let input = Span::new("   foo");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(6..),
//             Token {
//                 position: input.slice(3..3),
//                 value: Term::Identifier(Identifier("foo".to_owned()))
//             }
//         ))
//     );
// }

// #[test]
// fn test_term_literal() {
//     let input = Span::new("123");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(3..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::Literal(Literal::Integer(123))
//             }
//         ))
//     );

//     let input = Span::new("\"blah blah\"");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(11..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::Literal(Literal::String("blah blah".to_owned()))
//             }
//         ))
//     );
// }

// #[test]
// fn test_term_expression() {
//     let input = Span::new("1 + 2");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(5..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::Expression(
//                     Operator::Add,
//                     vec![
//                         Term::Literal(Literal::Integer(1)),
//                         Term::Literal(Literal::Integer(2)),
//                     ]
//                 )
//             }
//         ))
//     );
// }

// #[test]
// pub fn test_nested_expressions_1() {
//     let input = Span::new("1+(2*3)");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(7..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::Expression(
//                     Operator::Add,
//                     vec![
//                         Term::Literal(Literal::Integer(1)),
//                         Term::Expression(
//                             Operator::Multiply,
//                             vec![
//                                 Term::Literal(Literal::Integer(2)),
//                                 Term::Literal(Literal::Integer(3))
//                             ],
//                         ),
//                     ]
//                 )
//             }
//         ))
//     );
// }

// #[test]
// pub fn test_nested_expressions_2() {
//     let input = Span::new("(2*3)+(3*4)");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(11..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::Expression(
//                     Operator::Add,
//                     vec![
//                         Term::Expression(
//                             Operator::Multiply,
//                             vec![
//                                 Term::Literal(Literal::Integer(2)),
//                                 Term::Literal(Literal::Integer(3))
//                             ],
//                         ),
//                         Term::Expression(
//                             Operator::Multiply,
//                             vec![
//                                 Term::Literal(Literal::Integer(3)),
//                                 Term::Literal(Literal::Integer(4))
//                             ],
//                         ),
//                     ]
//                 )
//             }
//         ))
//     );
// }

// #[test]
// pub fn test_term_definition() {
//     let input = Span::new("def add (a, b) { a + b }");
//     assert_eq!(
//         term_definition(input),
//         Ok((
//             input.slice(24..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::Definition(Definition {
//                     name: Identifier("add".to_owned()),
//                     args: vec![Identifier("a".to_owned()), Identifier("b".to_owned())],
//                     body: Box::new(Term::Expression(
//                         Operator::Add,
//                         vec![
//                             Term::Identifier(Identifier("a".to_owned())),
//                             Term::Identifier(Identifier("b".to_owned()))
//                         ]
//                     ))
//                 })
//             }
//         ))
//     );

//     let input = Span::new("def incr (x) { 1 + x }");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(22..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::Definition(Definition {
//                     name: Identifier("incr".to_owned()),
//                     args: vec![Identifier("x".to_owned())],
//                     body: Box::new(Term::Expression(
//                         Operator::Add,
//                         vec![
//                             Term::Literal(Literal::Integer(1)),
//                             Term::Identifier(Identifier("x".to_owned()))
//                         ]
//                     ))
//                 })
//             }
//         ))
//     );
// }

// #[test]
// fn test_term_function_call() {
//     let input = Span::new("some_function(1)");
//     assert_eq!(
//         term(input),
//         Ok((
//             input.slice(16..),
//             Token {
//                 position: input.slice(0..0),
//                 value: Term::CallFunction(
//                     Identifier("some_function".to_owned()),
//                     vec![Term::Literal(Literal::Integer(1))]
//                 )
//             }
//         ))
//     );
// }

pub fn parse(line: &str) -> ParseResult<Block> {
    println!("Parsing: {}", line);
    parse_block_inner(Span::new(line))
}
