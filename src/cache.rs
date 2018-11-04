use crate::{
    commands::CommandablePairedConnection,
    error::{Error, Result},
    gen,
    model::VoiceState as CachedVoiceState,
    resp_impl::RespValueExt as _,
};
use essentials::result::ResultExt as _;
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
    inner: CommandablePairedConnection,
}

impl Cache {
    /// Creates a new cache accessing instance.
    pub fn new(redis: Arc<PairedConnection>) -> Self {
        Self {
            inner: CommandablePairedConnection::new(redis),
        }
    }

    /// Returns the inner commandable paired connection for use in lower level
    /// data manipulation.
    pub fn inner(&self) -> &CommandablePairedConnection {
        &self.inner
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
        self.delete_voice_state_atomic(guild_id, user_id);

        // Remove the user's ID from the guild's voice state set.
        let deleted = await!(self.inner.srem(
            gen::guild_voice_states(guild_id),
            vec![user_id as usize],
        ))?;

        Ok(deleted.into_array().len() > 0)
    }

    fn delete_voice_state_atomic(
        &self,
        guild_id: u64,
        user_id: u64,
    ) {
        self.inner.del_sync(gen::user_voice_state(guild_id, user_id))
    }

    fn delete_voice_state_list(
        &self,
        guild_id: u64,
    ) {
        self.inner.del_sync(gen::guild_voice_states(guild_id))
    }

    /// Deletes all of the voice states for a guild.
    ///
    /// Returns the number of voice states deleted.
    pub async fn delete_voice_states(
        &self,
        guild_id: u64,
    ) -> Result<u64> {
        let ids = await!(self.get_voice_state_list(guild_id)).unwrap_or_default();

        let count = ids.len();

        for id in ids {
            self.delete_voice_state_atomic(guild_id, id);
        }

        self.delete_voice_state_list(guild_id);

        Ok(count as u64)
    }

    /// Returns a voice state for a guild member, if one exists for them.
    pub async fn get_voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> Result<Option<CachedVoiceState>> {
        let value: Vec<RespValue> = await!(self.inner.send(resp_array![
            "HGETALL",
            gen::user_voice_state(guild_id, user_id)
        ]))?;

        if value.is_empty() {
            return Ok(None);
        }

        FromResp::from_resp(RespValue::Array(value)).map(Some).into_err()
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

    /// Gets the IDs of all members that have a voice state in a channel.
    pub async fn get_channel_voice_states(
        &self,
        channel_id: u64,
    ) -> Result<Vec<u64>> {
        let ids = await!(self.inner.smembers::<Vec<String>>(gen::channel_voice_states(channel_id)))?;

        let mut numbers = Vec::with_capacity(ids.len());

        for id in ids {
            numbers.push(id.parse()?);
        }

        Ok(numbers)
    }

    /// Gets the IDs of all members that have a voice state in a guild.
    pub async fn get_voice_state_list(
        &self,
        guild_id: u64,
    ) -> Result<Vec<u64>> {
        let resp = await!(self.inner.get(gen::guild_voice_states(guild_id)))?;

        if resp == RespValue::Nil {
            return Ok(vec![]);
        }

        FromResp::from_resp(resp).into_err()
    }

    /// Gets the choices available for a guild.
    pub async fn get_choices(
        &self,
        guild_id: u64,
    ) -> Result<Vec<String>> {
        let resp = await!(self.inner.get(gen::choice(guild_id)))?;

        if resp == RespValue::Nil {
            return Ok(vec![]);
        }

        FromResp::from_resp(resp).into_err()
    }

    /// Gets the choices availble for a guild within `min <= entry <= max`.
    pub async fn get_choices_ranged(
        &self,
        guild_id: u64,
        min: i64,
        max: i64,
    ) -> Result<Vec<String>> {
        let resp = await!(self.inner.lrange(gen::choice(guild_id), min, max))?;

        if resp == RespValue::Nil {
            return Ok(vec![]);
        }

        FromResp::from_resp(resp).into_err()
    }

    /// Deletes the choices of a guild.
    pub async fn delete_choices(
        &self,
        guild_id: u64,
    ) -> Result<()> {
        await!(self.inner.del(gen::choice(guild_id)))
    }

    /// Pushes choice alternatives for a guild.
    pub async fn push_choices(
        &self,
        guild_id: u64,
        blobs: Vec<String>,
    ) -> Result<()> {
        await!(self.inner.lpush(gen::choice(guild_id), blobs))
    }

    /// Gets the channel the bot is in, in a guild.
    pub async fn get_join(
        &self,
        guild_id: u64,
    ) -> Result<String> {
        await!(self.inner.get(gen::join(guild_id)))
    }

    /// Sets the channel to join of a guild.
    pub async fn set_join(
        &self,
        guild_id: u64,
        channel: u64,
    ) -> Result<i64> {
        await!(self.inner.set(gen::join(guild_id), vec![channel]))
    }

    /// Deletes the join value of a guild.
    pub async fn delete_join(
        &self,
        guild_id: u64,
    ) -> Result<()> {
        await!(self.inner.del(gen::join(guild_id)))
    }

    /// Sends a message to the sharder from a shard.
    pub async fn sharder_msg(
        &self,
        shard_id: u64,
        data: Vec<u8>,
    ) -> Result<()> {
        await!(self.inner.rpush(gen::sharder_to(shard_id), data))
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
        let values = await!(self.inner.send::<Vec<Vec<_>>>(arr))?;

        let mut ids = ids.into_iter();
        let mut map = HashMap::with_capacity(pair_len);

        for value in values {
            map.insert(ids.next()?, serde_json::from_slice(&value)?);
        }

        Ok(map)
    }

    pub fn delete_channel(&self, id: u64) {
        self.inner.del_sync(gen::channel(id))
    }

    pub fn delete_channels<'a>(
        &'a self,
        ids: impl IntoIterator<Item = u64> + 'a,
    ) {
        self.inner.delm_sync(ids.into_iter().map(gen::channel))
    }

    pub fn delete_guild(&self, id: u64) {
        self.inner.del_sync(gen::guild(id))
    }

    pub fn delete_guilds(
        &self,
        ids: impl IntoIterator<Item = u64>,
    ) {
        self.inner.delm_sync(ids.into_iter().map(gen::guild))
    }

    // pub async fn get_channel(&self, id: u64) -> Result<Channel> {
    //     await!(self.inner.get(gen::channel(id)))
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
        let values = await!(self.inner.hgetall(gen::guild(id)))?.into_array();

        if values.is_empty() {
            return Err(Error::None);
        }

        let mut values = RespValue::Array(values);

        let channels = await!(self.inner.smembers::<RespValue>(gen::guild_channels(id)))?;
        values.push("channels").push(channels);

        let features = await!(self.inner.smembers::<RespValue>(gen::guild_features(id)))?;
        values.push("features").push(features);

        let members = await!(self.inner.smembers::<RespValue>(gen::guild_members(id)))?;
        values.push("members").push(members);

        let roles = await!(self.inner.smembers::<RespValue>(gen::guild_roles(id)))?;
        values.push("roles").push(roles);

        let voice_states = await!(self.inner.smembers::<RespValue>(gen::guild_voice_states(id)))?;
        values.push("voice_states").push(voice_states);

        FromResp::from_resp(values).into_err()
    }

    pub async fn upsert_channel<'a>(
        &'a self,
        channel: &'a Channel,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(channel)?;

        await!(self.inner.set(gen::channel(channel.id().0), vec![bytes]))?;

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
        self.inner.send_sync(set);
        info!("Guild upsert HMSET successful");

        if let Some(del) = del {
            info!("Sending guild upsert HDEL");
            self.inner.hdel_sync(gen::guild(gid), del);
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

        let channel_states: HashMap<u64, Vec<usize>> = guild.voice_states
            .values()
            .fold(HashMap::new(), |mut acc, state| {
                let cid = match state.channel_id {
                    Some(id) => id.0,
                    None => return acc,
                };

                acc.entry(cid).or_default().push(state.user_id.0 as usize);

                return acc;
            });

        for (id, user_ids) in channel_states {
            self.set_channel_voice_states(id, user_ids);
        }

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
            self.inner.hdel_sync(
                gen::member(guild_id, user_id),
                vec!["afk_channel_id"],
            );
        }

        self.inner.hmset_sync(gen::member(guild_id, user_id), set.into_array());

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

        self.inner.hmset_sync(gen::role(guild_id, id), hashes.into_array());
    }

    pub async fn upsert_voice_state<'a>(
        &'a self,
        guild_id: u64,
        state: &'a VoiceState,
    ) -> Result<()> {
        let user_id = state.user_id.0;
        let key = gen::user_voice_state(guild_id, user_id);

        trace!("Getting old voice state");
        let old_state = await!(self.get_voice_state(guild_id, user_id))?;
        trace!("Got old voice state: {:?}", old_state);

        if let Some(channel_id) = state.channel_id {
            let channel_id = channel_id.0;
            trace!("Voice state has a channel ID: {}", channel_id);

            let mut values = resp_array![
                "channel_id",
                channel_id as usize,
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
                self.inner.hdel_sync(key.clone(), vec!["token"]);
            }

            self.inner.hmset_sync(key, values.into_array());

            let mut add_member = true;

            if let Some(old_cid) = old_state.map(|s| s.channel_id) {
                trace!("Old voice state exists and has a channel ID");

                if old_cid != channel_id {
                    trace!("Old channel ID is different from new");

                    self.inner.srem_sync(
                        gen::channel_voice_states(old_cid),
                        vec![user_id as usize],
                    );
                } else {
                    add_member = false;
                }
            }

            if add_member {
                self.inner.sadd_sync(
                    gen::channel_voice_states(channel_id),
                    vec![user_id as usize],
                );
            }
        } else {
            trace!("No channel ID for voice state");
            if let Some(channel_id) = old_state.map(|s| s.channel_id) {
                trace!("Deleting old voice state for channel {}", channel_id);

                self.inner.srem_sync(
                    gen::channel_voice_states(channel_id),
                    vec![user_id as usize],
                );
            }

            self.inner.srem_sync(
                gen::guild_voice_states(guild_id),
                vec![user_id as usize],
            );
            self.inner.del_sync(key);
        }

        Ok(())
    }

    pub fn upsert_voice_state_info<'a>(
        &'a self,
        guild_id: u64,
        user_id: u64,
        endpoint: String,
        token: String,
    ) {
        let key = gen::user_voice_state(guild_id, user_id);

        self.inner.hmset_sync(key, resp_array![
            "endpoint",
            endpoint,
            "token",
            token
        ].into_array());
    }

    fn set_channel_voice_states(
        &self,
        channel_id: u64,
        user_ids: Vec<usize>,
    ) {
        let key = gen::channel_voice_states(channel_id);

        self.inner.del_sync(key.clone());
        self.inner.sadd_sync(key, user_ids);
    }

    fn set_guild_channels(
        &self,
        guild_id: u64,
        channel_ids: Vec<usize>,
    ) {
        let key = gen::guild_channels(guild_id);

        self.inner.del_sync(key.clone());
        self.inner.sadd_sync(key, channel_ids);
    }

    fn set_guild_features(
        &self,
        guild_id: u64,
        features: Vec<String>,
    ) {
        let features_key = gen::guild_features(guild_id);

        self.inner.del_sync(features_key.clone());
        self.inner.sadd_sync(features_key, features);
    }

    fn set_guild_members(
        &self,
        guild_id: u64,
        members: Vec<usize>,
    ) {
        let key = gen::guild_members(guild_id);

        self.inner.del_sync(key.clone());
        self.inner.sadd_sync(key, members);
    }

    fn set_guild_roles(
        &self,
        guild_id: u64,
        roles: Vec<usize>,
    ) {
        let key = gen::guild_roles(guild_id);

        self.inner.del_sync(key.clone());
        self.inner.sadd_sync(key, roles);
    }

    fn set_guild_voice_states(
        &self,
        guild_id: u64,
        voice_states: Vec<usize>,
    ) {
        let key = gen::guild_voice_states(guild_id);

        self.inner.del_sync(key.clone());
        self.inner.sadd_sync(key, voice_states);
    }

    fn set_member_roles(
        &self,
        guild_id: u64,
        user_id: u64,
        roles: Vec<usize>,
    ) {
        let key = gen::member_roles(guild_id, user_id);

        self.inner.del_sync(key.clone());
        self.inner.sadd_sync(key, roles);
    }
}
