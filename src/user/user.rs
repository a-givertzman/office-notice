use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use crate::subscribe::subscription::Subscription;
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
    pub last_seen: String,
    pub role: Vec<UserRole>,
}
//
//
impl User {
    ///
    /// Returns User new instance
    pub fn new(
        id: ChatId,
        name: String,
        contact: Option<String>,
        address: Option<String>,
        subscriptions: Vec<Subscription>,
        last_seen: String,
        role: Vec<UserRole>,
    ) -> Self {
        Self { 
            id,
            name,
            contact,
            address,
            subscriptions,
            last_seen,
            role,
        }
    }
    ///
    /// 
    pub fn update(&mut self, other: Self) {
        self.id = other.id;
        self.name = other.name;
        self.contact = other.contact;
        self.address = other.address;
        self.subscriptions = other.subscriptions;
        self.last_seen = other.last_seen;
        self.role = other.role;
    }
    ///
    /// Returns true if `self.role` covers some of `role`
    pub fn has_role(&self, roles: &[UserRole]) -> bool {
        for role in &self.role {
            if roles.contains(role) {
                return true;
            }
        }
        false
    }
    ///
    /// Adds a role to user
    pub fn add_role(&mut self, role: UserRole) {
        if !self.role.contains(&role) {
            self.role.push(role);
        }
        if let Some(i) = self.role.iter().position(|r| *r == UserRole::Guest) {
            self.role.remove(i);
        }
    }
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
// ///
// /// Parse chat_id from / to string
// mod last_seen {
//     use chrono::{DateTime, Utc};
//     use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
//     pub fn serialize<S: Serializer>(v: &str, serializer: S) -> Result<S::Ok, S::Error> {
//     //   let v = v.as_ref().and_then(|v| Some(v.two.clone()));
//         match v {
//             Some(timestamp) => {
//                 String::serialize(&timestamp.to_rfc3339(), serializer)
//             },
//             None => SerializeStruct::skip_field(&mut serializer, key).map_err(|err| ),
//             // Err(S::Error::n "last_seen.serialize | Input is None"),
//         }
//     }
//     pub fn deserialize<'de, D: Deserializer<'de>, E: de::Error>(deserializer: D) -> Result<Option<DateTime<Utc>>, E> {
//         String::deserialize(deserializer)
//             .and_then(|str| {
//                 match DateTime::parse_from_rfc3339(&str) {
//                     Ok(timestamp) => Ok(Some(timestamp.with_timezone(&Utc))),
//                     Err(err) => Err(de::Error::custom(format!("{:?}", err))),
//                 }
//             })
//             .map_err(|err| de::Error::custom(format!("{:?}", err)))
//     }
// }
