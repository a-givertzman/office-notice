use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use teloxide::{dispatching::{dialogue::{self, InMemStorage}, UpdateFilterExt, UpdateHandler }, prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, ReplyMarkup, User, UserId}
};
use crate::{db, loc::loc, states::{HandlerResult, MainState, MyDialogue, StartState}};
///
/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub title: String,
    pub url: String,
}
///
/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Links {
    pub title: Option<String>,
    pub links: Vec<Link>,
    #[serde(default)]
    pub child: IndexMap<String, Links>,
}
///
/// 
#[derive(Debug, Clone)]
pub struct LinksState {
    pub prev_state: MainState,
    pub prev_level: Option<String>,
    pub level: String,
    pub child: IndexMap<String, Links>,
    pub user_id: UserId,
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, mut state: LinksState) -> HandlerResult {
    let user_id = state.user_id;
    log::debug!("links.view | state: {:#?}", state);
    let links =  match state.child.get(&state.level) {
        Some(links) => links.to_owned(),
        None => db::links(user_id).await?,
    };
    log::debug!("links.view | links: {:#?}", links);
    state.child = links.child.clone();
    let state = state.to_owned();
    dialogue.update(state.clone()).await?;
    view(bot, msg, state, links).await?;
    Ok(())
}
///
/// 
pub async fn view(bot: Bot, msg: Message, state: LinksState, links: Links) -> HandlerResult {
    let user_id = state.user_id;
    let markup = markup(&links, user_id).await?;
    let text = links.title.unwrap_or(format!("Useful links"));
    bot.edit_message_text(msg.chat.id, msg.id, text)
        // .edit_message_media(user_id, message_id, media)
        .reply_markup(markup)
        .await
        .map_err(|err| format!("inline::view {}", err))?;
    Ok(())
}
///
/// 
async fn markup(links: &Links, user_id: UserId) -> Result<InlineKeyboardMarkup, String> {
    let mut buttons: Vec<InlineKeyboardButton> = links.links
        .iter()
        .map(|link| {
            InlineKeyboardButton::url(
                link.title.clone(),
                reqwest::Url::parse(&link.url).unwrap(),
        )})
        .collect();
    for (id, child) in &links.child {
        if let Some(title) = &child.title {
            buttons.push(
                InlineKeyboardButton::callback(
                    title,
                    format!("/{}", id),
                )
            );
        }
    }
    let button_back = InlineKeyboardButton::callback(
        loc("⏪Back"), // "⏪Back"
        format!("/back")
    );
    buttons.push(button_back);
    let markup = buttons.into_iter()
    .fold(InlineKeyboardMarkup::default(), |acc, item| acc.append_row(vec![item]));
    Ok(markup)
}