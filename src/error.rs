use redis_async::error::Error as RedisError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    future::FutureObj,
    option::NoneError,
};

pub type FutureResult<T> = FutureObj<'static, Result<T, Error>>;

#[derive(Debug)]
pub enum Error {
    None,
    Redis(RedisError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match self {
            None => "none",
            Redis(why) => why.description(),
        }
    }
}

impl From<NoneError> for Error {
    fn from(_: NoneError) -> Error {
        Error::None
    }
}

impl From<RedisError> for Error {
    fn from(e: RedisError) -> Error {
        Error::Redis(e)
    }
}
