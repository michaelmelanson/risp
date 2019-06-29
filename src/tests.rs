use crate::{
    parser,
    evaluator
};

fn eval(code: &str) -> u64 {
    let (_, ast) = parser::term(code).expect("failed to parse");
    evaluator::execute(&ast).expect("failed to compile")
}

#[test]
fn test_arithmetic() {
    assert_eq!(eval("(+ 55 42)"), 97);
    assert_eq!(eval("(* 21 2)"), 42);
}
