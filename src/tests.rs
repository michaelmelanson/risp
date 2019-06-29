
#[cfg(test)]
mod test {
    use crate::{
        parser,
        compiler,

        parser::Literal
    };

    fn eval<'a>(code: &'a str) -> Literal<'a> {
        let (_, ast) = parser::term(code).expect("failed to parse");
        compiler::execute(&ast).expect("failed to compile")
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("(+ 55 42)"), Literal::Int(97));
        assert_eq!(eval("(* 21 2)"), Literal::Int(42));

        assert_eq!(eval("(+ 1 (* 2 3))"), Literal::Int(7));
        assert_eq!(eval("(+ (* 2 3) (* 3 4))"), Literal::Int(18));
    }
}