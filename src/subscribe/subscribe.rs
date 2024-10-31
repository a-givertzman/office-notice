use indexmap::IndexMap;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, User, UserId}};
use crate::{
    db, kernel::error::HandlerResult, loc::{loc, LocaleTag}, states::{MainState, MyDialogue},
};

use super::subscription::Subscriptions;
///
/// Subscribe menu
#[derive(Debug, Clone, PartialEq)]
pub enum SubscribeMenu {
   Group(String),   // Selected group to subscribe on
   Unknown(String), // Unknown command received
   Done,            // Exit menu
}
//
//
impl SubscribeMenu {
    pub fn parse(s: &str, _loc_tag: LocaleTag) -> Self {
        let input = s.strip_prefix('/').map_or_else(|| ("", s), |input| ("/", input));
        match input {
            ("/", "done" | "Done") => Self::Done,
            ("/", "back" | "Back") => Self::Done,
            ("/", "exit" | "Exit") => Self::Done,
            ("/", input) => Self::Group(input.to_owned()),
            (_, _) => Self::Unknown(s.to_owned()),
        }
}
}
///
/// 
#[derive(Debug, Clone)]
pub struct SubscribeState {
    pub prev_state: MainState,  // Where to go on Back btn
    pub group: String,          // Group id to be noticed
    pub chat_id: ChatId,        // User id doing notice
    pub user: User,
}
//
//
impl Default for SubscribeState {
    fn default() -> Self {
        Self { prev_state: MainState::default(), group: String::new(), chat_id: ChatId(0), user: User { id: UserId(0), is_bot: false, first_name: String::new(), last_name: None, username: None, language_code: None, is_premium: false, added_to_attachment_menu: false } }
    }
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: SubscribeState) -> HandlerResult {
    let user_id = state.chat_id;
    let user_name = state.user.username.clone().unwrap_or(state.user.full_name());
    log::debug!("subscribe.enter | state: {:#?}", state);
    let mut subscriptions =  match db::subscriptions().await {
        Ok(groups) => groups,
        Err(err) => {
            log::warn!("subscribe.enter | Groups is empty, error: {:#?}", err);
            IndexMap::new()
        }
    };
    if !state.group.is_empty() {
        // let group_title = groups.get(&state.group).map_or(state.group.clone(), |group| group.title.clone());
        subscribe(&mut subscriptions, &state.group, user_id, &user_name).await?;
        log::debug!("subscribe.enter | Subscription '{}' ({}) for group '{}' - updated", user_name, user_id, state.group);
    }
    let text = format!("Select group to subscribe / unsubscribe");
    dialogue.update(state.clone()).await?;
    view(&bot, &msg, &state, &subscriptions, text).await?;
    Ok(())
}
///
/// 
pub async fn subscribe(subscriptions: &mut Subscriptions, group: &str, user_id: ChatId, user_name: &str) -> HandlerResult {
    if let Some(group) = subscriptions.get_mut(group) {
        let user_id_str = &user_id.to_string();
        match group.members.get(user_id_str) {
            Some(_) => {
                log::debug!("subscribe.subscribe | Removing subscription '{}' ({}) from group '{}'", user_name, user_id, group.title);
                if let None = group.members.shift_remove(user_id_str) {
                    log::debug!("subscribe.subscribe | Error removing subscription '{}' ({}) from group '{}' - key not found", user_name, user_id, group.title);
                }
            }
            None => {
                log::debug!("subscribe.subscribe | Adding subscription '{}' ({}) to the group '{}' ", user_name, user_id, group.title);
                let user = crate::db::user(&user_id.into()).await?;
                if let Some(origin) = group.members.insert(user_id_str.to_owned(), user) {
                    log::warn!("subscribe.subscribe | Error adding subscription '{}' ({}) to the group '{}' - olready exists", user_name, user_id, group.title);
                    group.members.insert(user_id_str.to_owned(), origin);
                }
            }
        }
        db::update_subscriptions(subscriptions).await?;
    } else {
        log::warn!("subscribe.subscribe | Group '{}' not found in the subscriptions: {:#?}", group, subscriptions);
    }
    Ok(())
}
///
/// 
pub async fn view(bot: &Bot, msg: &Message, state: &SubscribeState, groups: &Subscriptions, text: impl Into<String>) -> HandlerResult {
    let user_id = state.chat_id;
    let markup = markup(&groups, user_id).await?;
    crate::message::edit_markup_message_or_send(bot, msg, &markup, &text.into()).await
    // bot.edit_message_text(msg.chat.id, msg.id, text)
    //     // .edit_message_media(user_id, message_id, media)
    //     .reply_markup(markup)
    //     .await
    //     .map_err(|err| format!("inline::view {}", err))?;
    // Ok(())
}
///
/// 
async fn markup(groups: &Subscriptions, user_id: ChatId) -> Result<InlineKeyboardMarkup, String> {
    let _ = user_id;
    let mut buttons: Vec<InlineKeyboardButton> = groups
        .iter()
        .map(|(group_id, group)| {
            InlineKeyboardButton::callback(
                if group.members.contains_key(&user_id.to_string()) {
                    format!("✅ {}", group.title)
                } else {
                    format!("{}", group.title)
                },
                format!("/{}", group_id),
        )})
        .collect();
    let button_back = InlineKeyboardButton::callback(
        loc("⏪Back"), // "⏪Back"
        format!("/back")
    );
    buttons.push(button_back);
    let markup = buttons.into_iter()
        .fold(InlineKeyboardMarkup::default(), |acc, item| acc.append_row(vec![item]));
    Ok(markup)
}