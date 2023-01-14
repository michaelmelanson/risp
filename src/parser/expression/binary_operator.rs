use std::fmt::Debug;

use nom::{branch::alt, multi::fold_many0, sequence::tuple};

use crate::parser::{
    tokens::{
        add_token, divide_token, equality_token, greater_or_equal_token, greater_than_token,
        inequality_token, less_or_equal_token, less_than_token, multiply_token, subtract_token,
    },
    ParseResult, Span, Token,
};

use super::{parse_factor_expression, Expression};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    ArithmeticOperator(ArithmeticOperator),
    ComparisonOperator(ComparisonOperator),
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::ArithmeticOperator(op) => write!(f, "{}", op),
            BinaryOperator::ComparisonOperator(op) => write!(f, "{}", op),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl std::fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithmeticOperator::Add => write!(f, "+"),
            ArithmeticOperator::Subtract => write!(f, "-"),
            ArithmeticOperator::Multiply => write!(f, "*"),
            ArithmeticOperator::Divide => write!(f, "/"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl std::fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperator::Equal => write!(f, "=="),
            ComparisonOperator::NotEqual => write!(f, "!="),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::LessOrEqual => write!(f, "<="),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::GreaterOrEqual => write!(f, ">="),
        }
    }
}

pub fn parse_binary_operator_expression(input: Span) -> ParseResult<Expression> {
    let (input, expression) = parse_binary_operators(input, 1)?;
    println!("Binary expression: {:?}", expression);

    Ok((input, expression))
}

pub fn parse_binary_operators(input: Span, precedence: usize) -> ParseResult<Expression> {
    let token_parser = |input| match precedence {
        1 => alt((
            equality_token,
            inequality_token,
            less_or_equal_token,
            less_than_token,
            greater_or_equal_token,
            greater_than_token,
        ))(input),

        2 => alt((add_token, subtract_token))(input),
        3 => alt((multiply_token, divide_token))(input),
        _ => unreachable!(),
    };

    let next_parser = |input| match precedence {
        1..=2 => parse_binary_operators(input, precedence + 1),
        _ => parse_factor_expression(input),
    };

    let (input, lhs) = next_parser(input)?;
    let (input, value) = fold_many0(
        tuple((token_parser, next_parser)),
        || lhs.clone(),
        accumulate_expression,
    )(input)?;

    Ok((input, value))
}

fn accumulate_expression<'a>(
    acc: Token<'a, Expression>,
    (operator, rhs): (Token<'a, String>, Token<'a, Expression>),
) -> Token<'a, Expression> {
    let operator = match operator.value.as_str() {
        "+" => BinaryOperator::ArithmeticOperator(ArithmeticOperator::Add),
        "-" => BinaryOperator::ArithmeticOperator(ArithmeticOperator::Subtract),
        "*" => BinaryOperator::ArithmeticOperator(ArithmeticOperator::Multiply),
        "/" => BinaryOperator::ArithmeticOperator(ArithmeticOperator::Divide),
        "==" => BinaryOperator::ComparisonOperator(ComparisonOperator::Equal),
        "!=" => BinaryOperator::ComparisonOperator(ComparisonOperator::NotEqual),
        "<" => BinaryOperator::ComparisonOperator(ComparisonOperator::LessThan),
        "<=" => BinaryOperator::ComparisonOperator(ComparisonOperator::LessOrEqual),
        ">" => BinaryOperator::ComparisonOperator(ComparisonOperator::GreaterThan),
        ">=" => BinaryOperator::ComparisonOperator(ComparisonOperator::GreaterOrEqual),

        // unreachable because it means a parser fucked up and gave us a token we don't expect
        _ => unreachable!("unknown operator {}", operator.value),
    };
    Token {
        position: acc.position,
        value: Expression::BinaryExpression(Box::new(acc.value), operator, Box::new(rhs.value)),
    }
}

#[cfg(test)]
mod tests {
    use nom::Slice;

    use crate::{
        parser::{Expression, Identifier, Literal, Token},
        tests::parse_test,
    };

    use super::{
        parse_binary_operator_expression, ArithmeticOperator, BinaryOperator, ComparisonOperator,
    };

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
                            BinaryOperator::ArithmeticOperator(ArithmeticOperator::Add),
                            Box::new(Expression::Literal(Literal::Integer(3))),
                        )),
                        BinaryOperator::ArithmeticOperator(ArithmeticOperator::Add),
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
                            BinaryOperator::ArithmeticOperator(ArithmeticOperator::Multiply),
                            Box::new(Expression::Literal(Literal::Integer(3))),
                        )),
                        BinaryOperator::ArithmeticOperator(ArithmeticOperator::Divide),
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
                        BinaryOperator::ArithmeticOperator(ArithmeticOperator::Add),
                        Box::new(Expression::BinaryExpression(
                            Box::new(Expression::Literal(Literal::Integer(3))),
                            BinaryOperator::ArithmeticOperator(ArithmeticOperator::Multiply),
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
                            BinaryOperator::ArithmeticOperator(ArithmeticOperator::Multiply),
                            Box::new(Expression::Literal(Literal::Integer(2))),
                        )),
                        BinaryOperator::ArithmeticOperator(ArithmeticOperator::Add),
                        Box::new(Expression::BinaryExpression(
                            Box::new(Expression::Literal(Literal::Integer(3))),
                            BinaryOperator::ArithmeticOperator(ArithmeticOperator::Multiply),
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
                        BinaryOperator::ArithmeticOperator(ArithmeticOperator::Add),
                        Box::new(Expression::Identifier(Identifier("x".to_string()))),
                    ),
                },
            )
        })
    }

    #[test]
    pub fn test_rainbow_operators() {
        parse_test(
            parse_binary_operator_expression,
            "1 + 2 - 3 * 4 / 5 == 7 != 8 < 9 <= 10 > 11 >= 12",
            |input| {
                (
                    input.slice(48..),
                    Token {
                        position: input.slice(0..0),
                        value: Expression::BinaryExpression(
                            Box::new(Expression::BinaryExpression(
                                Box::new(Expression::BinaryExpression(
                                    Box::new(Expression::BinaryExpression(
                                        Box::new(Expression::BinaryExpression(
                                            Box::new(Expression::BinaryExpression(
                                                Box::new(Expression::BinaryExpression(
                                                    Box::new(Expression::BinaryExpression(
                                                        Box::new(Expression::Literal(
                                                            Literal::Integer(1),
                                                        )),
                                                        BinaryOperator::ArithmeticOperator(
                                                            ArithmeticOperator::Add,
                                                        ),
                                                        Box::new(Expression::Literal(
                                                            Literal::Integer(2),
                                                        )),
                                                    )),
                                                    BinaryOperator::ArithmeticOperator(
                                                        ArithmeticOperator::Subtract,
                                                    ),
                                                    Box::new(Expression::BinaryExpression(
                                                        Box::new(Expression::BinaryExpression(
                                                            Box::new(Expression::Literal(
                                                                Literal::Integer(3),
                                                            )),
                                                            BinaryOperator::ArithmeticOperator(
                                                                ArithmeticOperator::Multiply,
                                                            ),
                                                            Box::new(Expression::Literal(
                                                                Literal::Integer(4),
                                                            )),
                                                        )),
                                                        BinaryOperator::ArithmeticOperator(
                                                            ArithmeticOperator::Divide,
                                                        ),
                                                        Box::new(Expression::Literal(
                                                            Literal::Integer(5),
                                                        )),
                                                    )),
                                                )),
                                                BinaryOperator::ComparisonOperator(
                                                    ComparisonOperator::Equal,
                                                ),
                                                Box::new(Expression::Literal(Literal::Integer(7))),
                                            )),
                                            BinaryOperator::ComparisonOperator(
                                                ComparisonOperator::NotEqual,
                                            ),
                                            Box::new(Expression::Literal(Literal::Integer(8))),
                                        )),
                                        BinaryOperator::ComparisonOperator(
                                            ComparisonOperator::LessThan,
                                        ),
                                        Box::new(Expression::Literal(Literal::Integer(9))),
                                    )),
                                    BinaryOperator::ComparisonOperator(
                                        ComparisonOperator::LessOrEqual,
                                    ),
                                    Box::new(Expression::Literal(Literal::Integer(10))),
                                )),
                                BinaryOperator::ComparisonOperator(ComparisonOperator::GreaterThan),
                                Box::new(Expression::Literal(Literal::Integer(11))),
                            )),
                            BinaryOperator::ComparisonOperator(ComparisonOperator::GreaterOrEqual),
                            Box::new(Expression::Literal(Literal::Integer(12))),
                        ),
                    },
                )
            },
        )
    }
}
