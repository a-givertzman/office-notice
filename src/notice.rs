use indexmap::IndexMap;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, UserId}};
use crate::{db, loc::loc, states::{HandlerResult, MainState, MyDialogue}, subscription::Subscriptions};
///
/// 
#[derive(Debug, Clone)]
pub struct NoticeState {
    pub prev_state: MainState,  // Where to go on Back btn
    pub group: String,          // Group id to be noticed
    pub user_id: UserId,        // User id doing notice
}
//
//
impl Default for NoticeState {
    fn default() -> Self {
        Self { prev_state: MainState::default(), group: String::new(), user_id: UserId(0) }
    }
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: NoticeState) -> HandlerResult {
    log::debug!("notice.enter | state: {:#?}", state);
    let groups =  match db::subscriptions().await {
        Ok(groups) => groups,
        Err(err) => {
            log::warn!("notice.enter | Groups is empty, error: {:#?}", err);
            IndexMap::new()
        }
    };
    if !state.group.is_empty() {
        let group_title = groups.get(&state.group).map_or(state.group.clone(), |group| group.title.clone());
        let text = format!("Type a text for group '{}'", group_title);
        dialogue.update(state.clone()).await?;
        bot.edit_message_text(msg.chat.id, msg.id, text)
            // .edit_message_media(user_id, message_id, media)
            .await
            .map_err(|err| format!("inline::view {}", err))?;
        // view(&bot, &msg, &state, &groups, text).await?;
    } else {
        let text = format!("Select group to notice");
        dialogue.update(state.clone()).await?;
        view(&bot, &msg, &state, &groups, text).await?;
    }
    Ok(())
}
///
/// 
pub async fn notice(bot: Bot, msg: Message, dialogue: MyDialogue, state: NoticeState) -> HandlerResult {
    let groups =  match db::subscriptions().await {
        Ok(groups) => groups,
        Err(err) => {
            log::warn!("notice.notice | Groups is empty, error: {:#?}", err);
            IndexMap::new()
        }
    };
    match msg.text() {
        Some(text) => {
            let user_name = msg.from().map_or("-".to_owned(), |user| user.username.clone().unwrap_or("-".to_owned()));
            log::debug!("notice.notice | Sending notice from '{}' ({}): '{:?}'", user_name, state.user_id, text);
            if let Some(group) = groups.get(&state.group) {
                log::debug!("notice.notice | Sending notice to the '{}' group...", group.title);
                if let Some(group_id) = &group.id {
                    if let Err(err) = bot.send_message(group_id.to_owned(), text).await {
                        log::debug!("notice.notice | Error sending message to the '{}' ({}): {:#?}", group.title, group_id, err);
                    };
                }
                for (_, user) in &group.members {
                    log::debug!("notice.notice | \t member '{}' ({})", user.name, user.id);
                    if let Err(err) = bot.send_message(user.id, text).await {
                        log::debug!("notice.notice | Error sending message to the '{}' ({}): {:#?}", user.name, user.id, err);
                    };
                }
            } else {
                log::warn!("notice.notice | Group '{}' not found in the subscriptions: {:#?}", state.group, groups);
            }
        }
        None => {
            bot.send_message(state.user_id, "Notice text can't be empty")
                .await
                .map_err(|err| format!("inline::view {}", err))?;
        }
    }
    let state = state.prev_state;
    dialogue.update(state.clone()).await?;
    crate::states::reload(bot, msg, dialogue, state).await?;
    Ok(())
}
///
/// 
pub async fn view(bot: &Bot, msg: &Message, state: &NoticeState, groups: &Subscriptions, text: impl Into<String>) -> HandlerResult {
    let _user_id = state.user_id;
    let markup = markup(&groups).await?;
    bot.edit_message_text(msg.chat.id, msg.id, text)
        // .edit_message_media(user_id, message_id, media)
        .reply_markup(markup)
        .await
        .map_err(|err| format!("inline::view {}", err))?;
    Ok(())
}
///
/// 
async fn markup(groups: &Subscriptions) -> Result<InlineKeyboardMarkup, String> {
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