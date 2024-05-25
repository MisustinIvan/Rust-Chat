use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum JoinFailureReason {
    UsernameInUse,
}

impl ToString for JoinFailureReason {
    fn to_string(&self) -> String {
        match self {
            JoinFailureReason::UsernameInUse => "username in use".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    JoinRequest {
        username: String,
    },
    JoinResponseSuccess {
        user_id: u32,
    },
    JoinResponseFailure {
        reason: JoinFailureReason,
    },
    Message {
        user_id: u32,
        content: String,
    },
    PrivateMessage {
        user_id: u32,
        target_name: String,
        content: String,
    },
    ListRequest {
        user_id: u32,
    },
}
