pub fn channel(id: u64) -> String {
    format!("ch:{}", id)
}

pub fn channel_voice_states(id: u64) -> String {
    format!("ch:{}:v", id)
}

pub fn choice(id: u64) -> String {
    format!("c:{}", id)
}

pub fn join(id: u64) -> String {
    format!("j:{}", id)
}

pub fn guild(id: u64) -> String {
    format!("g:{}", id)
}

pub fn guild_channels(id: u64) -> String {
    format!("g:{}:c", id)
}

pub fn guild_features(id: u64) -> String {
    format!("g:{}:f", id)
}

pub fn guild_members(id: u64) -> String {
    format!("g:{}:m", id)
}

pub fn guild_roles(id: u64) -> String {
    format!("g:{}:r", id)
}

pub fn guild_voice_states(guild_id: u64) -> String {
    format!("g:{}:v", guild_id)
}

pub fn queue(guild_id: u64) -> String {
    format!("queue:{}", guild_id)
}

pub fn loop_mode(guild_id: u64) -> String {
    format!("loop:{}:mode", guild_id)
}

pub fn member(guild_id: u64, user_id: u64) -> String {
    format!("g:{}:m:{}", guild_id, user_id)
}

pub fn member_roles(guild_id: u64, user_id: u64) -> String {
    format!("g:{}:m:{}:r", guild_id, user_id)
}

pub fn role(guild_id: u64, role_id: u64) -> String {
    format!("g:{}:r:{}", guild_id, role_id)
}

pub fn user_voice_state(guild_id: u64, user_id: u64) -> String {
    format!("g:{}:v:{}", guild_id, user_id)
}

pub fn sharder_to(shard_id: u64) -> String {
    format!("sharder:to:{}", shard_id)
}

#[cfg(test)]
mod tests {
    #[test]
    fn channel() {
        assert_eq!(super::channel(381880193700069377), "ch:381880193700069377");
    }

    #[test]
    fn channel_voice_states() {
        assert_eq!(super::channel_voice_states(2), "ch:2:v");
    }

    #[test]
    fn choice() {
        assert_eq!(super::choice(272410239947767808), "c:272410239947767808");
    }

    #[test]
    fn guild() {
        assert_eq!(super::guild(381880193251409931), "g:381880193251409931");
    }

    #[test]
    fn guild_channels() {
        assert_eq!(super::guild_channels(2), "g:2:c");
    }

    #[test]
    fn guild_features() {
        assert_eq!(super::guild_features(2), "g:2:f");
    }

    #[test]
    fn guild_members() {
        assert_eq!(super::guild_members(3), "g:3:m");
    }

    #[test]
    fn guild_roles() {
        assert_eq!(super::guild_roles(3), "g:3:r");
    }

    #[test]
    fn guild_voice_states() {
        assert_eq!(super::guild_voice_states(1), "g:1:v");
    }

    #[test]
    fn queue() {
        assert_eq!(super::queue(272410239947767808), "queue:272410239947767808");
    }

    #[test]
    fn loop_mode() {
        assert_eq!(super::loop_mode(272410239947767808), "loop:272410239947767808:mode");
    }

    #[test]
    fn member() {
        assert_eq!(super::member(1, 2), "g:1:m:2");
    }

    #[test]
    fn member_roles() {
        assert_eq!(super::member_roles(1, 2), "g:1:m:2:r");
    }

    #[test]
    fn user_voice_state() {
        assert_eq!(super::user_voice_state(1, 2), "g:1:v:2");
        assert_eq!(
            super::user_voice_state(381880193251409931, 114941315417899012),
            "g:381880193251409931:v:114941315417899012",
        );
    }

    #[test]
    fn join() {
        assert_eq!(super::join(272410239947767808), "j:272410239947767808");
    }

    #[test]
    fn sharder_to() {
        assert_eq!(super::sharder_to(1337), "sharder:to:1337");
    }
}
