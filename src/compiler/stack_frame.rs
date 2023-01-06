use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::Function;
use crate::parser::Identifier;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Symbol {
    Argument(usize),
    Function(Rc<Function>, usize),
}

#[derive(Default, Debug)]
pub struct StackFrame<'a> {
    parent: Option<&'a StackFrame<'a>>,
    definitions: HashMap<Identifier, Symbol>,
}

impl<'a> StackFrame<'a> {
    pub fn push(&self) -> StackFrame {
        StackFrame {
            parent: Some(self),
            definitions: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: Identifier, symbol: Symbol) {
        self.definitions.insert(name, symbol);
    }

    pub fn resolve(&self, name: &Identifier) -> Option<&Symbol> {
        if let Some(symbol) = self.definitions.get(name) {
            Some(symbol)
        } else if let Some(parent) = self.parent {
            parent.resolve(name)
        } else {
            None
        }
    }
}
