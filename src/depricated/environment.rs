use std::env;

use chrono::FixedOffset;
use once_cell::sync::OnceCell;
use teloxide::{
    prelude::*, types::{Recipient, ChatId, MessageId},
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
//
//

impl Vars {
    pub async fn from_env(service_bot: Bot) -> Self {
        // == main body
        // Link to bot for advertise from its name
        let link = service_bot
        .get_me()
        .await
        .map_err(|_| ())
        .and_then(|me| {
            match me.user.username {
                Some(name) => Ok(format!("http://t.me/{}?start=", name)),
                None => Err(()),
            }
        });
        let link = link.unwrap_or(String::from("Ошибка"));
    
        // Service chat
        let chat = if let Ok(log_group_id_env) = env::var("LOG_GROUP_ID") {
            if let Ok(log_group_id) = log_group_id_env.parse::<i64>() {
                // Save id and bot
                let id = ChatId(log_group_id);
                Some(ServiceChat {
                    recipient: Recipient::Id(id),
                    bot: service_bot,
                })
            } else {
                log::info!("Environment variable LOG_GROUP_ID must be integer");
                None
            }
        } else {
            log::info!("There is no environment variable LOG_GROUP_ID, no service chat");
            None
        };
    
        Vars {
            // Admins contact for customer
            admin_contact_info: {
                match env::var("CONTACT_INFO") {
                    Ok(s) => {
                        log::info!("admin name is {}", s);
                        s
                    }
                    Err(e) => {
                        log(&format!("Something wrong with CONTACT_INFO: {}", e)).await;
                        String::default()
                    }
                }
            },
    
            // User id of admins
            admin_id1: {
                match env::var("TELEGRAM_ADMIN_ID1") {
                    Ok(s) => match s.parse::<u64>() {
                        Ok(n) => n,
                        Err(e) => {
                            log(&format!("Something wrong with TELEGRAM_ADMIN_ID1: {}", e)).await;
                            0
                        }
                    }
                    Err(e) => {
                    log(&format!("Something wrong with TELEGRAM_ADMIN_ID1: {}", e)).await;
                    0
                    }
                }
            },
    
            admin_id2: {
                match env::var("TELEGRAM_ADMIN_ID2") {
                    Ok(s) => if s.is_empty() {0} else {
                    match s.parse::<u64>() {
                        Ok(n) => n,
                        Err(e) => {
                            log(&format!("Something wrong with TELEGRAM_ADMIN_ID2: {}", e)).await;
                            0
                        }
                    }
                    }
                    Err(_) => 0 // if the variable is not set, that's ok
                }
            },
    
            admin_id3: {
                match env::var("TELEGRAM_ADMIN_ID3") {
                    Ok(s) => if s.is_empty() {0} else {
                    match s.parse::<u64>() {
                        Ok(n) => n,
                        Err(e) => {
                            log(&format!("Something wrong with TELEGRAM_ADMIN_ID3: {}", e)).await;
                            0
                        }
                    }
                    }
                    Err(_) => 0 // if the variable is not set, that's ok
                }
            },
    
            // Price suffix
            price_unit: {
                match env::var("PRICE_UNIT") {
                    Ok(s) => s,
                    Err(e) => {
                    log(&format!("Something wrong with PRICE_UNIT: {}", e)).await;
                    String::default()
                    }
                }
            },
    
            // Time zone, UTC
            time_zone: {
                match env::var("TIME_ZONE") {
                    Ok(s) => match s.parse::<i32>() {
                        Ok(n) => FixedOffset::east_opt(n * 3600),
                        Err(e) => {
                            log(&format!("Something wrong with TIME_ZONE: {}", e)).await;
                            FixedOffset::east_opt(0)
                        }
                    }
                    Err(e) => {
                    log(&format!("Something wrong with TIME_ZONE: {}", e)).await;
                    FixedOffset::east_opt(0)
                    }
                }
            },
    
            link,
            chat,
        }
    }
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
    log::info!("environment::log | service message: {:?}", text);
    match &VARS.get() {
        Some(vars) => {
            if let Some(chat) = &vars.chat {
                chat.send(text, None).await
            } else {
                log::info!("environment::log | Unable to send message to the service chat");
                None
            }
        }
        None => {
            log::info!("environment::log | Unable to send message to the service chat");
            None
        }
    }
}
 