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
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
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
        "+" => BinaryOperator::Add,
        "-" => BinaryOperator::Subtract,
        "*" => BinaryOperator::Multiply,
        "/" => BinaryOperator::Divide,
        "==" => BinaryOperator::Equal,
        "!=" => BinaryOperator::NotEqual,
        "<" => BinaryOperator::LessThan,
        "<=" => BinaryOperator::LessOrEqual,
        ">" => BinaryOperator::GreaterThan,
        ">=" => BinaryOperator::GreaterOrEqual,

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
                                                        BinaryOperator::Add,
                                                        Box::new(Expression::Literal(
                                                            Literal::Integer(2),
                                                        )),
                                                    )),
                                                    BinaryOperator::Subtract,
                                                    Box::new(Expression::BinaryExpression(
                                                        Box::new(Expression::BinaryExpression(
                                                            Box::new(Expression::Literal(
                                                                Literal::Integer(3),
                                                            )),
                                                            BinaryOperator::Multiply,
                                                            Box::new(Expression::Literal(
                                                                Literal::Integer(4),
                                                            )),
                                                        )),
                                                        BinaryOperator::Divide,
                                                        Box::new(Expression::Literal(
                                                            Literal::Integer(5),
                                                        )),
                                                    )),
                                                )),
                                                BinaryOperator::Equal,
                                                Box::new(Expression::Literal(Literal::Integer(7))),
                                            )),
                                            BinaryOperator::NotEqual,
                                            Box::new(Expression::Literal(Literal::Integer(8))),
                                        )),
                                        BinaryOperator::LessThan,
                                        Box::new(Expression::Literal(Literal::Integer(9))),
                                    )),
                                    BinaryOperator::LessOrEqual,
                                    Box::new(Expression::Literal(Literal::Integer(10))),
                                )),
                                BinaryOperator::GreaterThan,
                                Box::new(Expression::Literal(Literal::Integer(11))),
                            )),
                            BinaryOperator::GreaterOrEqual,
                            Box::new(Expression::Literal(Literal::Integer(12))),
                        ),
                    },
                )
            },
        )
    }
}
