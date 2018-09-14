pub fn user_voice_state(guild_id: u64, user_id: u64) -> String {
    format!("vs:{}:{}", guild_id, user_id)
}

#[cfg(test)]
mod tests {
    #[test]
    fn user_voice_state() {
        assert_eq!(super::user_voice_state(1, 2), "vs:1:2");
        assert_eq!(
            super::user_voice_state(381880193251409931, 114941315417899012),
            "vs:381880193251409931:114941315417899012",
        );
    }
}
