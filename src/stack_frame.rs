use std::collections::HashMap;

use crate::compiler::Function;
use crate::parser::Identifier;

#[derive(Debug)]
pub enum Symbol {
    Argument(usize),
    Function(Function, usize),
}

pub struct StackFrame<'a> {
    parent: Option<&'a StackFrame<'a>>,
    definitions: HashMap<Identifier, Symbol>,
}

impl<'a> StackFrame<'a> {
    pub fn new() -> StackFrame<'a> {
        StackFrame {
            parent: None,
            definitions: HashMap::new(),
        }
    }

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
