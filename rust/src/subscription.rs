use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::user::User;
///
/// 
pub type Subscriptions = IndexMap<String, Subscription>;
///
/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
   pub title: String,
   pub members: IndexMap<String, User>,
}
