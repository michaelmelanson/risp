mod error;

use std::rc::Rc;

use crate::{
    compiler::{
        self,
        stack_frame::{StackFrame, Symbol},
    },
    parser::{self, Statement},
    value::Value,
};

use self::error::EvaluationError;

#[derive(Default)]
pub struct Evaluator<'a> {
    stack_frame: StackFrame<'a>,
}

impl<'a> Evaluator<'a> {
    pub fn evaluate<'b>(&mut self, line: &'b str) -> Result<Value, EvaluationError<'b>> {
        let (remainder, block) = parser::parse(line)?;

        for statement in &block.value.0 {
            self.evaluate_statement(statement)?;
        }

        let function = compiler::compile(&mut self.stack_frame, &block.value)?;
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
            Statement::FunctionDefinition(ref definition) => {
                let mut stack_frame = self.stack_frame.push();

                for (index, arg) in definition.args.iter().enumerate() {
                    stack_frame.insert(arg, Symbol::Argument(index));
                }

                let function = compiler::compile(&mut stack_frame, &definition.body)?;
                let symbol = Symbol::Function(Rc::new(function), definition.args.len());
                println!("Function {} defined", definition.name);
                self.stack_frame.insert(&definition.name, symbol);
            }
            Statement::VariableDeclaration(_declaration) => todo!("evaluate variable declaration"),
            Statement::Expression(_expression) => {}
        }
        Ok(())
    }
}
