#[derive(Clone, Debug)]
pub struct VoiceState {
    pub channel_id: u64,
    pub session_id: String,
    pub token: Option<String>,
}
