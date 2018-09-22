use crate::{
    error::{Error, Result},
    gen,
    model::VoiceState as CachedVoiceState,
    resp_impl::RespValueExt as _,
};
use essentials::{
    result::ResultExt as _,
    VecExt as _,
};
use futures::compat::Future01CompatExt as _;
use redis_async::{
    client::PairedConnection,
    resp::{FromResp, RespValue},
};
use serde::de::DeserializeOwned;
use serenity::model::prelude::*;
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
        let deleted = await!(self.send::<usize>(resp_array![
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
        ])).into_err()
    }

    async fn delete_voice_state_list(
        &self,
        guild_id: u64,
    ) -> Result<()> {
        await!(self.send(resp_array![
            "DEL",
            gen::guild_voice_states(guild_id)
        ])).into_err()
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
    ) -> Result<Option<CachedVoiceState>> {
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

        Ok(Some(CachedVoiceState {
            channel_id: FromResp::from_resp(channel_id)?,
            session_id: FromResp::from_resp(session_id)?,
            token: String::from_resp(token).ok(),
        }))
    }

    /// Gets all of the voice states for a guild.
    pub async fn get_voice_states(
        &self,
        guild_id: u64,
    ) -> Result<HashMap<u64, CachedVoiceState>> {
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
        await!(self.get(gen::guild_voice_states(guild_id)))
    }

    async fn send<T: FromResp>(&self, value: RespValue) -> Result<T> {
        await!(self.redis.send(value).compat()).into_err()
    }

    fn send_and_forget(&self, value: RespValue) {
        self.redis.send_and_forget(value)
    }
}

/// Redis commands.
impl Cache {
    async fn del(&self, key: String) -> Result<()> {
        await!(self.send::<i64>(resp_array!["DEL", key]))?;

        Ok(())
    }

    fn del_and_forget(&self, key: String) {
        self.send_and_forget(resp_array!["DEL", key]);
    }

    async fn delm<'a, T: Into<String>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        keys: It,
    ) -> Result<()> {
        for key in keys.into_iter() {
            await!(self.del(key.into()))?;
        }

        Ok(())
    }

    async fn get<T: FromResp + 'static>(
        &self,
        key: String,
    ) -> Result<T> {
        let value = await!(self.send(resp_array![
            "GET",
            key
        ]))?;

        FromResp::from_resp(value).into_err()
    }

    async fn hdel<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<()> {
        let mut values = values.into_iter().map(Into::into).collect();

        await!(self.send::<i64>(resp_array!["HDEL", key].append(&mut values)))?;

        Ok(())
    }

    fn hdel_and_forget<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) {
        let mut values = values.into_iter().map(Into::into).collect();

        self.send_and_forget(resp_array!["HDEL", key].append(&mut values));
    }

    async fn hgetall(&self, key: String) -> Result<RespValue> {
        await!(self.send(resp_array!["HGETALL", key]))
    }

    async fn hmset<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<()> {
        let mut values = values.into_iter().map(Into::into).collect();

        await!(self.send(resp_array!["HMSET", key].append(&mut values)))?;

        Ok(())
    }

    fn hmset_and_forget<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    )  {
        let mut values = values.into_iter().map(Into::into).collect();

        self.send_and_forget(resp_array!["HMSET", key].append(&mut values));
    }

    async fn rpush<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<()> {
        let mut values = values.into_iter().map(Into::into).collect();

        await!(self.send(resp_array!["RPUSH", key].append(&mut values)))?;

        Ok(())
    }

    async fn sadd<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
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

    fn sadd_and_forget<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) {
        let mut values = values.into_iter().map(Into::into).collect::<Vec<_>>();

        if values.is_empty() {
            return;
        }

        self.send_and_forget(resp_array![
            "SADD",
            key
        ].append(&mut values));
    }

    async fn set<'a, T: Into<RespValue>, It: IntoIterator<Item = T> + 'a>(
        &'a self,
        key: String,
        values: It,
    ) -> Result<i64> {
        let mut values = values.into_iter().map(Into::into).collect();

        await!(self.send(resp_array!["SET", key].append(&mut values)))
    }

    async fn smembers(&self, key: String) -> Result<RespValue> {
        await!(self.send(resp_array!["SMEMBERS", key]))
    }
}

/// Discord event updates.
impl Cache {
    async fn get_multiple<'a, T: DeserializeOwned + 'static>(
        &'a self,
        pairs: Vec<(u64, String)>,
    ) -> Result<HashMap<u64, T>> {
        let pair_len = pairs.len();
        let mut values = Vec::with_capacity(pair_len + 1);
        values.push(RespValue::from("MGET"));

        let mut ids = Vec::with_capacity(pair_len);

        for (id, key) in pairs {
            ids.push(id);
            values.push(RespValue::from(key));
        }

        let arr = RespValue::Array(values);
        let values = await!(self.send::<Vec<Vec<_>>>(arr))?;

        let mut ids = ids.into_iter();
        let mut map = HashMap::with_capacity(pair_len);

        for value in values {
            map.insert(ids.next()?, serde_json::from_slice(&value)?);
        }

        Ok(map)
    }

    pub async fn delete_channel(&self, id: u64) -> Result<()> {
        await!(self.del(gen::channel(id)))
    }

    pub async fn delete_channels<'a>(
        &'a self,
        ids: impl IntoIterator<Item = u64> + 'a,
    ) -> Result<()> {
        await!(self.delm(ids.into_iter().map(gen::channel)))
    }

    pub async fn delete_guild(&self, id: u64) -> Result<()> {
        await!(self.del(gen::guild(id)))
    }

    pub async fn delete_guilds<'a>(
        &'a self,
        ids: impl IntoIterator<Item = u64> + 'a,
    ) -> Result<()> {
        await!(self.delm(ids.into_iter().map(gen::guild)))
    }

    // pub async fn get_channel(&self, id: u64) -> Result<Channel> {
    //     await!(self.get(gen::channel(id)))
    // }

    pub async fn get_channels<'a>(
        &'a self,
        ids: impl IntoIterator<Item = u64> + 'a,
    ) -> Result<HashMap<u64, Channel>> {
        await!(self.get_multiple::<Channel>(ids.into_iter().map(|id| {
            (id, gen::channel(id))
        }).collect()))
    }

    pub async fn get_guild(&self, id: u64) -> Result<crate::model::Guild> {
        let values = await!(self.hgetall(gen::guild(id)))?.into_array();

        if values.is_empty() {
            return Err(Error::None);
        }

        let mut values = RespValue::Array(values);

        let channels = await!(self.smembers(gen::guild_channels(id)))?;
        values.push("channels").push(channels);

        let features = await!(self.smembers(gen::guild_features(id)))?;
        values.push("features").push(features);

        let members = await!(self.smembers(gen::guild_members(id)))?;
        values.push("members").push(members);

        let roles = await!(self.smembers(gen::guild_roles(id)))?;
        values.push("roles").push(roles);

        let voice_states = await!(self.smembers(gen::guild_voice_states(id)))?;
        values.push("voice_states").push(voice_states);

        FromResp::from_resp(values).into_err()
    }

    pub async fn upsert_channel<'a>(
        &'a self,
        channel: &'a Channel,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(channel)?;

        await!(self.set(gen::channel(channel.id().0), vec![bytes]))?;

        Ok(())
    }

    pub async fn upsert_guild<'a>(
        &'a self,
        guild: &'a Guild,
    ) -> Result<()> {
        let gid = guild.id.0;
        info!("Upserting guild ID {}", gid);

        let mut set = resp_array![
            "HMSET",
            gen::guild(gid),
            "name",
            &guild.name,
            "owner_id",
            guild.owner_id.0 as usize,
            "region",
            &guild.region
        ];
        let mut del = None;

        if let Some(afk_channel_id) = guild.afk_channel_id {
            set.push("afk_channel_id".to_owned()).push(afk_channel_id.0 as usize);
        } else {
            del = Some(vec![
                "afk_channel_id",
            ]);
        }

        info!("Sending guild upsert HMSET");
        self.redis.send_and_forget(set);
        info!("Guild upsert HMSET successful");

        if let Some(del) = del {
            info!("Sending guild upsert HDEL");
            self.hdel_and_forget(gen::guild(gid), del);
            info!("Sent guild upsert HDEL");
        }

        info!("Sending guild set channels");
        self.set_guild_channels(
            gid,
            guild.channels.keys().map(|x| x.0 as usize).collect(),
        );
        info!("Guild set channels successful");

        info!("Sending guild set features");
        self.set_guild_features(gid, guild.features.clone());
        info!("Guild set features successful");
        info!("Sending guild set members");
        self.set_guild_members(
            gid,
            guild.members.keys().map(|x| x.0 as usize).collect(),
        );
        info!("Guild set members successful");

        info!("Upserting guild members");
        for member in guild.members.values() {
            self.upsert_member(member)?;
        }
        info!("Guild members' upsert complete");

        info!("Sending guild set roles");
        self.set_guild_roles(
            gid,
            guild.roles.keys().map(|x| x.0 as usize).collect(),
        );
        info!("Guild set roles successful");

        info!("Upserting guild roles");
        for role in guild.roles.values() {
            self.upsert_role(gid, role);
        }
        info!("Guild roles' upsert complete");

        info!("Sending guild set voice states");
        self.set_guild_voice_states(
            gid,
            guild.voice_states.keys().map(|x| x.0 as usize).collect(),
        );
        info!("Guild set voice state successful");

        info!("Upserting guild voice states");
        for state in guild.voice_states.values() {
            self.upsert_voice_state(gid, state);
        }
        info!("Guild voice states' upsert complete");

        Ok(())
    }

    fn upsert_member<'a>(&'a self, member: &'a Member) -> Result<()> {
        let guild_id = member.guild_id.0;
        let user_id = member.user.id.0;

        let mut set = resp_array![
            "deaf",
            usize::from(member.deaf),
            "mute",
            usize::from(member.mute),
            "user_id",
            user_id as usize
        ];

        if let Some(joined_at) = member.joined_at {
            let ser = serde_json::to_vec(&joined_at)?;

            set.push("joined_at").push(ser);
        }

        if let Some(nick) = member.nick.as_ref() {
            set.push("nick").push(nick);
        } else {
            self.hdel_and_forget(
                gen::member(guild_id, user_id),
                vec!["afk_channel_id"],
            );
        }

        self.hmset_and_forget(gen::member(guild_id, user_id), set.into_array());

        self.set_member_roles(
            guild_id,
            user_id,
            member.roles.iter().map(|x| x.0 as usize).collect(),
        );

        Ok(())
    }

    fn upsert_role<'a>(
        &'a self,
        guild_id: u64,
        role: &'a Role,
    ) {
        let id = role.id.0;

        let hashes = resp_array![
            "colour",
            role.colour.0 as usize,
            "name",
            role.name.clone(),
            "permissions",
            role.permissions.bits() as usize
        ];

        self.hmset_and_forget(gen::role(guild_id, id), hashes.into_array());
    }

    pub fn upsert_voice_state<'a>(
        &'a self,
        guild_id: u64,
        state: &'a VoiceState,
    ) {
        let user_id = state.user_id.0;
        let key = gen::user_voice_state(guild_id, user_id);

        if let Some(channel_id) = state.channel_id {
            let mut values = resp_array![
                "channel_id",
                channel_id.0 as usize,
                "mute",
                usize::from(state.mute),
                "self_deaf",
                usize::from(state.self_deaf),
                "self_mute",
                usize::from(state.self_mute),
                "session_id",
                state.session_id.clone(),
                "suppress",
                usize::from(state.suppress)
            ];

            if let Some(token) = state.token.as_ref() {
                values.push("token".to_owned()).push(token);
            } else {
                self.hdel_and_forget(key.clone(), vec!["token"]);
            }

            self.hmset_and_forget(key, values.into_array());
        } else {
            self.del_and_forget(key);
        }
    }

    fn set_guild_channels(
        &self,
        guild_id: u64,
        channel_ids: Vec<usize>,
    ) {
        let key = gen::guild_channels(guild_id);

        self.del_and_forget(key.clone());
        self.sadd_and_forget(key, channel_ids);
    }

    fn set_guild_features(
        &self,
        guild_id: u64,
        features: Vec<String>,
    ) {
        let features_key = gen::guild_features(guild_id);

        self.del_and_forget(features_key.clone());
        self.sadd_and_forget(features_key, features);
    }

    fn set_guild_members(
        &self,
        guild_id: u64,
        members: Vec<usize>,
    ) {
        let key = gen::guild_members(guild_id);

        self.del_and_forget(key.clone());
        self.sadd_and_forget(key, members);
    }

    fn set_guild_roles(
        &self,
        guild_id: u64,
        roles: Vec<usize>,
    ) {
        let key = gen::guild_roles(guild_id);

        self.del_and_forget(key.clone());
        self.sadd_and_forget(key, roles);
    }

    fn set_guild_voice_states(
        &self,
        guild_id: u64,
        voice_states: Vec<usize>,
    ) {
        let key = gen::guild_voice_states(guild_id);

        self.del_and_forget(key.clone());
        self.sadd_and_forget(key, voice_states);
    }

    fn set_member_roles(
        &self,
        guild_id: u64,
        user_id: u64,
        roles: Vec<usize>,
    ) {
        let key = gen::member_roles(guild_id, user_id);

        self.del_and_forget(key.clone());
        self.sadd_and_forget(key, roles);
    }
}
