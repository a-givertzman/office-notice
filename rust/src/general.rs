use strum::AsRefStr;
use teloxide::{prelude::*, types::ParseMode, };
use crate::states::*;
use crate::loc::*;
///
///
#[derive(Clone)]
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
   #[strum(to_string = "/start ")]
   StartFrom(i32),
   #[strum(to_string = "/msg")]
   Message(ChatId),
//    #[strum(to_string = "/get")]
//    Goto(i32),
   Unknown,
}
//
//
impl Command {
    fn parse(s: &str) -> Self {
        if s == Self::Start.as_ref() { Command::Start }
        else {
            // Looking for the commands with arguments
            let l_part = s.get(..4).unwrap_or_default();
            let r_part = s.get(4..).unwrap_or_default();
    
            if l_part == Self::Message(ChatId(0)).as_ref() {
                let id = r_part.parse().unwrap_or_default();
                Command::Message(ChatId(id))
            // } else if l_part == Self::Goto(0).as_ref() {
            //     Command::Goto(r_part.parse().unwrap_or_default())
            } else {
                // More long command
                let l_part = s.get(..7).unwrap_or_default();
                if l_part == Self::StartFrom(0).as_ref() {
                    let r_part = s.get(7..).unwrap_or_default();
                    Command::StartFrom(r_part.parse().unwrap_or_default())
                } else {
                    Command::Unknown
                }
            }
        }
    }
}
///
/// 
async fn enter_input(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState, receiver: ChatId) -> HandlerResult {
    let chat_id = msg.chat.id;
 
    // "Enter a message to send (/ to cancel)"
    let text = loc("Enter a message to send (/ to cancel)");
 
    bot.send_message(chat_id, text)
    .reply_markup(cancel_markup(0))
    .await?;
 
    let new_state = MessageState {
       prev_state: state,
       receiver,
    };
    dialogue.update(new_state).await?;
    Ok(())
} 
///
/// 
pub async fn update(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    // Parse and handle commands
    let chat_id = msg.chat.id;
    let input = msg.text().unwrap_or_default();
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
        Command::Message(receiver) => return enter_input(bot, msg, dialogue, state, receiver).await,
       
        Command::StartFrom(node_id) => return crate::menu::enter(bot, msg, state).await,
       
        Command::Unknown => {
            let text = loc("Text message please");
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
        let text = loc(format!("Reply {}{}", Command::Message(ChatId(0)).as_ref(), state.prev_state.user_id));
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