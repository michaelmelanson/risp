use std::{collections::HashMap, rc::Rc};

use crate::{compiler::Function, parser::Identifier};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Symbol {
    Argument(usize),
    Function(Rc<Function>, usize),
    StackVariable(usize),
}

#[derive(Default, Debug)]
pub struct StackFrame<'a> {
    parent: Option<&'a StackFrame<'a>>,
    definitions: HashMap<Identifier, Symbol>,
    stack_slots: usize,
}

impl<'a> StackFrame<'a> {
    pub fn push(&self) -> StackFrame {
        StackFrame {
            parent: Some(self),
            definitions: HashMap::new(),
            stack_slots: 0,
        }
    }

    pub fn insert(&mut self, name: &Identifier, symbol: Symbol) {
        self.definitions.insert(name.clone(), symbol);
    }

    pub fn insert_stack_variable(&mut self, name: &Identifier) -> Symbol {
        let offset = self.stack_slots;
        self.stack_slots += 1;

        let symbol = Symbol::StackVariable(offset);
        self.insert(name, symbol.clone());
        symbol
    }

    pub fn resolve(&self, name: &Identifier) -> Option<Symbol> {
        self.resolve_with_offset(name, 0)
    }

    pub fn resolve_with_offset(&self, name: &Identifier, frame_offset: usize) -> Option<Symbol> {
        if let Some(symbol) = self.definitions.get(name) {
            match symbol {
                Symbol::StackVariable(variable_offset) => {
                    Some(Symbol::StackVariable(frame_offset + variable_offset))
                }
                _ => Some(symbol.clone()),
            }
        } else if let Some(parent) = self.parent {
            parent.resolve_with_offset(name, frame_offset + self.stack_slots)
        } else {
            None
        }
    }

    pub(crate) fn stack_slots(&self) -> usize {
        self.stack_slots
    }
}
