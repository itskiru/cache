use redis_async::resp::RespValue;

pub trait RespValueExt {
    fn into_array(self) -> Vec<RespValue>;

    fn into_string(self) -> String;

    fn push(&mut self, value: impl Into<RespValue>) -> &mut Self;
}

impl RespValueExt for RespValue {
    fn into_array(self) -> Vec<RespValue> {
        match self {
            RespValue::Array(v) => v,
            other => unreachable!("Not a RESP array: {:?}", other),
        }
    }

    fn into_string(self) -> String {
        match self {
            RespValue::BulkString(bytes) => String::from_utf8(bytes).unwrap(),
            RespValue::SimpleString(string) => string,
            other => panic!("Not a RESP string: {:?}", other),
        }
    }

    fn push(&mut self, value: impl Into<RespValue>) -> &mut Self {
        if let RespValue::Array(inner) = self {
            inner.push(value.into());
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use redis_async::resp::RespValue;
    use super::RespValueExt;

    #[test]
    fn test_into_array() {
        assert_eq!(RespValue::Array(vec![]).into_array(), vec![]);
    }

    #[should_panic]
    #[test]
    fn test_into_array_from_bulk_string() {
        RespValue::BulkString(b"hi".to_vec()).into_array();
    }

    #[should_panic]
    #[test]
    fn test_into_array_from_error() {
        RespValue::Error("hello".to_owned()).into_array();
    }

    #[should_panic]
    #[test]
    fn test_into_array_from_integer() {
        RespValue::Integer(1).into_array();
    }

    #[should_panic]
    #[test]
    fn test_into_array_from_simple_string() {
        RespValue::SimpleString("hey".to_owned()).into_array();
    }

    #[should_panic]
    #[test]
    fn test_into_array_from_nil() {
        RespValue::Nil.into_array();
    }
}
