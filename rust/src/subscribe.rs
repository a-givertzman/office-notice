use indexmap::IndexMap;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, UserId}};
use crate::{db, loc::loc, states::{HandlerResult, MainState, MyDialogue}, subscription::Subscriptions, user::User};
///
/// 
#[derive(Debug, Clone)]
pub struct SubscribeState {
    pub prev_state: MainState,  // Where to go on Back btn
    pub group: String,          // Group id to be noticed
    pub user_id: UserId,        // User id doing notice
}
//
//
impl Default for SubscribeState {
    fn default() -> Self {
        Self { prev_state: MainState::default(), group: String::new(), user_id: UserId(0) }
    }
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: SubscribeState) -> HandlerResult {
    let user_id = state.user_id;
    let user_id_str = &user_id.to_string();
    let user_name = msg.from().map_or(user_id_str.to_owned(), |user| user.username.clone().unwrap_or(user_id_str.to_owned()));
    log::debug!("subscribe.enter | state: {:#?}", state);
    let mut groups =  match db::subscriptions().await {
        Ok(groups) => groups,
        Err(err) => {
            log::warn!("subscribe.enter | Groups is empty, error: {:#?}", err);
            IndexMap::new()
        }
    };
    if !state.group.is_empty() {
        // let group_title = groups.get(&state.group).map_or(state.group.clone(), |group| group.title.clone());
        subscribe(&mut groups, &state.group, user_id, &user_name).await?;
    }
    let text = format!("Select group to subscribe / unsubscribe");
    dialogue.update(state.clone()).await?;
    view(&bot, &msg, &state, &groups, text).await?;
    Ok(())
}
///
/// 
pub async fn subscribe(subscriptions: &mut Subscriptions, group: &str, user_id: UserId, user_name: &str) -> HandlerResult {
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
                let user = User {
                    id: ChatId::from(user_id),
                    name: user_name.to_owned(),
                    contact: None,
                    address: None,
                    subscriptions: vec![],
                };
                if let Some(origin) = group.members.insert(user_id_str.to_owned(), user) {
                    log::warn!("subscribe.subscribe | Error adding subscription '{}' ({}) to the group '{}' - olready exists", user_name, user_id, group.title);
                    group.members.insert(user_id_str.to_owned(), origin);
                }
            }
        }
    } else {
        log::warn!("subscribe.subscribe | Group '{}' not found in the subscriptions: {:#?}", group, subscriptions);
    }
    // let state = state.prev_state;
    // dialogue.update(state.clone()).await?;
    // crate::states::reload(bot, msg, dialogue, state).await?;
    Ok(())
}
///
/// 
pub async fn view(bot: &Bot, msg: &Message, state: &SubscribeState, groups: &Subscriptions, text: impl Into<String>) -> HandlerResult {
    let user_id = state.user_id;
    let markup = markup(&groups, user_id).await?;
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
    let _ = user_id;
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