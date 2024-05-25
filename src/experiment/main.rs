use chat::Message;
use serde::Serialize;
fn main() {
    let msg = Message::JoinRequest {
        username: "Yyvan".to_string(),
    };

    println!("source = {:?}", msg);

    let serialized = serde_json::to_string(&msg).unwrap();
    println!("serialized = {serialized}");

    let deserialized: Message = serde_json::from_str(serialized.as_str()).unwrap();
    println!("deserialized = {:?}", deserialized)
}
