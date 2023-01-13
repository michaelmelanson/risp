use std::convert::{Into, TryFrom, TryInto};

#[derive(Clone, Copy, Debug)]
pub enum ValueEncodeError {
    Overflow,
}

#[derive(Clone, Copy, Debug)]
pub enum ValueDecodeError {
    UnknownType(u64),
    InvalidBoolean(u64),
}

#[derive(Clone, Copy, Debug)]
#[repr(i64)]
pub enum ValueType {
    Integer,
    String,
    Boolean,
}

impl TryFrom<u64> for ValueType {
    type Error = ValueDecodeError;

    fn try_from(type_number: u64) -> Result<Self, Self::Error> {
        match type_number {
            0 => Ok(ValueType::Integer),
            1 => Ok(ValueType::String),
            2 => Ok(ValueType::Boolean),
            _ => Err(ValueDecodeError::UnknownType(type_number)),
        }
    }
}

impl From<ValueType> for u64 {
    fn from(value: ValueType) -> Self {
        match value {
            ValueType::Integer => 0,
            ValueType::String => 1,
            ValueType::Boolean => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
}

impl From<Value> for ValueType {
    fn from(value: Value) -> Self {
        match value {
            Value::Integer(_) => ValueType::Integer,
            Value::String(_) => ValueType::String,
            Value::Boolean(_) => ValueType::Boolean,
        }
    }
}

// Note: this should not derive Copy or Clone, because decoding a String will drop it.
#[derive(Debug)]
#[repr(transparent)]
pub struct EncodedValue(u64);

impl EncodedValue {
    const VALUE_BITS: u64 = 53;
    const VALUE_MASK: u64 = (1 << Self::VALUE_BITS) - 1;
    const TYPE_MASK: u64 = !Self::VALUE_MASK;

    // Returns the encoded value.
    //
    // This is unsafe and consumes the value, because it requires whoever
    // calls it to ensure that the value ends up being decoded again so it
    // gets properly dropped. If not then memory will leak.
    //
    // It also must only be decoded exactly once, or else the memory will be
    // dropped more than once which will crash the program.
    pub unsafe fn encoded_value(self) -> u64 {
        self.0
    }
}

impl From<String> for EncodedValue {
    fn from(s: String) -> Self {
        let value = Value::String(s);
        value.try_into().expect("failed to encode string value")
    }
}

impl TryFrom<Value> for EncodedValue {
    type Error = ValueEncodeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        EncodedValue::try_from(&value)
    }
}

impl TryFrom<&Value> for EncodedValue {
    type Error = ValueEncodeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let (value, value_type) = match value {
            Value::Integer(value) => (*value as u64, ValueType::Integer),
            Value::String(s) => {
                let boxed_str = Box::new(s.clone());
                let str_ref = Box::<String>::leak(boxed_str);
                let ptr = str_ref as *mut String;
                (ptr as u64, ValueType::String)
            }
            Value::Boolean(true) => (1, ValueType::Boolean),
            Value::Boolean(false) => (0, ValueType::Boolean),
        };

        if value & Self::TYPE_MASK != 0 {
            return Err(ValueEncodeError::Overflow);
        }

        let type_code = Into::<u64>::into(value_type) << Self::VALUE_BITS;
        let encoded = value | type_code;
        Ok(EncodedValue(encoded))
    }
}

impl TryFrom<EncodedValue> for Value {
    type Error = ValueDecodeError;

    fn try_from(encoded: EncodedValue) -> Result<Self, Self::Error> {
        let type_number = encoded.0 >> EncodedValue::VALUE_BITS;
        let value = (encoded.0 & EncodedValue::VALUE_MASK) as i64;
        match ValueType::try_from(type_number)? {
            ValueType::Integer => Ok(Value::Integer(value)),
            ValueType::String => {
                let ptr = value as u64 as *mut String;
                let boxed_str = unsafe { Box::from_raw(ptr) };
                let string = *boxed_str;
                Ok(Value::String(string))
            }
            ValueType::Boolean => match value {
                0 => Ok(Value::Boolean(false)),
                1 => Ok(Value::Boolean(true)),
                value => Err(ValueDecodeError::InvalidBoolean(encoded.0)),
            },
        }
    }
}
