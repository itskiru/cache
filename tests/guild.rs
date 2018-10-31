#![feature(async_await, await_macro, futures_api)]

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use dabbot_cache::Cache;
use futures::{
    compat::Future01CompatExt,
    future::{FutureExt, TryFutureExt},
};
use redis_async::client;
use serenity::{
    model::prelude::*,
};
use std::{
    collections::{HashMap, HashSet},
    env::self,
    error::Error as StdError,
    net::{SocketAddr, SocketAddrV4, Ipv4Addr},
    str::FromStr,
    sync::Arc,
};
use tokio;

fn now() -> DateTime<FixedOffset> {
    DateTime::from_utc(
        NaiveDateTime::from_timestamp(1, 0),
        FixedOffset::east(0),
    )
}

fn panic(err: Box<StdError + 'static>) -> () {
    panic!("err: {:?}", err);
}

async fn client() -> Result<Cache, Box<StdError + 'static>> {
    let host = Ipv4Addr::from_str(&env::var("REDIS_HOST")?)?;
    let port = env::var("REDIS_PORT")?.parse()?;

    let client = await!(client::paired_connect(
        &SocketAddr::V4(SocketAddrV4::new(host, port)),
    ).compat())?;

    Ok(Cache::new(Arc::new(client)))
}

#[test]
fn retrieval() {
    async fn _get_guild() -> Result<(), Box<StdError + 'static>> {
        let client = await!(client())?;

        let guild = Guild {
            afk_channel_id: Some(ChannelId(2)),
            afk_timeout: 900,
            application_id: None,
            channels: {
                let mut map = HashMap::new();

                map.insert(ChannelId(4), GuildChannel {
                    id: ChannelId(4),
                    bitrate: Some(86400),
                    category_id: Some(ChannelId(3)),
                    guild_id: GuildId(1),
                    kind: ChannelType::Voice,
                    last_message_id: None,
                    last_pin_timestamp: None,
                    name: "some-channel".to_owned(),
                    permission_overwrites: vec![PermissionOverwrite {
                        allow: Permissions::all(),
                        deny: Permissions::SEND_MESSAGES,
                        kind: PermissionOverwriteType::Member(UserId(5)),
                    }],
                    position: 2,
                    topic: Some("a topic".to_owned()),
                    user_limit: 99.into(),
                    nsfw: false,
                });

                map
            },
            default_message_notifications: DefaultMessageNotificationLevel::Mentions,
            emojis: HashMap::new(),
            explicit_content_filter: ExplicitContentFilter::None,
            features: vec![],
            icon: None,
            id: GuildId(1),
            joined_at: now(),
            large: false,
            member_count: 1,
            members: {
                let mut map = HashMap::new();

                map.insert(UserId(5), Member {
                    deaf: false,
                    guild_id: GuildId(1),
                    joined_at: None,
                    mute: false,
                    nick: None,
                    roles: vec![RoleId(6)],
                    user: User {
                        id: UserId(5),
                        avatar: None,
                        bot: false,
                        discriminator: 1,
                        name: "hello".to_owned(),
                    },
                });

                map
            },
            mfa_level: MfaLevel::Elevated,
            name: "a guild".to_owned(),
            owner_id: UserId(5),
            presences: HashMap::new(),
            region: "us-west".to_owned(),
            roles: {
                let mut map = HashMap::new();

                map.insert(RoleId(6), Role {
                    id: RoleId(6),
                    colour: 1u64.into(),
                    hoist: true,
                    managed: false,
                    mentionable: true,
                    name: "a role".to_owned(),
                    permissions: Permissions::MOVE_MEMBERS,
                    position: 1,
                });

                map
            },
            splash: None,
            system_channel_id: None,
            verification_level: VerificationLevel::High,
            voice_states: {
                let mut map = HashMap::new();

                map.insert(UserId(5), VoiceState {
                    channel_id: Some(ChannelId(4)),
                    deaf: true,
                    mute: true,
                    self_deaf: true,
                    self_mute: true,
                    session_id: "a string".to_owned(),
                    suppress: false,
                    token: None,
                    user_id: UserId(5),
                });

                map
            },
        };
        await!(client.upsert_guild(&guild))?;
        let guild = await!(client.get_guild(1))?;

        assert_eq!(guild.afk_channel_id, Some(2));
        assert_eq!(guild.channels, {
            let mut set = HashSet::with_capacity(1);
            set.insert(4);
            set
        });

        client.delete_guild(1);

        Ok(())
    }

    tokio::run(_get_guild().map_err(panic).boxed().compat());
}
