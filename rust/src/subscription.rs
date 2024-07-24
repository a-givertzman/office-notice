use serde::{Deserialize, Serialize};
use teloxide::types::UserId;
///
/// 
#[derive(Clone, Serialize, Deserialize)]
pub struct Owners (pub UserId, pub UserId, pub UserId);
///
/// 
#[derive(Clone, Serialize, Deserialize)]
pub struct Subscription {
   pub id: i32,  // zero for a new, not saved in database yet or for root
   pub parent: i32,
   pub children: Vec<Subscription>,
   pub title: String,
   pub descr: String,
   pub enabled: bool,
   pub banned: bool,
   pub owners: Owners,
   pub price: usize,
}
