/* ===============================================================================
Office menu bot.
User interface with inline buttons
=============================================================================== */
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};
use arraylib::iter::IteratorExt;
use crate::{states::*, user::{user::User, user_role::UserRole}};
use crate::db as db;
use crate::loc::*;
///
/// Main menu
#[derive(Debug, Clone, PartialEq)]
pub enum MainMenu {
   Links(String),      // Links menu
   Notice,     // Notice menu
   Subscribe,  // subscribe to receive notice
   RequestAccess,   // User request access
   Help,
   Done,       // Exit menu
   Unknown,
}
//
//
impl MainMenu {
   pub fn parse(s: &str, _loc_tag: LocaleTag) -> Self {
        match s.to_lowercase().as_str() {
            "/notice" | "/Notice" => Self::Notice,
            "/links" | "/Links" => Self::Links(s.to_owned()),
            "/subscribe" | "/Subscribe" => Self::Subscribe,
            "/requestaccess" | "/RequestAccess" => Self::RequestAccess,
            "/help" | "/Help" => Self::Help,
            "/done" | "/Done" => Self::Done,
            "/back" | "/Back" => Self::Done,
            "/exit" | "/Exit" => Self::Done,
            _ => Self::Unknown,
        }
   }
}
///
/// 
#[derive(Debug, Serialize, Deserialize)]
pub struct MenuItem {
    pub title: String,
    pub command: String,
}
///
/// Create a MainMenu
pub async fn enter(bot: &Bot, msg: &Message, user: &User) -> HandlerResult {
    let menu =  db::menu().await?;
    let markup = markup(user, &menu).await?;
    let text = "Main menu";
    bot.send_message(msg.chat.id, text)
        // .caption(text)
        .reply_markup(markup)
        .parse_mode(ParseMode::Html)
        // .disable_notification(true)
        .await?;
    Ok(())
}
///
/// Reloads a MainMenu
pub async fn reload(bot: &Bot, msg: &Message, user: &User) -> HandlerResult {
    let menu =  db::menu().await?;
    let markup = markup(user, &menu).await?;
    let text = "Main menu";
    crate::message::edit_message_text_or_send(bot, msg, &markup, text).await
}
///
/// Exits a MainMenu
pub async fn exit(bot: &Bot, msg: &Message, user: &User) -> HandlerResult {
    let text = format!("Bye, {}", user.name);
    bot.edit_message_text(msg.chat.id, msg.id, text)
        // .edit_message_media(user_id, message_id, media)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
///
/// Returns MainMenu items
async fn markup(user: &User, menu: &IndexMap<String, MenuItem>) -> Result<InlineKeyboardMarkup, String> {
    // Create buttons for each group
    let buttons: Vec<InlineKeyboardButton> = menu.iter()
        .filter(|(key, _menu_item)| {
            match key.as_str() {
                "RequestAccess" => {
                    user.has_role(&[UserRole::Guest])
                }
                "Links" => {
                    user.has_role(&[UserRole::Admin, UserRole::Moder, UserRole::Sender, UserRole::Member])
                }
                "Notice" => {
                    user.has_role(&[UserRole::Admin, UserRole::Moder, UserRole::Sender])
                }
                "Subscribe" => {
                    user.has_role(&[UserRole::Admin, UserRole::Moder, UserRole::Sender, UserRole::Member])
                }
                "Help" => {
                    user.has_role(&[UserRole::Admin, UserRole::Moder, UserRole::Sender, UserRole::Member])
                }
                _ => user.has_role(&[UserRole::Admin]),
            }
        })
        .map(|(_id, menu_item)| {
            InlineKeyboardButton::callback(
            menu_item.title.clone(),
            menu_item.command.clone(),
        )})
    .collect();
    // Separate into long and short
    let (long, mut short) : (Vec<_>, Vec<_>) = buttons
    .into_iter()
    .partition(|n| n.text.chars().count() > 21);
    // Put in vec last unpaired button, if any
    let mut last_row = vec![];
    if short.len() % 2 == 1 {
        let unpaired = short.pop();
        if unpaired.is_some() {
            last_row.push(unpaired.unwrap());
        }
    }
    // Long buttons by one in row
    let markup = long.into_iter()
    .fold(InlineKeyboardMarkup::default(), |acc, item| acc.append_row(vec![item]));
    // Short by two
    let mut markup = IteratorExt::array_chunks::<[_; 2]>(short.into_iter())
    .fold(markup, |acc, [left, right]| acc.append_row(vec![left, right]));
    // Back button
    let button_back = InlineKeyboardButton::callback(
        loc("⏪Back"), // "⏪Back"
        format!("/back")
    );
    last_row.push(button_back);
    // Add the last unpaired button and the back button
    if !last_row.is_empty() {
        markup = markup.append_row(last_row);
    }
    Ok(markup)
}
