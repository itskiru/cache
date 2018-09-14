use crate::{
    error::Result,
    gen,
    model::VoiceState,
};
use essentials::VecExt as _;
use futures::compat::Future01CompatExt as _;
use redis_async::{
    client::PairedConnection,
    resp::{FromResp, RespValue},
};
use std::{
    collections::HashMap,
    sync::Arc,
};

/// A struct with common shared functionality over the bot's cache.
pub struct Cache {
    redis: Arc<PairedConnection>,
}

impl Cache {
    /// Creates a new cache accesser instance.
    pub fn new(redis: Arc<PairedConnection>) -> Self {
        Self {
            redis,
        }
    }

    /// Removes a guild member's voice state.
    ///
    /// Removes the user's ID to the guild's voice state Set if it was in the
    /// Set.
    ///
    /// Returns whether a voice state was deleted.
    pub async fn delete_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> Result<bool> {
        // Remove the voice state for the user.
        await!(self.delete_voice_state_atomic(guild_id, user_id))?;

        // Remove the user's ID from the guild's voice state set.
        let deleted: usize = await!(self.send(resp_array![
            "SREM",
            gen::guild_voice_states(guild_id),
            user_id as usize
        ]))?;

        Ok(deleted > 0)
    }

    async fn delete_voice_state_atomic(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> Result<()> {
        await!(self.send(resp_array![
            "DEL",
            gen::user_voice_state(guild_id, user_id)
        ])).map_err(From::from)
    }

    async fn delete_voice_state_list(
        &self,
        guild_id: u64,
    ) -> Result<()> {
        await!(self.send(resp_array![
            "DEL",
            gen::guild_voice_states(guild_id)
        ])).map_err(From::from)
    }

    /// Deletes all of the voice states for a guild.
    ///
    /// Returns the number of voice states deleted.
    pub async fn delete_voice_states(
        &self,
        guild_id: u64,
    ) -> Result<u64> {
        let ids = await!(self.get_voice_state_list(guild_id))?;

        let count = ids.len();

        for id in ids {
            await!(self.delete_voice_state_atomic(guild_id, id))?;
        }

        await!(self.delete_voice_state_list(guild_id))?;

        Ok(count as u64)
    }

    /// Returns a voice state for a guild member, if one exists for them.
    pub async fn get_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> Result<Option<VoiceState>> {
        let value: Option<Vec<RespValue>> = await!(self.send(resp_array![
            "HGETALL",
            gen::user_voice_state(guild_id, user_id)
        ]))?;

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
    }

    /// Gets all of the voice states for a guild.
    pub async fn get_voice_states(
        &self,
        guild_id: u64,
    ) -> Result<HashMap<u64, VoiceState>> {
        let user_ids = await!(self.get_voice_state_list(guild_id))?;

        let mut map = HashMap::new();

        for id in user_ids {
            let state = await!(self.get_voice_state(guild_id, id))??;

            map.insert(id, state);
        }

        Ok(map)
    }

    /// Gets the IDs of all members that have a voice state in a guild.
    pub async fn get_voice_state_list(
        &self,
        guild_id: u64,
    ) -> Result<Vec<u64>> {
        await!(self.send(resp_array![
            "GET",
            gen::guild_voice_states(guild_id)
        ])).map_err(From::from)
    }

    /// Upserts a guild member's voice state.
    ///
    /// Adds the user's ID to the guild's voice state Set if it wasn't already
    /// in the Set.
    pub async fn set_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
        voice_state: VoiceState,
    ) -> Result<()> {
        let guild_key = gen::guild_voice_states(guild_id);
        let user_key = gen::user_voice_state(guild_id, user_id);

        if let Some(token) = voice_state.token {
            await!(self.send(resp_array![
                "HMSET",
                user_key,
                "channel_id",
                voice_state.channel_id as usize,
                "session_id",
                voice_state.session_id,
                "token",
                token
            ]))?;

            await!(self.send(resp_array![
                "SADD",
                guild_key,
                user_id as usize
            ]))?;
        } else {
            await!(self.send(resp_array![
                "HMSET",
                &user_key,
                "channel_id",
                voice_state.channel_id as usize,
                "session_id",
                voice_state.session_id
            ]))?;
            await!(self.send(resp_array![
                "HDEL",
                user_key,
                "token"
            ]))?;
            await!(self.send(resp_array![
                "SADD",
                guild_key,
                user_id as usize
            ]))?;
        }

        Ok(())
    }

    async fn send<T: FromResp>(&self, value: RespValue) -> Result<T> {
        await!(self.redis.send(value).compat()).map_err(From::from)
    }
}
