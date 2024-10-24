use chrono::DateTime;
use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use crate::subscription::Subscription;
use super::user_role::UserRole;
///
/// User, representing telegram user, subscribed on the notices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(alias = "user_id", alias = "chat_id")]
    #[serde(with = "chat_id")]
    pub id: ChatId,
    pub name: String,
    pub contact: Option<String>,
    pub address: Option<String>,
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
    #[serde(with = "last_seen")]
    #[serde(skip_serializing)]
    pub last_seen: Option<DateTime<chrono::Utc>>,
    pub role: Vec<UserRole>,
}
///
/// Parse chat_id from / to string
mod chat_id {
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use teloxide::types::ChatId;
    pub fn serialize<S: Serializer>(v: &ChatId, serializer: S) -> Result<S::Ok, S::Error> {
    //   let v = v.as_ref().and_then(|v| Some(v.two.clone()));
        String::serialize(&v.0.to_string(), serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>, E: de::Error>(deserializer: D) -> Result<ChatId, E> {
        String::deserialize(deserializer)
            .and_then(|str| {
                match str.parse() {
                    Ok(parsed) => Ok(ChatId(parsed)),
                    Err(err) => Err(de::Error::custom(format!("{:?}", err))),
                }
            })
            .map_err(|err| de::Error::custom(format!("{:?}", err)))
    }
}
///
/// Parse chat_id from / to string
mod last_seen {
    use chrono::{DateTime, Utc};
    use serde::{de, Deserialize, Deserializer};
    // pub fn serialize<S: Serializer>(v: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error> {
    // //   let v = v.as_ref().and_then(|v| Some(v.two.clone()));
    //     String::serialize(&v.0.to_string(), serializer)
    // }
    pub fn deserialize<'de, D: Deserializer<'de>, E: de::Error>(deserializer: D) -> Result<Option<DateTime<Utc>>, E> {
        String::deserialize(deserializer)
            .and_then(|str| {
                match DateTime::parse_from_rfc3339(&str) {
                    Ok(datetime) => Ok(Some(datetime.with_timezone(&Utc))),
                    Err(err) => Err(de::Error::custom(format!("{:?}", err))),
                }
            })
            .map_err(|err| de::Error::custom(format!("{:?}", err)))
    }
}
