use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::user::user::User;
///
/// 
pub type Subscriptions = IndexMap<String, Subscription>;
///
/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
   #[serde(skip_serializing_if = "Option::is_none")]
   pub id: Option<String>,
   pub title: String,
   pub members: IndexMap<String, User>,
}
