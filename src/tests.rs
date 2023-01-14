#[cfg(test)]
pub fn parse_test<T: std::fmt::Debug + PartialEq>(
    parser: impl Fn(crate::parser::Span) -> crate::parser::ParseResult<T>,
    input: &str,
    expected: impl Fn(crate::parser::Span) -> (crate::parser::Span, crate::parser::Token<T>),
) {
    use crate::parser::Span;

    let input = Span::new(input);
    let actual = parser(input);

    match actual {
        Ok(actual) => {
            let expected = expected(input);
            assert_eq!(actual, expected);
        }
        Err(error) => panic!("Parsing test failed: {:?}", error),
    }
}

#[cfg(test)]
mod test {
    use crate::{evaluator::Evaluator, value::Value};

    fn eval(code: &str) -> Value {
        Evaluator::default()
            .evaluate(code)
            .expect("evaluation failed")
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("55 + 42"), Value::Integer(97));
        assert_eq!(eval("21 * 2"), Value::Integer(42));
        assert_eq!(eval("1+2*3"), Value::Integer(7));
        assert_eq!(eval("2*3+3*4"), Value::Integer(18));
    }

    #[test]
    fn test_bracketed_expressions() {
        assert_eq!(eval("1+(2*3)"), Value::Integer(7));
        assert_eq!(eval("(2*3)+(3*4)"), Value::Integer(18));
    }

    #[test]
    fn test_function_evaluation() {
        assert_eq!(
            eval(
                "def add_one(x) { 
                    1 + x
                } 
                add_one(54)"
            ),
            Value::Integer(55)
        );
    }

    #[test]
    fn test_reuse_function_argument() {
        assert_eq!(
            eval("def square (x) { x * x } square(3)"),
            Value::Integer(9)
        )
    }

    #[test]
    fn test_string_value() {
        assert_eq!(
            eval("\"Hello world!\""),
            Value::String("Hello world!".to_string())
        );
    }

    #[test]
    fn test_let() {
        assert_eq!(
            eval("def square (x) { let result = x * x\n  result }\nsquare(3)"),
            Value::Integer(9)
        )
    }

    #[test]
    fn test_if() {
        assert_eq!(
            eval(
                "
            def is_one(x) { 
                if x {
                    return 1
                }
                
                0
            } is_one(1)"
            ),
            Value::Integer(1)
        );

        assert_eq!(
            eval(
                "
            def is_one(x) { 
                if x {
                    return 1
                }
                0
            } is_one(0)"
            ),
            Value::Integer(0)
        );

        assert_eq!(
            eval(
                "
            def is_one(x) { 
                if x {
                    return 1
                } else {
                    return 0
                }
                2
            } is_one(0)"
            ),
            Value::Integer(0)
        );

        assert_eq!(
            eval(
                "
            def is_one(x) { 
                if x {
                    return 1
                } else {
                    return 0
                }
                2
            } is_one(1)"
            ),
            Value::Integer(1)
        );

        assert_eq!(
            eval(
                "
            def this_or_that(x,y) { 
                if x {
                    return 1
                } else if y {
                    return 2
                }
                3
            } this_or_that(1,0)"
            ),
            Value::Integer(1)
        );

        assert_eq!(
            eval(
                "
            def this_or_that(x,y) { 
                if x {
                    return 1
                } else if y {
                    return 2
                }
                3
            } this_or_that(0,1)"
            ),
            Value::Integer(2)
        );

        assert_eq!(
            eval(
                "
            def this_or_that(x,y) { 
                if x {
                    return 1
                } else if y {
                    return 2
                }
                3
            } this_or_that(0,0)"
            ),
            Value::Integer(3)
        );
    }
}
