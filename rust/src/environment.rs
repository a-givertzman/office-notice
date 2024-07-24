use chrono::{FixedOffset, NaiveDateTime, Utc,};
use once_cell::sync::{OnceCell};
use teloxide::{
    prelude::*, types::{Recipient, ChatId, UserId, MessageId},
};
///
/// Settings
pub static VARS: OnceCell<Vars> = OnceCell::new();
///
/// Enviroment variables
pub struct Vars {
    // Service chat
    chat: Option<ServiceChat>,
    // Admins contact for customer
    admin_contact_info: String,
    // User id of admins
    admin_id1: u64,
    admin_id2: u64,
    admin_id3: u64,
    // Price suffix
    price_unit: String,
    // Time zone, UTC
    time_zone: Option<FixedOffset>,
    // Link for open node from /start http://t.me/{bot name} ?start=
    link: String,
}
///
///
// For send info to service chat
#[derive(Clone)]
struct ServiceChat {
    recipient: Recipient,
    bot: Bot,
}
///
///
impl ServiceChat {
    async fn send(&self, text: &str, reply_to: Option<MessageId>) -> Option<MessageId> {
        // Prepare to send text
        let mut res = self.bot
            .send_message(self.recipient.to_owned(), text)
            .disable_notification(true);
        // Quoted message
        if let Some(reply_to) = reply_to {
            res = res.reply_to_message_id(reply_to);
        }
        match res.await {
            Ok(m) => Some(m.id),
            Err(err) => {
                log::info!("ServiceChat.send | Error log({}): {}", text, err);
                None
            }
        }
    }
}
///
/// Send message to service chat without notification
pub async fn log(text: &str) -> Option<MessageId> {
    log::info!("environment::log | Unable to send message to the service chat");
    if let Some(chat) = &VARS.get().unwrap().chat {
       chat.send(text, None).await
    } else {
       None
    }
}
///
/// Checking if the user id is admin
pub fn is_admin_id(user_id: UserId) -> bool {
    let vars = VARS.get().unwrap();
    let user_id = user_id.0;
    user_id == vars.admin_id1 || user_id == vars.admin_id2 || user_id == vars.admin_id3
}
 