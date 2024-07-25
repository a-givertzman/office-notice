/* ===============================================================================
Restaurant menu bot.
User interface with inline buttons. 27 May 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2020-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use std::collections::HashMap;

use teloxide::{
    prelude::*,
    types::{InputFile, InlineKeyboardButton, InlineKeyboardMarkup,
       CallbackQuery, InputMedia, ParseMode, InputMediaPhoto,
    },
};
use crate::callback::Command;
use arraylib::iter::IteratorExt;

use crate::environment as env;
use crate::states::*;
use crate::db as db;
use crate::subscription::*;
use crate::loc::*;
///
/// 
pub async fn enter(bot: Bot, msg: Message, state: MainState) -> HandlerResult {
    let user_id = state.user_id; // user needs to sync with cart
    let chat_id = msg.chat.id;
    // Load groups
    let subscriptions =  db::subscriptions(user_id).await?;
    // All is ok, collect and display info
    let menu = HashMap::from([
        ("Links", "/Links"),
        ("Notice", "/Notice"),
        ("Subscribe", "/Subscribe"),
    ]);
    let markup = markup(&subscriptions, user_id).await?;
    let text = "Main menu";
    bot.send_message(chat_id, text)
    // .caption(text)
    .reply_markup(markup)
    .parse_mode(ParseMode::Html)
    // .disable_notification(true)
    .await?;
        Ok(())
}
///
///
async fn msg(bot: &Bot, user_id: UserId, text: &str) -> Result<(), String> {
    bot.send_message(user_id, text)
    .await
    .map_err(|err| format!("inline::msg {}", err))?;
    Ok(())
}
///
/// 
pub async fn view(bot: &Bot, q: CallbackQuery) -> Result<(), String> {
    let user_id = q.from.id;
    log::debug!("menu.view | user_id: {}", user_id);
    // Load from storage
    let subscription =  db::subscriptions(user_id).await?;
    // // Collect info
    let markup = markup(&subscription, user_id).await?;
    let text = format!("Navigation view");
    // Message to modify
    let message = q.message;
    if message.is_none() {
       // "Error, update message is invalid, please start again"
       let text = loc("Error, update message is invalid, please start again");
       msg(bot, user_id, &text).await?;
       return Ok(())
    }
    // let chat_id = ChatId::Id(message.chat_id());
    let message_id = message.unwrap().id;
    msg(bot, user_id, &text).await?;
    bot.edit_message_text(user_id, message_id, text)
        // .edit_message_media(user_id, message_id, media)
        .reply_markup(markup)
    .await
    .map_err(|err| format!("inline::view {}", err))?;
    Ok(())
}
///
/// 
async fn markup(menu: &HashMap<String, Subscription>, user_id: UserId) -> Result<InlineKeyboardMarkup, String> {
    // Create buttons for each group
    let buttons: Vec<InlineKeyboardButton> = menu
    .iter()
    .map(|(group_id, group)| {
        InlineKeyboardButton::callback(
        group.title.clone(),
        group_id
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
        format!("Back")
    );
    last_row.push(button_back);
    // Add the last unpaired button and the back button
    if !last_row.is_empty() {
        markup = markup.append_row(last_row);
    }
    Ok(markup)
}