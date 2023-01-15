mod x86_64;

type CodegenResult<T> = Result<T, CodegenError>;

pub use self::x86_64::{codegen, CodegenError, FuncPointer, Function};
