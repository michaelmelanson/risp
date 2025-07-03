use crate::value::ValueEncodeError;

#[derive(Debug)]
pub enum CodegenError {
    IcedError(iced_x86::IcedError),
    MmapError(std::io::Error),
    NotImplemented(String),
    InternalError(String),
    ValueEncodeError(ValueEncodeError),
    // ValueDecodeError(ValueDecodeError),
}

impl From<iced_x86::IcedError> for CodegenError {
    fn from(err: iced_x86::IcedError) -> Self {
        CodegenError::IcedError(err)
    }
}
