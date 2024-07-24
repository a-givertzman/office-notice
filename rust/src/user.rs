use serde::{Deserialize, Serialize};
use crate::subscription::Subscription;
///
/// 
#[derive(Clone, Serialize, Deserialize)]
pub struct User {
   pub name: String,
   pub contact: String,
   pub address: String,
   pub subscriptions: Vec<Subscription>,
}
