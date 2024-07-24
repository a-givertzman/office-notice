use teloxide::{prelude::*,
    types::{ReplyMarkup, KeyboardButton, KeyboardMarkup, User, UserId,},
    dispatching::{dialogue::{self, InMemStorage}, UpdateHandler, UpdateFilterExt, },
};
use crate::states::{HandlerResult, MainState, MyDialogue};
///
/// 
#[derive(Clone)]
pub struct LinksState {
   pub prev_state: MainState,
//    pub customer: Customer,
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {

    // Load user info
    // let customer = db::user(state.user_id.0).await?;
 
    // Display
    let state = LinksState {
        prev_state: state,
        // customer
    };
    let s = state.to_owned();
    dialogue.update(s).await?;
    Ok(())
    // view(bot, msg, state).await
}