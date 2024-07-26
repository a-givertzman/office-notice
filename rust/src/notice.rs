use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, UserId}};
use crate::{db, loc::loc, states::{HandlerResult, MainState, MyDialogue, StartState}, subscription::{Subscription, Subscriptions}};
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
pub struct NoticeState {
    pub prev_state: MainState,  // Where to go on Back btn
    pub group: String,          // Group id to be noticed
    pub text: String,           // Notice text to be sent
    pub user_id: UserId,        // User id doing notice
}
//
//
impl Default for NoticeState {
    fn default() -> Self {
        Self { prev_state: MainState::default(), group: String::new(), text: String::new(), user_id: UserId(0) }
    }
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: NoticeState) -> HandlerResult {
    let user_id = state.user_id;
    log::debug!("notice.enter | state: {:#?}", state);
    let groups =  match db::subscriptions(user_id).await {
        Ok(groups) => groups,
        Err(err) => {
            log::warn!("notice.enter | Groups is empty, error: {:#?}", err);
            IndexMap::new()
        }
    };
    log::debug!("notice.enter | groups: {:#?}", groups);
    // let state = state.to_owned();
    if !state.group.is_empty() {
        if let Some(group) = groups.get(&state.group) {
            notice(&bot, &msg, &state, &group).await?
        }
    }
    dialogue.update(state.clone()).await?;
    view(bot, msg, state, groups).await?;
    Ok(())
}
///
/// 
pub async fn notice(bot: &Bot, msg: &Message, state: &NoticeState, group: &Subscription) -> HandlerResult {
    log::warn!("notice.notice | Sending notice to the '{}' group...", group.title);
    for (_, user) in &group.members {
        bot.send_message(user.id, &state.text)
            // .edit_message_media(user_id, message_id, media)
            .await
            .map_err(|err| format!("inline::view {}", err))?;
    }
    Ok(())
}
///
/// 
pub async fn view(bot: Bot, msg: Message, state: NoticeState, groups: Subscriptions) -> HandlerResult {
    let user_id = state.user_id;
    let markup = markup(&groups, user_id).await?;
    let text = format!("Select group to notice");
    bot.edit_message_text(msg.chat.id, msg.id, text)
        // .edit_message_media(user_id, message_id, media)
        .reply_markup(markup)
        .await
        .map_err(|err| format!("inline::view {}", err))?;
    Ok(())
}
///
/// 
async fn markup(groups: &Subscriptions, user_id: UserId) -> Result<InlineKeyboardMarkup, String> {
    let mut buttons: Vec<InlineKeyboardButton> = groups
        .iter()
        .map(|(group_id, group)| {
            InlineKeyboardButton::callback(
                group.title.clone(),
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