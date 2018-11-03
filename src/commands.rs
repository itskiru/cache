use crate::error::Result;
use essentials::result::ResultExt;
use futures::compat::Future01CompatExt;
use redis_async::{
    client::PairedConnection,
    resp::{FromResp, RespValue},
};
use std::sync::Arc;

pub struct CommandablePairedConnection {
    inner: Arc<PairedConnection>,
}

impl CommandablePairedConnection {
    pub fn new(connection: Arc<PairedConnection>) -> Self {
        Self {
            inner: connection,
        }
    }

    pub async fn send<T: FromResp>(&self, value: RespValue) -> Result<T> {
        await!(self.inner.send(value).compat()).into_err()
    }

    pub fn send_sync(&self, value: RespValue) {
        self.inner.send_and_forget(value)
    }

    pub async fn del(&self, key: String) -> Result<()> {
        await!(self.send::<i64>(resp_array!["DEL", key]))?;

        Ok(())
    }

    pub fn del_sync(&self, key: String) {
        self.send_sync(resp_array!["DEL", key]);
    }

    pub async fn delm<'a, T: Into<String>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        keys: It,
    ) -> Result<()> {
        for key in keys.into_iter() {
            await!(self.del(key.into()))?;
        }

        Ok(())
    }

    pub fn delm_sync<T: Into<String>, It: IntoIterator<Item = T>>(
        &self,
        keys: It,
    ) {
        for key in keys.into_iter() {
            self.del_sync(key.into());
        }
    }

    pub async fn get<T: FromResp + 'static>(
        &self,
        key: String,
    ) -> Result<T> {
        let value = await!(self.send(resp_array![
            "GET",
            key
        ]))?;

        FromResp::from_resp(value).into_err()
    }

    pub async fn hdel<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<()> {
        let mut values = values.into_iter().map(Into::into).collect();

        await!(self.send::<i64>(resp_array!["HDEL", key].append(&mut values)))?;

        Ok(())
    }

    pub fn hdel_sync<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) {
        let mut values = values.into_iter().map(Into::into).collect();

        self.send_sync(resp_array!["HDEL", key].append(&mut values));
    }

    pub async fn hgetall(&self, key: String) -> Result<RespValue> {
        await!(self.send(resp_array!["HGETALL", key]))
    }

    pub async fn hmset<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<()> {
        let mut values = values.into_iter().map(Into::into).collect();

        await!(self.send(resp_array!["HMSET", key].append(&mut values)))?;

        Ok(())
    }

    pub fn hmset_sync<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    )  {
        let mut values = values.into_iter().map(Into::into).collect();

        self.send_sync(resp_array!["HMSET", key].append(&mut values));
    }

    pub async fn rpush<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<()> {
        let mut values = values.into_iter().map(Into::into).collect();

        await!(self.send(resp_array!["RPUSH", key].append(&mut values)))?;

        Ok(())
    }

    pub async fn sadd<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<i64> {
        let mut values = values.into_iter().map(Into::into).collect::<Vec<_>>();

        if values.is_empty() {
            return Ok(0);
        }

        await!(self.send::<i64>(resp_array![
            "SADD",
            key
        ].append(&mut values)))
    }

    pub fn sadd_sync<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) {
        let mut values = values.into_iter().map(Into::into).collect::<Vec<_>>();

        if values.is_empty() {
            return;
        }

        self.send_sync(resp_array![
            "SADD",
            key
        ].append(&mut values));
    }

    pub async fn set<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<i64> {
        let mut values = values.into_iter().map(Into::into).collect();

        await!(self.send(resp_array!["SET", key].append(&mut values)))
    }

    pub async fn smembers<T: FromResp + 'static>(
        &self,
        key: String,
    ) -> Result<T> {
        let values = await!(self.send(resp_array!["SMEMBERS", key]))?;

        FromResp::from_resp(values).into_err()
    }

    pub async fn srem(&self, key: String, mut ids: Vec<usize>) -> Result<RespValue> {
        await!(self.send(resp_array!["SREM", key].append(&mut ids)))
    }

    pub fn srem_sync(&self, key: String, mut ids: Vec<usize>) {
        self.send_sync(resp_array!["SREM", key].append(&mut ids))
    }

    pub async fn lrange(&self, key: String, min: i64, max: i64) -> Result<RespValue> {
        // TODO(Proximyst): Use just `resp_array!` when coercion from
        // i32/i64 to RespValue::Integer is added

        await!(self.send(resp_array!["LRANGE", key].append(&mut vec![
            RespValue::Integer(min),
            RespValue::Integer(max),
        ])))
    }

    pub fn lrange_sync(&self, key: String, min: i64, max: i64) {
        // TODO(Proximyst): Use just `resp_array!` when coercion from
        // i32/i64 to RespValue::Integer is added

        self.send_sync(resp_array!["LRANGE", key].append(&mut vec![
            RespValue::Integer(min),
            RespValue::Integer(max),
        ]))
    }
}
