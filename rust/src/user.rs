use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use crate::subscription::Subscription;
///
/// 
#[derive(Clone, Serialize, Deserialize)]
pub struct User {
   #[serde(alias = "user_id", alias = "chat_id")]
   #[serde(with = "chat_id")]
   pub id: ChatId,
   pub name: String,
   pub contact: Option<String>,
   pub address: Option<String>,
   #[serde(default)]
   pub subscriptions: Vec<Subscription>,
}
///
/// 
mod chat_id {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use teloxide::types::ChatId;
    pub fn serialize<S: Serializer>(v: &ChatId, serializer: S) -> Result<S::Ok, S::Error> {
    //   let v = v.as_ref().and_then(|v| Some(v.two.clone()));
        String::serialize(&v.0.to_string(), serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<ChatId, D::Error> {
        String::deserialize(deserializer)
            .and_then(|str| Ok(ChatId(str.parse().unwrap())) )
    }
}
