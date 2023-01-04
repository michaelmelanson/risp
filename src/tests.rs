#[cfg(test)]
mod test {
    use crate::{evaluator::Evaluator, value::Value};

    fn eval(code: &str) -> Value {
        Evaluator::new().evaluate(code).expect("evaluation failed")
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("55 + 42"), Value::Integer(97));
        assert_eq!(eval("21 * 2"), Value::Integer(42));
        assert_eq!(eval("1+(2*3)"), Value::Integer(7));
        assert_eq!(eval("(2*3)+(3*4)"), Value::Integer(18));
    }

    #[test]
    fn test_function_evaluation() {
        assert_eq!(
            eval("def add_one(x) { 1 + x } add_one(54)"),
            Value::Integer(55)
        );
    }

    #[test]
    fn test_string_value() {
        assert_eq!(
            eval("\"Hello world!\""),
            Value::String("Hello world!".to_string())
        );
    }
}
