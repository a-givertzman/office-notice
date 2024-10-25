use teloxide::{payloads::{EditMessageTextSetters, SendMessageSetters}, prelude::Requester, types::{InlineKeyboardMarkup, Message, ParseMode, Recipient}, Bot};
use crate::states::HandlerResult;
///
/// Edit current message if possible or sending new one
pub async fn edit_message_text_or_send(bot: &Bot, msg: &Message, markup: &InlineKeyboardMarkup, text: &str) -> HandlerResult {
    let result = bot.edit_message_text(msg.chat.id, msg.id, text)
        // .edit_message_media(user_id, message_id, media)
        .reply_markup(markup.to_owned())
        .parse_mode(ParseMode::Html)
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(err) => match &err {
            teloxide::RequestError::Api(api_error) => match api_error {
                teloxide::ApiError::MessageCantBeEdited => {
                    bot.send_message(msg.chat.id, text)
                        // .edit_message_media(user_id, message_id, media)
                        .reply_markup(markup.to_owned())
                        .parse_mode(ParseMode::Html)
                        .await?;
                    Ok(())
                }
                _ => Err(Box::new(err)),
            },
            _ => Err(Box::new(err)),
        }
    }
}
///
/// Sends message with header
pub async fn send_message_with_header(bot: &Bot, chat_id: impl Into<Recipient>, header: &str, text: &str) -> HandlerResult {
    bot
        .send_message(chat_id, format!("<b>{}:</b>\n{}", header, text))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
