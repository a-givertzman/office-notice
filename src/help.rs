use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode}
 };
use crate::{loc::loc, states::{HandlerResult, MainState, MyDialogue}};
 ///
 /// 
 pub async fn enter(bot: &Bot, msg: &Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    dialogue.update(state).await?;
    view(bot, msg, HELP_TEXT_RU).await
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
    let markup = markup().await?;
    bot.edit_message_text(msg.chat.id, msg.id, text)
        // .edit_message_media(user_id, message_id, media)
        .reply_markup(markup)
        .parse_mode(ParseMode::Html)
        .await
        .map_err(|err| format!("inline::view {}", err))?;
    Ok(())
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