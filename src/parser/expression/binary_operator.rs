use nom::{branch::alt, multi::fold_many0, sequence::tuple};

use crate::parser::{
    tokens::{add_token, divide_token, multiply_token, subtract_token},
    ParseResult, Span, Token,
};

use super::{parse_factor_expression, Expression};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic operators
    Add,
    Subtract,
    Multiply,
    Divide,

    // Comparison operators
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::LessOrEqual => write!(f, "<="),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::GreaterOrEqual => write!(f, ">="),
        }
    }
}

pub fn parse_binary_operator_expression(input: Span) -> ParseResult<Expression> {
    let (input, lhs) = parse_expression_term(input)?;
    let (input, value) = fold_many0(
        tuple((alt((add_token, subtract_token)), parse_expression_term)),
        || lhs.clone(),
        |acc, (operator, rhs)| {
            let operator = match operator.value.as_str() {
                "+" => BinaryOperator::Add,
                "-" => BinaryOperator::Subtract,
                _ => unreachable!("unknown operator {}", operator.value),
            };
            Token {
                position: acc.position,
                value: Expression::BinaryExpression(
                    Box::new(acc.value),
                    operator,
                    Box::new(rhs.value),
                ),
            }
        },
    )(input)?;

    println!("Binary expression: {:?}", value);
    Ok((input, value))
}

pub fn parse_expression_term(input: Span) -> ParseResult<Expression> {
    let (input, lhs) = parse_factor_expression(input)?;
    let (input, value) = fold_many0(
        tuple((alt((multiply_token, divide_token)), parse_factor_expression)),
        || lhs.clone(),
        |acc, (operator, rhs)| {
            let operator = match operator.value.as_str() {
                "*" => BinaryOperator::Multiply,
                "/" => BinaryOperator::Divide,
                _ => unreachable!("unknown operator {}", operator.value),
            };

            Token {
                position: acc.position,
                value: Expression::BinaryExpression(
                    Box::new(acc.value),
                    operator,
                    Box::new(rhs.value),
                ),
            }
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
    fn test_multiplication_division() {
        parse_test(parse_binary_operator_expression, "2 * 3 / 4", |input| {
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
                        BinaryOperator::Divide,
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
