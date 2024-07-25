//! Office menu bot.
//! Callback from inline button
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::{menu, states::*};
use crate::loc::*;

#[derive(BotCommands )]
pub enum Command {
   Start, // make the specified node active
   Unknown,
}
///
/// 
pub async fn update(bot: Bot, q: CallbackQuery) -> HandlerResult {
   let query_id = q.id.to_owned();
   // Parse and process commands by receiving a message to send back
   let input = q.data.to_owned().unwrap_or_default();
   log::debug!("callback.update | Input: {}", input);
   let bot_me = bot.get_me().await?;
   let cmd = Command::parse(&input.to_lowercase(), bot_me.username())?;
   let msg = match cmd {
      Command::Start => {
         menu::view(&bot, q).await?;
         loc("Start")
      }
      Command::Unknown => format!("callback.update | Unknowm command {}", input),
      _ => format!("callback.update | Command {} - not implemented", input)
   };
   // Sending a response that is shown in a pop-up window
   bot.answer_callback_query(query_id)
   .text(msg)
   .await
   .map_err(|err| format!("inline::update {}", err))?;
   Ok(())
}