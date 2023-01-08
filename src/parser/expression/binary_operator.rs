use nom::{multi::fold_many0, sequence::preceded};

use crate::parser::{
    tokens::{add_token, multiply_token},
    ParseResult, Span, Token,
};

use super::{parse_factor_expression, Expression};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Multiply,
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Multiply => write!(f, "*"),
        }
    }
}

pub fn parse_binary_operator_expression(input: Span) -> ParseResult<Expression> {
    let (input, lhs) = parse_expression_term(input)?;
    let (input, value) = fold_many0(
        preceded(add_token, parse_expression_term),
        || lhs.clone(),
        |acc, rhs| Token {
            position: acc.position,
            value: Expression::BinaryExpression(
                Box::new(acc.value),
                BinaryOperator::Add,
                Box::new(rhs.value),
            ),
        },
    )(input)?;

    println!("Binary expression: {:?}", value);
    Ok((input, value))
}

pub fn parse_expression_term(input: Span) -> ParseResult<Expression> {
    let (input, lhs) = parse_factor_expression(input)?;
    let (input, value) = fold_many0(
        preceded(multiply_token, parse_factor_expression),
        || lhs.clone(),
        |acc, rhs| Token {
            position: acc.position,
            value: Expression::BinaryExpression(
                Box::new(acc.value),
                BinaryOperator::Multiply,
                Box::new(rhs.value),
            ),
        },
    )(input)?;

    println!("Expression term: {:?}", value);
    Ok((input, value))
}

#[cfg(test)]
mod tests {
    use nom::Slice;

    use crate::{
        parser::{Expression, Identifier, Literal, Token},
        tests::parse_test,
    };

    use super::{parse_binary_operator_expression, BinaryOperator};

    #[test]
    fn test_addition() {
        parse_test(parse_binary_operator_expression, "2 + 3 + 4", |input| {
            (
                input.slice(9..),
                Token {
                    position: input.slice(0..0),
                    value: Expression::BinaryExpression(
                        Box::new(Expression::BinaryExpression(
                            Box::new(Expression::Literal(Literal::Integer(2))),
                            BinaryOperator::Add,
                            Box::new(Expression::Literal(Literal::Integer(3))),
                        )),
                        BinaryOperator::Add,
                        Box::new(Expression::Literal(Literal::Integer(4))),
                    ),
                },
            )
        })
    }

    #[test]
    fn test_multiplication() {
        parse_test(parse_binary_operator_expression, "2 * 3 * 4", |input| {
            (
                input.slice(9..),
                Token {
                    position: input.slice(0..0),
                    value: Expression::BinaryExpression(
                        Box::new(Expression::BinaryExpression(
                            Box::new(Expression::Literal(Literal::Integer(2))),
                            BinaryOperator::Multiply,
                            Box::new(Expression::Literal(Literal::Integer(3))),
                        )),
                        BinaryOperator::Multiply,
                        Box::new(Expression::Literal(Literal::Integer(4))),
                    ),
                },
            )
        })
    }

    #[test]
    fn test_mixed_expression_1() {
        parse_test(parse_binary_operator_expression, "2 + 3*4", |input| {
            (
                input.slice(7..),
                Token {
                    position: input.slice(0..0),
                    value: Expression::BinaryExpression(
                        Box::new(Expression::Literal(Literal::Integer(2))),
                        BinaryOperator::Add,
                        Box::new(Expression::BinaryExpression(
                            Box::new(Expression::Literal(Literal::Integer(3))),
                            BinaryOperator::Multiply,
                            Box::new(Expression::Literal(Literal::Integer(4))),
                        )),
                    ),
                },
            )
        })
    }

    #[test]
    fn test_mixed_expression_2() {
        parse_test(parse_binary_operator_expression, "1*2 + 3*4", |input| {
            (
                input.slice(9..),
                Token {
                    position: input.slice(0..0),
                    value: Expression::BinaryExpression(
                        Box::new(Expression::BinaryExpression(
                            Box::new(Expression::Literal(Literal::Integer(1))),
                            BinaryOperator::Multiply,
                            Box::new(Expression::Literal(Literal::Integer(2))),
                        )),
                        BinaryOperator::Add,
                        Box::new(Expression::BinaryExpression(
                            Box::new(Expression::Literal(Literal::Integer(3))),
                            BinaryOperator::Multiply,
                            Box::new(Expression::Literal(Literal::Integer(4))),
                        )),
                    ),
                },
            )
        })
    }

    #[test]
    fn test_one_plus_x() {
        parse_test(parse_binary_operator_expression, "1 + x", |input| {
            (
                input.slice(5..),
                Token {
                    position: input.slice(0..0),
                    value: Expression::BinaryExpression(
                        Box::new(Expression::Literal(Literal::Integer(1))),
                        BinaryOperator::Add,
                        Box::new(Expression::Identifier(Identifier("x".to_string()))),
                    ),
                },
            )
        })
    }
}
