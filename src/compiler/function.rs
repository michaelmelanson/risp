use memmap2::Mmap;

use crate::{
    codegen::{self, FuncPointer},
    value::Value,
};

#[derive(Debug)]
pub struct Function {
    #[allow(dead_code)]
    memory_map: Mmap,
    ptr: codegen::FuncPointer,
}

impl Function {
    pub fn call(&self) -> Value {
        let result = unsafe { (self.ptr)() };
        match result.try_into() {
            Ok(value) => value,
            Err(err) => panic!("failed to decode value: {:?}", err),
        }
    }

    pub fn address(&self) -> u64 {
        self.ptr as u64
    }

    pub fn new(memory_map: Mmap, ptr: FuncPointer) -> Self {
        Self { memory_map, ptr }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{:#X}", self.ptr as usize)
    }
}