use crate::{
    error::FutureResult,
    gen,
    model::VoiceState,
};
use essentials::VecExt as _;
use futures::compat::Future01CompatExt as _;
use redis_async::{
    client::PairedConnection,
    resp::{FromResp, RespValue},
};
use std::future::FutureObj;

pub trait DabbotCache {
    fn delete_guild_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> FutureResult<()>;

    fn get_guild_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> FutureResult<Option<VoiceState>>;

    fn set_guild_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
        voice_state: VoiceState,
    ) -> FutureResult<()>;
}

impl DabbotCache for PairedConnection {
    fn delete_guild_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> FutureResult<()> {
        let del = {
            let key = gen::user_voice_state(guild_id, user_id);
            let cmd = resp_array!["DEL", key];

            self.send(cmd).compat()
        };
        let update = {
            let key = gen::guild_voice_states(guild_id);
            let cmd = resp_array!["SREM", key, user_id as usize];

            self.send(cmd).compat()
        };

        FutureObj::new(Box::new(async {
            let (res1, res2) = join!(del, update);
            res1?;
            res2?;

            Ok(())
        }))
    }

    fn get_guild_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> FutureResult<Option<VoiceState>> {
        let key = gen::user_voice_state(guild_id, user_id);
        let cmd = resp_array!["HGETALL", key];

        let res = self.send(cmd).compat();

        FutureObj::new(Box::new(async {
            let value: Option<Vec<RespValue>> = await!(res)?;

            let mut values = match value {
                Some(values) => values,
                None => return Ok(None),
            };

            let token = values.try_remove(2)?;
            let session_id = values.try_remove(1)?;
            let channel_id = values.try_remove(0)?;

            Ok(Some(VoiceState {
                channel_id: FromResp::from_resp(channel_id)?,
                session_id: FromResp::from_resp(session_id)?,
                token: String::from_resp(token).ok(),
            }))
        }))
    }

    fn set_guild_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
        voice_state: VoiceState,
    ) -> FutureResult<()> {
        let guild_key = gen::guild_voice_states(guild_id);
        let user_key = gen::user_voice_state(guild_id, user_id);

        if let Some(token) = voice_state.token {
            let add = resp_array![
                "SADD",
                guild_key,
                user_id as usize
            ];
            let set = resp_array![
                "HMSET",
                user_key,
                "channel_id",
                voice_state.channel_id as usize,
                "session_id",
                voice_state.session_id,
                "token",
                token
            ];

            let [f1, f2] = [
                self.send(add).compat(),
                self.send(set).compat(),
            ];

            FutureObj::new(Box::new(async {
                let (res1, res2) = join!(f1, f2);
                res1?;
                res2?;

                Ok(())
            }))
        } else {
            let add = resp_array![
                "SADD",
                guild_key,
                user_id as usize
            ];
            let set = resp_array![
                "HMSET",
                &user_key,
                "channel_id",
                voice_state.channel_id as usize,
                "session_id",
                voice_state.session_id
            ];
            let del = resp_array![
                "HDEL",
                user_key,
                "token"
            ];

            let [f1, f2, f3] = [
                self.send(add).compat(),
                self.send(set).compat(),
                self.send(del).compat(),
            ];

            FutureObj::new(Box::new(async {
                let (res1, res2, res3) = join!(f1, f2, f3);
                res1?;
                res2?;
                res3?;

                Ok(())
            }))
        }
    }
}
