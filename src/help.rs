use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup}};
use crate::{loc::loc, states::{HandlerResult, MainState, MyDialogue}};
///
/// 
#[derive(Debug, Clone)]
pub struct HelpState {
    pub prev_state: MainState,  // Where to go on Back btn
    // pub user_id: UserId,        // User id doing notice
}
///
/// 
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: HelpState) -> HandlerResult {
    dialogue.update(state.clone()).await?;
    view(&bot, &msg, HELP_TEXT_RU).await
    // bot.send_message(chat_id, HELP_TEXT_RU)
    //     // .caption(text)
    //     .parse_mode(ParseMode::Html)
    //     // .disable_notification(true)
    //     .await?;
    // Ok(())
}
///
/// 
pub async fn view(bot: &Bot, msg: &Message, text: impl Into<String>) -> HandlerResult {
    let text = text.into();
    let markup = markup().await?;
    crate::message::edit_message_text_or_send(bot, msg, &markup, &text).await
}
///
/// 
async fn markup() -> Result<InlineKeyboardMarkup, String> {
    let mut buttons: Vec<InlineKeyboardButton> = vec![];
    let button_back = InlineKeyboardButton::callback(
        loc("⏪Back"), // "⏪Back"
        format!("/back")
    );
    buttons.push(button_back);
    let markup = buttons.into_iter()
    .fold(InlineKeyboardMarkup::default(), |acc, item| acc.append_row(vec![item]));
    Ok(markup)
}
///
/// 
// const HELP_TEXT_EN: &str = "
// - The Bot can help you to notify a Group of users with the text (for now).
// - The Group can be created manualy (using subscription.json for now, using menu in the future)
// or bot can be added to the existing telegram group.
// - Users can be subscribed on both types of the groups: manually created and existing
// - If the user subscribed on the existing telegram group, hi will receaves the Bot's notices only, but not the Group conversations messages.
// - Use menu notice to send message to the Group
// - Use menu subscribe to select a Groups to be subscribed on
// ";
const HELP_TEXT_RU: &str = "
- Бот поможет разсылать сообщения (пока текстовые) по группам пользователей;
- Группа может быть создана двумя способами:
    - Вручную (пока используя assets/subscription.json, в будущем через меню Бота);
    - Или Бота можно добавить в существующую телеграм-группу;
- Пользователи могут получать сообщения будучи участниками телеграм группы или могут подписаться на группу бота;
В последнем случае пользователь будет получать прямые сообщения от Бота не состоя в телеграм-группе.
- Для отправки сообщения используйте /start -> Notice
- Для подписки / отписки на оповещения используйте /start -> Subscribe
";