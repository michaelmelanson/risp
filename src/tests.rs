
#[cfg(test)]
mod test {
    use crate::{
        evaluator::Evaluator,
        parser::Literal
    };

    fn eval(code: &str) -> Literal {
        Evaluator::new()
            .evaluate(code)
            .expect("evaluation failed")
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("(+ 55 42)"), Literal::Int(97));
        assert_eq!(eval("(* 21 2)"), Literal::Int(42));
        assert_eq!(eval("(+ 1 (* 2 3))"), Literal::Int(7));
        assert_eq!(eval("(+ (* 2 3) (* 3 4))"), Literal::Int(18));
    }

    #[test]
    fn test_function_evaluation() {
        assert_eq!(eval("(def add_one (x) (+ 1 x)) (add_one 54)"), Literal::Int(55));
    }
}