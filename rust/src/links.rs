use serde::{Deserialize, Serialize};
use teloxide::{dispatching::{dialogue::{self, InMemStorage}, UpdateFilterExt, UpdateHandler }, prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, ReplyMarkup, User, UserId}
};
use crate::{db, loc::loc, states::{HandlerResult, MainState, MyDialogue, StartState}};
///
/// 
#[derive(Serialize, Deserialize)]
pub struct Link {
    pub title: String,
    pub url: String,
}
///
/// 
#[derive(Serialize, Deserialize)]
pub struct Links {
    pub links: Vec<Link>,
    pub child: Vec<Links>,
}
///
/// 
#[derive(Clone)]
pub struct LinksState {
    pub prev_state: MainState,
    pub user_id: UserId,
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {

    // Load user info
    let user = db::user(state.user_id.0).await?;
 
    // Display
    let state = LinksState {
        prev_state: state,
        user_id: state.user_id,
    // customer
    };
    let state = state.to_owned();
    dialogue.update(state.clone()).await?;
    view(bot, msg, state).await?;
    Ok(())
}
///
/// 
pub async fn view(bot: Bot, msg: Message, state: LinksState) -> HandlerResult {
    let user_id = state.user_id;
    // Load from storage
    let links =  db::links(user_id).await?;
    // // Collect info
    let markup = markup(&links, user_id).await?;
    let text = format!("Useful links");
    // let chat_id = ChatId::Id(message.chat_id());
    bot.send_message(msg.chat.id, text)
        .reply_markup(markup)
        .await?;    
    // bot.edit_message_text(user_id, message_id, text)
    //     // .edit_message_media(user_id, message_id, media)
    //     .reply_markup(markup)
    //     .await
    //     .map_err(|err| format!("inline::view {}", err))?;
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
    let button_back = InlineKeyboardButton::callback(
        loc("⏪Back"), // "⏪Back"
        format!("Back")
    );
    buttons.push(button_back);
    let markup = buttons.into_iter()
    .fold(InlineKeyboardMarkup::default(), |acc, item| acc.append_row(vec![item]));
    Ok(markup)
}