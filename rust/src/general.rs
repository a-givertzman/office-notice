use strum::AsRefStr;
use teloxide::{prelude::*, types::ParseMode, };
use crate::states::*;
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
   #[strum(to_string = "/get")]
   Goto(i32),
   Unknown,
}
///
/// 
pub async fn update(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    // Parse and handle commands
    let tag = state.tag;
    let chat_id = msg.chat.id;
    let input = msg.text().unwrap_or_default();
    let cmd = Command::parse(input);
    match cmd {
        //
       Command::Start => {
          // "Welcome. Please click on 'All' to display the full list, 'Open' for those currently working (if the panel with buttons is hidden, expand it), or send a text to search."
          let text = loc(Key::GeneralUpdate1, tag, &[]);
          bot.send_message(chat_id, text)
          .reply_markup(main_menu_markup(tag))
          .await?;
       }
       //
       Command::Message(receiver) => return enter_input(bot, msg, dialogue, state, receiver).await,
       
       Command::Goto(node_id)
       | Command::StartFrom(node_id) => return crate::navigation::enter(bot, msg, state, WorkTime::AllFrom(node_id)).await,
       
       Command::Unknown => {
          let text = if input.is_empty() {
             // Text message please
             loc(Key::GeneralUpdate2, tag, &[])
          } else {
             let found = search::search(input).await?;
             if found.is_empty() {
                // "Search for <b>'{}'</b> returned no results"
                loc(Key::GeneralUpdate3, tag, &[&input])
             } else {
                // Add hint if too many founds
                let hint = if found.len() > 30 {
                   // "<i>Only the first 30 results are shown, please try to refine your query</i>"
                   loc(Key::GeneralUpdate4, tag, &[])
                } else {
                   String::default()
                };
 
                // "Search results for '{}'.{}\n"
                let init =  loc(Key::GeneralUpdate5, tag, &[&input, &hint]);
 
                found.iter()
                .take(30)
                .fold(init, |acc, v| {
                   format!("{}\n{}", acc, v)
                })
             }
          };
 
          bot.send_message(chat_id, text)
          .reply_markup(main_menu_markup(tag))
          .parse_mode(ParseMode::Html)
          .await?;
       },
    }
    Ok(())
}
///
///
pub async fn update_input(bot: Bot, msg: Message, dialogue: MyDialogue, state: MessageState) -> HandlerResult {

    let tag = state.prev_state.tag;
    let chat_id = msg.chat.id;
    let input = msg.text().unwrap_or_default();

    let info = if input == loc(Key::CommonCancel, tag, &[]) {
       // "Cancel, message not sent"
       loc(Key::GeneralUpdateInput1, tag, &[])
    } else {
       // Forward message to receiver
       let msg = bot.forward_message(state.receiver, msg.chat.id, msg.id).await?;

       // Add info with qoute. "Reply {}{}"
       let args: Args = &[&Command::Message(ChatId(0)).as_ref(),
          &state.prev_state.user_id
       ];
       let text = loc(Key::GeneralUpdateInput2, tag, args);
       bot.send_message(state.receiver, &text)
       .reply_to_message_id(msg.id)
       .await?;

       // "Message sent"
       loc(Key::GeneralUpdateInput3, tag, &[])
    };

    // Report result and return to main menu
    bot.send_message(chat_id, info)
    .reply_markup(main_menu_markup(tag))
    .await?;

    // Return to previous state
    dialogue.update(state.prev_state).await?;
    Ok(())
}