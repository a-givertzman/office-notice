use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, UserId}};
use crate::{db, loc::loc, states::{HandlerResult, MainState, MyDialogue}, subscription::Subscriptions};
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
    pub prev_state: MainState,
    pub user_id: UserId,
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, mut state: NoticeState) -> HandlerResult {
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
    let state = state.to_owned();
    dialogue.update(state.clone()).await?;
    view(bot, msg, state, groups).await?;
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