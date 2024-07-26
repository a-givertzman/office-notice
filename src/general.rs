use strum::AsRefStr;
use teloxide::{prelude::*, types::ParseMode, };
use crate::states::*;
use crate::loc::*;
///
///
#[derive(Debug, Clone)]
pub struct MessageState {
   pub prev_state: MainState,
   pub receiver: ChatId,
}
///
///
#[derive(AsRefStr)]
pub enum Command {
   #[strum(to_string = "/start")]
   Start,
//    #[strum(to_string = "/start ")]
//    StartFrom,
//    #[strum(to_string = "/msg")]
   Message(String),
//    #[strum(to_string = "/get")]
//    Goto(i32),
   Unknown(String),
}
//
//
impl Command {
    fn parse(s: &str) -> Self {
        if s == Self::Start.as_ref() { Command::Start }
        else {
            if s.starts_with('/') {
                Command::Unknown(s.to_owned())
            } else {
                Command::Message(s.to_owned())
            }
        }
    }
}
// ///
// /// 
// async fn enter_input(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState, receiver: ChatId) -> HandlerResult {
//     let chat_id = msg.chat.id;
//     // "Enter a message to send (/ to cancel)"
//     let text = loc("Enter a message to send (/ to cancel)");
//     bot.send_message(chat_id, text)
//         .reply_markup(cancel_markup(0))
//         .await?;
//     let new_state = MessageState {
//        prev_state: state,
//        receiver,
//     };
//     dialogue.update(new_state).await?;
//     Ok(())
// } 
///
/// 
pub async fn update(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    let _ = dialogue;
    // Parse and handle commands
    let chat_id = msg.chat.id;
    let input = msg.text().unwrap_or_default();
    let user_id = state.user_id;
    let user_name = format!("{} {}", msg.chat.first_name().unwrap_or(""), msg.chat.first_name().unwrap_or(""));
    log::debug!("general.update | user: {} ({}), input {} ", user_name, user_id, input);
    let cmd = Command::parse(input);
    match cmd {
        //
        Command::Start => {
            let text = loc("Welcome. Please click on 'All' to display the full list, 'Open' for those currently working (if the panel with buttons is hidden, expand it), or send a text to search.");
            bot.send_message(chat_id, text)
            .reply_markup(main_menu_markup(0))
            .await?;
        }
        //
        Command::Message(text) => {
            // return enter_input(bot, msg, dialogue, state, receiver).await
            log::warn!("general.update | Unhandled message: '{}'", text);
            return Ok(());
        }
        //
        // Command::StartFrom => return crate::menu::enter(bot, msg, state).await,
        //
        Command::Unknown(command) => {
            log::warn!("general.update | Unknown commnd: '{}'", command);
            let text = format!("Unknown commnd: {}", command);
            bot.send_message(chat_id, text)
                .reply_markup(main_menu_markup(0))
                .parse_mode(ParseMode::Html)
                .await?;
        },
    }
    Ok(())
}
///
///
pub async fn update_input(bot: Bot, msg: Message, dialogue: MyDialogue, state: MessageState) -> HandlerResult {
    let chat_id = msg.chat.id;
    let input = msg.text().unwrap_or_default();
    let info = if input == loc("/") {
       // "Cancel, message not sent"
       loc("Cancel, message not sent")
    } else {
        // Forward message to receiver
        let msg = bot.forward_message(state.receiver, msg.chat.id, msg.id).await?;
        // Add info with qoute. "Reply {}{}"
        let text = loc(format!("Reply {}{}", Command::Message(input.to_owned()).as_ref(), state.prev_state.user_id));
        bot.send_message(state.receiver, &text)
            .reply_to_message_id(msg.id).await?;
        // "Message sent"
        loc("Message sent")
    };

    // Report result and return to main menu
    bot.send_message(chat_id, info)
    .reply_markup(main_menu_markup(0))
    .await?;

    // Return to previous state
    dialogue.update(state.prev_state).await?;
    Ok(())
}