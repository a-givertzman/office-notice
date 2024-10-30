use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
///
/// 
pub type UserRoles = IndexMap<String, UserRoleDb>;
///
/// User role
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    ///
    /// Administrative role, Full access
    #[serde(alias = "Admin", alias = "admin")]
    Admin,
    ///
    /// Moderator role, Allows / Denys New user
    #[serde(alias = "Moder", alias = "moder")]
    Moder,
    ///
    /// Sender role, Can send Notice's
    #[serde(alias = "Member", alias = "member")]
    Sender,
    ///
    /// Member role, Can Subscribe and receive Notice's
    #[serde(alias = "Member", alias = "member")]
    Member,
    ///
    /// Guest role, can request acces from Moderator's
    #[serde(alias = "Guest", alias = "guest")]
    Guest,
}
//
//
impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_owned(),
            UserRole::Moder => "moder".to_owned(),
            UserRole::Sender => "sender".to_owned(),
            UserRole::Member => "member".to_owned(),
            UserRole::Guest => "guest".to_owned(),
        }
    }
}
///
/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleDb {
   #[serde(skip_serializing_if = "Option::is_none")]
   pub id: Option<String>,
   pub title: String,
   pub role: UserRole,
}
