mod error;

use std::rc::Rc;

use crate::{
    compiler,
    parser::{self, Statement},
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
        let (remainder, block) = parser::parse(line).map_err(EvaluationError::ParseError)?;

        for statement in &block.value.0 {
            self.evaluate_statement(statement)?;
        }

        let function = compiler::compile(&self.stack_frame, &block.value)?;
        let result = function.call();

        if *remainder == "" {
            Ok(result)
        } else {
            self.evaluate(*remainder)
        }
    }

    pub fn evaluate_statement<'b>(
        &mut self,
        statement: &Statement,
    ) -> Result<(), EvaluationError<'b>> {
        match statement {
            Statement::Definition(ref definition) => {
                let mut stack_frame = self.stack_frame.push();

                for (index, arg) in definition.args.iter().enumerate() {
                    stack_frame.insert(arg.clone(), Symbol::Argument(index));
                }

                let function = compiler::compile(&stack_frame, &definition.body)?;
                let symbol = Symbol::Function(Rc::new(function), definition.args.len());
                println!("Function {} defined", definition.name);
                self.stack_frame.insert(definition.name.clone(), symbol);
            }
            Statement::Expression(ref _expression) => {}
        }
        Ok(())
    }
}
