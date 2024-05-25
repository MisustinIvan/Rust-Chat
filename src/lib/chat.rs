use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum JoinFailureReason {
    UsernameInUse,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    JoinRequest { username: String },
    JoinResponseSuccess { user_id: u32 },
    JoinResponseFailure { reason: JoinFailureReason },
    Message { user_id: u32, content: String },
}
