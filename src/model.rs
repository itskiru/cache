use crate::resp_impl::RespValueExt;
use redis_async::{
    error::Error,
    resp::{FromResp, RespValue},
};
use serenity::model::{
    channel::PermissionOverwriteType,
    permissions::Permissions,
};
use serde_json::{Map, Number, Value};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Guild {
    pub afk_channel_id: Option<u64>,
    pub channels: HashSet<u64>,
    pub features: HashSet<String>,
    pub members: HashSet<u64>,
    pub name: String,
    pub owner_id: u64,
    pub region: String,
    pub roles: HashSet<u64>,
    pub voice_states: HashSet<u64>,
}

impl FromResp for Guild {
    fn from_resp_int(resp: RespValue) -> Result<Self, Error> {
        let values = match resp {
            RespValue::Array(x) => x,
            _ => return Err(Error::RESP("Expected an array".to_owned(), None)),
        };

        let map = create_hashmap(values);

        Ok(serde_json::from_value(Value::from(map)).expect("err deserializing"))
    }
}

#[derive(Clone, Debug)]
pub struct GuildChannel {
    pub bitrate: Option<u64>,
    pub category_id: Option<u64>,
    pub kind: u64,
    pub name: String,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub user_limit: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct Member {
    pub deaf: bool,
    pub nick: Option<String>,
    pub roles: Vec<u64>,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

#[derive(Clone, Debug)]
pub struct Role {
    pub name: String,
    pub permissions: Permissions,
}

#[derive(Clone, Debug)]
pub struct User {
    pub bot: bool,
    pub discriminator: u16,
    pub id: u64,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct VoiceState {
    pub channel_id: u64,
    pub session_id: String,
    pub token: Option<String>,
}

fn create_hashmap(resp: Vec<RespValue>) -> Map<String, Value> {
    let mut map = Map::with_capacity(resp.len() / 2);
    let mut iter = resp.into_iter();

    loop {
        let key = match iter.next() {
            Some(key) => key,
            None => break,
        };
        let value = iter.next().unwrap();
        let v = resp_to_value(value);
        map.insert(key.into_string(), v);
    }

    map
}

fn resp_to_value(resp: RespValue) -> Value {
    match resp {
        RespValue::Nil => Value::Null,
        RespValue::Array(resps) => Value::Array(resps.into_iter().map(resp_to_value).collect()),
        RespValue::BulkString(bytes) => {
            let string = String::from_utf8(bytes).unwrap();

            if let Ok(v) = string.parse::<u64>() {
                Value::Number(Number::from(v))
            } else {
                Value::String(string)
            }
        },
        RespValue::Error(why) => panic!("{:?}", why),
        RespValue::Integer(integer) => Value::Number(Number::from(integer)),
        RespValue::SimpleString(string) => Value::String(string),
    }
}
