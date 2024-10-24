use serde::{Deserialize, Serialize};
///
/// User role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    #[serde(alias = "Admin", alias = "admin")]
    Admin,
    #[serde(alias = "Moder", alias = "moder")]
    Moder,
    #[serde(alias = "Member", alias = "member")]
    Member,
    #[serde(alias = "Guest", alias = "guest")]
    Guest,
}
