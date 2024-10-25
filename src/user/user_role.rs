use serde::{Deserialize, Serialize};
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
