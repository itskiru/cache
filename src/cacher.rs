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
        let key = gen::user_voice_state(guild_id, user_id);

        if let Some(token) = voice_state.token {
            let cmd = resp_array![
                "HMSET",
                key,
                "channel_id",
                voice_state.channel_id as usize,
                "session_id",
                voice_state.session_id,
                "token",
                token
            ];

            let res = self.send(cmd).compat();

            FutureObj::new(Box::new(async {
                await!(res)?;

                Ok(())
            }))
        } else {
            let cmd_upsert = resp_array![
                "HMSET",
                &key,
                "channel_id",
                voice_state.channel_id as usize,
                "session_id",
                voice_state.session_id
            ];

            let cmd_del = resp_array![
                "HDEL",
                key,
                "token"
            ];

            let [f1, f2] = [
                self.send(cmd_upsert).compat(),
                self.send(cmd_del).compat(),
            ];

            FutureObj::new(Box::new(async {
                let (res1, res2) = join!(f1, f2);
                res1?;
                res2?;

                Ok(())
            }))
        }
    }
}
