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
