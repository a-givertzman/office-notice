
use derive_more::From;
use teloxide::{prelude::*,
   types::{ReplyMarkup, KeyboardButton, KeyboardMarkup, User, UserId,},
   dispatching::{dialogue::{self, InMemStorage}, UpdateHandler, UpdateFilterExt, },
};

use crate::{db, environment as env, links::LinksState, notice::NoticeState, subscribe::SubscribeState};
// use crate::database as db;
// use crate::gear::*;
// use crate::cart::*;
use crate::general::MessageState;
use crate::loc::*;

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

// FSM states
#[derive(Clone, From)]
pub enum State {
    Start(StartState), // initial state
    Command(MainState), // await for select menu item from bottom
    Links(LinksState), // in Links menu
    Notice(NoticeState), // in Notice menu
    Subscribe(SubscribeState), // in Subscribe menu
    GeneralMessage(MessageState), // general commands, enter text of message to send
}

impl Default for State {
   fn default() -> Self {
      Self::Start(StartState { restarted: true })
   }
}

#[derive(Copy, Clone, PartialEq)]
pub struct StartState {
   pub restarted: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub struct MainState {
   pub prev_state: StartState,
   pub user_id: UserId,
   pub is_admin: bool,
   pub tag: LocaleTag,
}


// Main menu
enum MainMenu {
    Links,      // Links menu
    Notice,     // Notice menu
    Subscribe,  // subscribe to receive notice
    Done,       // Exit menu
    Unknown,
}

impl MainMenu {
   fn parse(s: &str, _loc_tag: LocaleTag) -> Self {
    match s.to_lowercase().as_str() {
        "notice" => Self::Notice,
        "links" => Self::Links,
        "subscribe" => Self::Subscribe,
        "done" => Self::Done,
        _ => Self::Unknown,
    }
   }
}

pub enum WorkTime {
   All,  // show all nodes
   Now,  // considering work time
   AllFrom(i32), // like all but from the specified node id
}


pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {

    let message_handler = Update::filter_message()
    .branch(
       // Private message handler
       dptree::filter(|msg: Message| { msg.chat.is_private() })
       .branch(dptree::case![State::Start(state)].endpoint(start))
       .branch(dptree::case![State::Command(state)].endpoint(command))
       .branch(dptree::case![State::GeneralMessage(state)].endpoint(crate::general::update_input))
    )
    .branch(dptree::entry().endpoint(chat_message_handler));

    let callback_query_handler = Update::filter_callback_query().endpoint(callback);

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
    .branch(message_handler)
    .branch(callback_query_handler)
}
///
/// Command | Start
async fn start(bot: Bot, msg: Message, dialogue: MyDialogue, state: StartState) -> HandlerResult {
    // Extract user id
    let user = msg.from();
    if user.is_none() {
       let chat_id = msg.chat.id;
       bot.send_message(chat_id, "Error, no user").await?;
       dialogue.update(StartState { restarted: false }).await?;
       return Ok(());
    }
    let user = user.unwrap();
    let user_id = user.id;
    let new_state = MainState { prev_state: state, user_id, is_admin: false , tag: 0};
    // Insert or update info about user
    update_last_seen_full(user).await?;

    command(bot, msg, dialogue, new_state)
    .await
}
///
/// 
async fn exit(bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, "Ok, by").await?;
    Ok(())
}
///
///
async fn receive_main_menu_choise(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    Ok(())
}

pub async fn reload(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    dialogue.update(state).await?;
    let text =  loc("You are in the main menu");
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, text)
    .reply_markup(main_menu_markup(0))
    .await?;
    Ok(())
}

// #[async_recursion]
pub async fn command(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    let chat_id = msg.chat.id;

    // For admin and regular users there is different interface
    let user_id = state.user_id;
    let new_state = MainState {
       prev_state: StartState { restarted: false },
       user_id,
       is_admin: env::is_admin_id(user_id), // reload permissions every time
       tag: 0,
    };

    // Update FSM
    if state != new_state {
       dialogue.update(new_state.to_owned()).await?;
    }

    // Try to execute command and if it impossible notify about restart
    let text = msg.text().unwrap_or_default();
    let cmd = MainMenu::parse(text, 0);
    match cmd {
       MainMenu::Links => crate::links::enter(bot, msg, dialogue, new_state).await?,
       MainMenu::Subscribe => crate::subscribe::enter(bot, msg, dialogue, new_state).await?,
       MainMenu::Notice => crate::notice::enter(bot, msg, dialogue, new_state).await?,
       MainMenu::Done => exit(bot, msg).await?,

       MainMenu::Unknown => {

          // Report about a possible restart and loss of context
          if state.prev_state.restarted {
             let text =  loc("Извините, бот был перезапущен"); // Sorry, the bot has been restarted
             bot.send_message(chat_id, text)
             .reply_markup(main_menu_markup(0))
             .await?;
          } else {

             // Process general commands without search if restarted (to prevent search submode commands)
             crate::general::update(bot, msg, dialogue, new_state).await?;
          }
       }
    };
    Ok(())
}

pub async fn chat_message_handler(bot: Bot, msg: Message) -> HandlerResult {

    // For chat messages react only command for printout group id (need for identify service chat)
    if let Some(input) = msg.text() {
       match input.get(..5).unwrap_or_default() {
          "/chat" => {
             let chat_id = msg.chat.id;
             let text = format!("Chat id={}", chat_id);
             bot.send_message(chat_id, text).await?;
          }
          _ => (),
       }
    }

    Ok(())
}

pub async fn callback(bot: Bot, q: CallbackQuery) -> HandlerResult {
    let user_id = q.from.id;

    // Determine the language of the user
    let locale = q.from.language_code.as_deref();
    let tag = loc_tag(locale);

    let res = crate::callback::update(bot.to_owned(), q.to_owned(), tag).await;

    // Notify user about possible error
    if let Err(e) = res {
       // Sending a response that is shown in a pop-up window
       let text = loc("Error, start again"); // "Error, start over"
       bot.answer_callback_query(q.id)
       .text(&text)
       .await
       .map_err(|err| format!("inline::update {} {}", text, err))?;

       // Send full text of error
       bot.send_message(q.from.id, format!("{}\n{}", text, e)).await?;

       // For default handler
       return Err(e);
    }
    Ok(())
}
///
/// 
pub fn main_menu_markup(_loc_tag: LocaleTag) -> ReplyMarkup {
    let commands = vec![
       loc("🛒"),
       loc("Все"),
       loc("Открыто"),
       loc("⚙"),
    ];
    kb_markup(vec![commands])
}


async fn update_last_seen_full(user: &User) -> Result<(), String> {
    let user_id = user.id.0;

    // Collect info about the new user and store in database
    let name = if let Some(last_name) = &user.last_name {
       format!("{} {}", user.first_name, last_name)
    } else {user.first_name.clone()};

    let contact = if let Some(username) = &user.username {
       format!(" @{}", username)
    } else {String::from("-")};

    db::user_insert(user_id, name, contact).await?;
    Ok(())
}


// // Convert for flag value
// pub fn to_flag(text: &String, tag: LocaleTag) -> Result<bool, String> {
//     if text == &loc(Key::StatesOn, tag, &[]) { // On
//        Ok(true)
//     } else if text == &loc(Key::StatesOff, tag, &[]) { // Off
//        Ok(false)
//     } else {
//        Err(loc(Key::StatesWrongSwitch, tag, &[text])) // Expected On or Off, got {}
//     }
// }

// pub fn from_flag(flag: bool, tag: LocaleTag) -> String {
//     if flag { loc(Key::StatesOn, tag, &[]) } // On
//     else { loc(Key::StatesOff, tag, &[]) } // Off
// }

// Frequently used menu
pub fn cancel_markup(_loc_tag: LocaleTag) -> ReplyMarkup {
    kb_markup(vec![vec![loc("/")]])
}

// pub fn flag_markup(_loc_tag: LocaleTag) -> ReplyMarkup {
//     kb_markup(vec![vec![from_flag(true, _loc_tag), from_flag(false, _loc_tag)]])
// }

// Construct keyboard from strings
pub fn kb_markup(keyboard: Vec<Vec<String>>) -> ReplyMarkup {
    let kb: Vec<Vec<KeyboardButton>> = keyboard.iter()
       .map(|row| {
          row.iter()
          .map(|label| KeyboardButton::new(label))
          .collect()
       })
       .collect();

    let markup = KeyboardMarkup::new(kb)
       .resize_keyboard(true);

    ReplyMarkup::Keyboard(markup)
}