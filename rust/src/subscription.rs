use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use teloxide::{types::{ChatId, Message, UserId}, Bot};
use crate::user::User;
///
/// 
pub type Subscriptions = HashMap<String, Subscription>;
///
/// 
#[derive(Clone, Serialize, Deserialize)]
pub struct Owners (pub UserId, pub UserId, pub UserId);
///
/// 
#[derive(Clone, Serialize, Deserialize)]
pub struct Subscription {
   pub title: String,
   pub members: HashMap<String, User>,
}
