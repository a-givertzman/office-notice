use teloxide::{prelude::*, types::ParseMode
 };
use crate::states::{HandlerResult, MainState, MyDialogue};
 ///
 /// 
 pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    let chat_id = msg.chat.id;
    dialogue.update(state.prev_state.to_owned()).await?;
    bot.send_message(chat_id, HELP_TEXT_RU)
        // .caption(text)
        .parse_mode(ParseMode::Html)
        // .disable_notification(true)
        .await?;
    Ok(())
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
- Бот поможет разсылать сообщения (пока текстовые) по группам пользователей.
- Группа может быть создана вручную (пока используя assets/subscription.json, в будущем через меню Бота)
или Бота можно добавить в существующую телеграм-группу.
- Пользователи могут получать сообщения будучи участниками телеграм группы или могут подписаться на группу бота.
В последнем случае пользователь будет получать прямые сообщения от Бота не состоя в телеграм-группе.
- Для отправки сообщения используйте /start -> Notice
- Для подписки / отписки на оповещения используйте /start -> Subscribe
";