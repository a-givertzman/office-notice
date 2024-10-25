use indexmap::IndexMap;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, UserId}};
use crate::{db, loc::{loc, LocaleTag}, message::send_message_with_header, states::{HandlerResult, MainState, MyDialogue}, subscription::Subscriptions};
///
/// Notice menu
#[derive(Debug, Clone, PartialEq)]
pub enum NoticeMenu {
   Group(String),   // Selected group to be noticed
   Unknown(String), // Unknown command received
   Done,            // Exit menu
}
//
//
impl NoticeMenu {
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
        view(&bot, &msg, &state, &groups, text, Some(())).await?;
    } else {
        let text = format!("Select group to notice");
        dialogue.update(state.clone()).await?;
        view(&bot, &msg, &state, &groups, text, None).await?;
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
    let user = db::user(&state.user_id.into()).await?;
    match msg.text() {
        Some(text) => {
            log::debug!("notice.notice | Sending notice from '{}' ({}): '{:?}'", user.name, state.user_id, text);
            if let Some(group) = groups.get(&state.group) {
                log::debug!("notice.notice | Sending notice to the '{}' group...", group.title);
                if let Some(group_id) = &group.id {
                    if let Err(err) = send_message_with_header(&bot, group_id.to_owned(), &user.name, text).await {
                        log::warn!("notice.notice | Error sending message to the '{}' ({}): {:#?}", group.title, group_id, err);
                    };
                }
                for (_, receiver) in &group.members {
                    log::debug!("notice.notice | \t member '{}' ({})", receiver.name, receiver.id);
                    if let Err(err) = send_message_with_header(&bot, receiver.id, &user.name, text).await {
                        log::warn!("notice.notice | Error sending message to the '{}' ({}): {:#?}", receiver.name, receiver.id, err);
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
    let state = NoticeState { prev_state: state.prev_state, user_id: state.user_id,  ..Default::default() };
    dialogue.update(state.clone()).await?;
    crate::notice::enter(bot.to_owned(), msg.to_owned(), dialogue, state).await?;
    Ok(())
}
///
/// Menu buttons to select a notice group
pub async fn view(bot: &Bot, msg: &Message, state: &NoticeState, groups: &Subscriptions, text: impl Into<String>, is_message: Option<()>) -> HandlerResult {
    let _user_id = state.user_id;
    let markup = markup(&groups, is_message).await?;
    crate::message::edit_message_text_or_send(bot, msg, &markup, &text.into()).await?;
    Ok(())
}
///
/// 
async fn markup(groups: &Subscriptions, is_message: Option<()>) -> Result<InlineKeyboardMarkup, String> {
    let mut buttons: Vec<InlineKeyboardButton> = match is_message {
        Some(_) => vec![],
        None => groups
            .iter()
            .map(|(group_id, group)| {
                InlineKeyboardButton::callback(
                    group.title.clone(),
                    format!("/{}", group_id),
            )})
            .collect(),
    };
    let button_back = InlineKeyboardButton::callback(
        loc("⏪Back"), // "⏪Back"
        format!("/back")
    );
    buttons.push(button_back);
    let markup = buttons.into_iter()
    .fold(InlineKeyboardMarkup::default(), |acc, item| acc.append_row(vec![item]));
    Ok(markup)
}
