mod error;

use std::rc::Rc;

use crate::{
    compiler, parser,
    stack_frame::{StackFrame, Symbol},
    value::Value,
};

use self::error::EvaluationError;

pub struct Evaluator<'a> {
    stack_frame: StackFrame<'a>,
}

impl<'a> Evaluator<'a> {
    pub fn new() -> Evaluator<'a> {
        Evaluator {
            stack_frame: StackFrame::new(),
        }
    }

    pub fn evaluate<'b>(&mut self, line: &'b str) -> Result<Value, EvaluationError<'b>> {
        let (remainder, term) = parser::term(line).map_err(EvaluationError::ParseError)?;

        if let parser::Term::Definition(ref definition) = term {
            let mut stack_frame = self.stack_frame.push();

            for (index, arg) in definition.args.iter().enumerate() {
                stack_frame.insert(arg.clone(), Symbol::Argument(index));
            }

            let function = compiler::compile(&stack_frame, &definition.body)?;
            let symbol = Symbol::Function(Rc::new(function), definition.args.len());
            println!("Function {} defined", definition.name);
            self.stack_frame.insert(definition.name.clone(), symbol);
        }

        let function =
            compiler::compile(&self.stack_frame, &term)?;
        let result = function.call();

        if remainder == "" {
            Ok(result)
        } else {
            self.evaluate(remainder)
        }
    }
}
