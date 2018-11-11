use redis_async::error::Error as RedisError;
use serde_json::Error as JsonError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    num::ParseIntError,
    option::NoneError,
    result::Result as StdResult,
};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidLoopMode,
    Json(JsonError),
    None,
    ParseInt(ParseIntError),
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
            InvalidLoopMode => "Invalid loop mode",
            Json(why) => why.description(),
            None => "none",
            ParseInt(why) => why.description(),
            Redis(why) => why.description(),
        }
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Error {
        Error::Json(e)
    }
}

impl From<NoneError> for Error {
    fn from(_: NoneError) -> Error {
        Error::None
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error {
        Error::ParseInt(e)
    }
}

impl From<RedisError> for Error {
    fn from(e: RedisError) -> Error {
        Error::Redis(e)
    }
}
