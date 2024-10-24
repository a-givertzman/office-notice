use std::time::Duration;

use derive_more::From;
use indexmap::IndexMap;
use teloxide::{prelude::*,
   types::{User, UserId,},
   dispatching::{dialogue::{self, InMemStorage}, UpdateHandler, UpdateFilterExt, },
};
use tokio::time::sleep;
use crate::{db, help::HelpState, links::{LinksMenu, LinksState}, menu::{self, MainMenu}, notice::{self, NoticeMenu, NoticeState}, subscribe::{SubscribeMenu, SubscribeState}};
// use crate::database as db;
// use crate::gear::*;
// use crate::cart::*;
// use crate::general::MessageState;
use crate::loc::*;
pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

// FSM states
#[derive(Debug, Clone, From)]
pub enum State {
   Start(StartState),   // initial state
   Main(MainState),     // main menu state
   Links(LinksState),   // in Links menu
   NoticeMenu(NoticeState),         // in Notice menu
   Subscribe(SubscribeState),    // in Subscribe menu
   Help(HelpState),
//    GeneralMessage(MessageState), // general commands, enter text of message to send
}
//
//
impl Default for State {
   fn default() -> Self {
      Self::Start(StartState { restarted: true })
   }
}
///
/// 
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StartState {
   pub restarted: bool,
}
//
//
impl Default for StartState {
    fn default() -> Self {
        Self { restarted: false }
    }
}
///
/// 
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MainState {
   pub prev_state: StartState,
   pub user_id: UserId,
}
//
//
impl Default for MainState {
    fn default() -> Self {
        Self { prev_state: StartState::default(), user_id: UserId(0) }
    }
}
///
/// 
pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let message_handler = Update::filter_message()
        .branch(
            // Private message handler
            dptree::filter(|msg: Message| { msg.chat.is_private() })
            .branch(dptree::case![State::Start(state)].endpoint(start))
            .branch(dptree::case![State::Main(state)].endpoint(command))
            .branch(dptree::case![State::Links(state)].endpoint(command))
            .branch(dptree::case![State::NoticeMenu(state)].endpoint(notice::notice))
            .branch(dptree::case![State::Subscribe(state)].endpoint(command))
            // .branch(dptree::case![State::GeneralMessage(state)].endpoint(crate::general::update_input))
        )
        .branch(dptree::entry().endpoint(chat_message_handler));
    let callback_query_handler = Update::filter_callback_query()
        .endpoint(callback);
    // let chat_member_update_handler = Update::filter_my_chat_member()// filter_chat_member()
    //     .branch(dptree::filter(|m: ChatMemberUpdated| {
    //         m.new_chat_member.is_member() //m.old_chat_member.is_left() && 
    //     })
    //     .endpoint(new_chat_member))
    //     .branch(dptree::filter(|m: ChatMemberUpdated| {
    //         m.new_chat_member.is_left() // m.old_chat_member.is_member() && 
    //     })
    //     .endpoint(left_chat_member),
    // );
    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
        // .branch(chat_member_update_handler)
}
///
/// Callback on bot was added to chat
pub async fn new_chat_member(chat_member: &ChatMemberUpdated) -> HandlerResult {
    let user = chat_member.old_chat_member.user.clone();
    let chat_id = chat_member.chat.id;
    let chat_id_string = chat_id.to_string();
    let chat_name = chat_member.chat.username().unwrap_or(&chat_id_string);
    let chat_title = chat_member.chat.title().unwrap_or(chat_name);
    // We get a "@username" mention via `mention()` method if the user has a
    // username, otherwise we create a textual mention with "Full Name" as the
    // text linking to the user
    let username = user.mention().unwrap_or_else(|| format!("{} ({})", user.full_name(), user.id));
    log::debug!("states.new_chat_member | MyChatMember(added): user {}, chat: {}", username, chat_name);
    // bot.send_message(chat_member.chat.id, format!("Welcome to {telegram_group_name} {username}!")).await?;
    db::insert_subscription(&chat_id_string, chat_title).await?;
    Ok(())
}
///
/// Callback on bot was removed from chat
pub async fn left_chat_member(chat_member: &ChatMemberUpdated) -> HandlerResult {
    let chat_name = format!("{} ({})", chat_member.chat.username().unwrap_or("-"), chat_member.chat.id);
    let user = &chat_member.old_chat_member.user;
    let username = user.mention().unwrap_or_else(|| format!("{} ({})", user.full_name(), user.id));
    log::debug!("states.left_chat_member | MyChatMember(removed):user {}, chat: {}", username, chat_name);
    // bot.send_message(chat_member.chat.id, format!("Goodbye {username}!")).await?;
    db::remove_subscription(chat_member.chat.id).await?;
    Ok(())
}
///
/// Command | Start
async fn start(bot: Bot, msg: Message, dialogue: MyDialogue, state: StartState) -> HandlerResult {
    // Extract user id
    let user = msg.from.clone();
    if user.is_none() {
        let chat_id = msg.chat.id;
        bot.send_message(chat_id, "Error, no user").await?;
        dialogue.update(StartState { restarted: false }).await?;
        log::debug!("states.start | no user, chat_id: {}", chat_id);
        return Ok(());
    }
    let user = user.unwrap();
    let user_id = user.id;
    // let new_state = MainState { prev_state: state, user_id };
    // Insert or update info about user
    update_last_seen_full(&user).await?;
    log::debug!("states.start | user {} ({})", user.full_name(), user_id);
    let cmd_raw = msg.text().unwrap_or_default();
    match cmd_raw {
        "/start" | "/Start" => crate::states::enter(&bot, &msg, dialogue, MainState { prev_state: state, user_id }).await,
        _ => {
            let text =  loc(format!("Please type '/Start' to begin"));
            bot.send_message(msg.chat.id, text)
                .await?;
            Ok(())
        }
    }
    // command(bot, msg, dialogue, new_state).await
}
///
/// 
pub async fn enter(bot: &Bot, msg: &Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    dialogue.update(state).await?;
    // let text =  loc("You are in the main menu");
    // let chat_id = msg.chat.id;
    menu::enter(bot, msg).await?;
    // bot.send_message(chat_id, text)
    //     .reply_markup(main_menu_markup(0))
    //     .await?;
    Ok(())
}

///
/// 
pub async fn reload(bot: Bot, msg: &Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    dialogue.update(state).await?;
    // let text =  loc("You are in the main menu");
    // let chat_id = msg.chat.id;
    menu::reload(&bot, msg).await?;
    // bot.send_message(chat_id, text)
    //     .reply_markup(main_menu_markup(0))
    //     .await?;
    Ok(())
}
pub async fn exit(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    dialogue.update(state.prev_state).await?;
    // let text =  loc("You are in the main menu");
    // let chat_id = msg.chat.id;
    menu::exit(&bot, &msg).await?;
    // bot.send_message(chat_id, text)
    //     .reply_markup(main_menu_markup(0))
    //     .await?;
    Ok(())
}
///
///
// #[async_recursion]
pub async fn command(bot: Bot, msg: Message, dialogue: MyDialogue, state: State) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user_name = format!("{} {}", msg.chat.first_name().unwrap_or(""), msg.chat.first_name().unwrap_or(""));
    let cmd_raw = msg.text().unwrap_or_default();
    log::debug!("states.command | input '{}', from: {} ({:?})", cmd_raw, user_name, msg.from);
    let cmd = MainMenu::parse(cmd_raw, 0);
    // Try to execute command and if it impossible notify about restart
    match state {
        State::Start(start_state) => {
            // For admin and regular users there is different interface
            log::debug!("states.command | State: {:?}", start_state);
            match msg.from.clone() {
                Some(from) => {
                    let user_id = from.id;
                    log::debug!("states.command | Input {}, user: {} ({})", cmd_raw, user_name, user_id);
                    let new_state = MainState {
                        prev_state: StartState { restarted: false },
                        user_id,
                    };
                    // Update FSM
                    dialogue.update(new_state.to_owned()).await?;
                    crate::states::enter(&bot, &msg, dialogue, new_state).await?;
                }
                None => log::debug!("states.command | Cmd '{}' ignired because from: {} ({:?}), expected from regular user", cmd_raw, user_name, msg.from),
            }
        }
        State::Main(main_state) => {
            let user_id = main_state.user_id;
            log::debug!("states.command | State: {:?}", main_state);
            match cmd {
                MainMenu::Links(level) =>   crate::links::enter(bot, msg, dialogue, LinksState {prev_state: main_state, level, child: IndexMap::new(), user_id}).await?,
                MainMenu::Notice =>                 crate::notice::enter(bot, msg, dialogue, NoticeState { prev_state: main_state, user_id, ..Default::default()}).await?,
                MainMenu::Subscribe =>              crate::subscribe::enter(bot, msg, dialogue, SubscribeState { prev_state: main_state, user_id, ..Default::default() }).await?,
                MainMenu::Help =>                   crate::help::enter(bot, msg, dialogue, HelpState { prev_state: main_state }).await?,
                MainMenu::Done =>                   crate::states::exit(bot, msg, dialogue, main_state).await?,
                MainMenu::Unknown => {
                    log::debug!("states.command | user: {} ({}), Unknown command {}", user_name, user_id, cmd_raw);
                    // Report about a possible restart and loss of context
                    if main_state.prev_state.restarted {
                        let text =  loc(format!("Unknown command '{}'", cmd_raw)); // Sorry, the bot has been restarted
                        // let text =  loc("Извините, бот был перезапущен"); // Sorry, the bot has been restarted
                        bot.send_message(chat_id, text)
                            // .reply_markup(main_menu_markup(0))
                            .await?;
                    } else {
                        // Process general commands without search if restarted (to prevent search submode commands)
                        let text =  loc(format!("Unknown command '{}'", cmd_raw)); // Sorry, the bot has been restarted
                        bot.send_message(chat_id, text)
                        // .reply_markup(main_menu_markup(0))
                        .await?;
                        // crate::states::reload(bot, msg, dialogue, state).await?;
                    }
                    sleep(Duration::from_secs(2)).await;
                    dialogue.update(main_state.prev_state).await?;
                    crate::states::reload(bot.clone(), &msg, dialogue, main_state).await?
                    // match crate::states::reload(bot.clone(), &msg, dialogue.clone(), state.prev_state).await {
                    //     Ok(_) => todo!(),
                    //     Err(_) => crate::states::exit(bot, &msg, dialogue, state).await?,
                    // }
                }
            };
        }
        State::Links(links_state) => {
            // let user_id = links_state.user_id;
            log::debug!("states.command | State: {:?}", links_state);
            dialogue.update(links_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, links_state.prev_state).await?
        }
        State::NoticeMenu(notice_state) => {
            // let user_id = notice_state.user_id;
            log::debug!("states.command | State: {:?}", notice_state);
            dialogue.update(notice_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, notice_state.prev_state).await?
        }
        State::Subscribe(subscribe_state) => {
            // let user_id = subscribe_state.user_id;
            log::debug!("states.command | State: {:?}", subscribe_state);
            dialogue.update(subscribe_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, subscribe_state.prev_state).await?
        }
        State::Help(help_state) => {
            log::debug!("states.command | State: {:?}", help_state);
            dialogue.update(help_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, help_state.prev_state).await?
        }
    }
    Ok(())
}
///
/// 
pub async fn chat_message_handler(bot: Bot, msg: Message) -> HandlerResult {
    // For chat messages react only command for printout group id (need for identify service chat)
    let (user_name, user_id) = match &msg.from {
        Some(from) => (from.full_name(), from.id.to_string()),
        None => ("-".to_owned(), "-".to_owned()),
    };
    log::debug!("states.chat_message_handler | user: {} ({}), message {:?}", user_name, user_id, msg.text());
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
///
/// 
pub async fn callback(bot: Bot, q: CallbackQuery, dialogue: MyDialogue, state: State) -> HandlerResult {
    let user_id = q.from.id;
    let user_name = q.from.full_name();
    // Determine the language of the user
    let input = q.data.to_owned().unwrap_or_default();
    log::debug!("states.callback | State: {:?}, User {} ({}) Input: {}", state, user_name, user_id, input);
    match state {
        State::Start(_) => {}
        State::Main(state) => {
            let input = q.data.to_owned().unwrap_or_default();
            log::debug!("states.callback | Main > Input: {}", input);
            let cmd = MainMenu::parse(&input, 0);
            log::debug!("states.callback | Main > Cmd: {:?}", cmd);
            match cmd {
                MainMenu::Links(level) => {
                    let state = LinksState {prev_state: state, level, child: IndexMap::new(), user_id };
                    crate::links::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Notice => {
                    let state = NoticeState { prev_state: state, user_id, ..Default::default() };
                    crate::notice::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Subscribe => {
                    let state = SubscribeState { prev_state: state, user_id, ..Default::default() };
                    crate::subscribe::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Help => {
                    let state = HelpState { prev_state: state };
                    crate::help::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Done => crate::states::exit(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?,
                MainMenu::Unknown => {
                    log::debug!("states.command | Main > user: {} ({}), Unknown command {}", user_name, user_id, input);
                }
            }
        }
        State::Links(state) => {
            // log::debug!("states.callback | Links > state: {:#?}", state);
            let input = q.data.to_owned().unwrap_or_default();
            log::debug!("states.callback | Links > Input: {}", input);
            let cmd = LinksMenu::parse(&input, 0);
            log::debug!("states.callback | Links > Cmd: {:?}", cmd);
            match cmd {
                LinksMenu::Link(level) => {
                    let state = LinksState {
                        prev_state: state.prev_state,
                        // prev_level: Some(state.level),
                        level,
                        child: state.child,
                        user_id: state.user_id,
                    };
                    crate::links::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                LinksMenu::Done => crate::states::reload(bot.clone(), q.regular_message().unwrap(), dialogue, state.prev_state).await?,
            }
        }
        State::NoticeMenu(state) => {
            log::debug!("states.callback | Notice > state: {:#?}", state);
            let input = q.data.to_owned().unwrap_or_default();
            log::debug!("states.callback | Notice > Input: {}", input);
            let cmd = NoticeMenu::parse(&input, 0);
            log::debug!("states.callback | Notice > Cmd: {:?}", cmd);
            match cmd {
                NoticeMenu::Group(group) => {
                    log::debug!("states.callback | Notice > Notice Will send to the: '{}' group", group);
                    let state = NoticeState {
                        prev_state: state.prev_state,
                        group,
                        user_id: state.user_id,
                    };
                    crate::notice::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                NoticeMenu::Unknown(text) => {
                    log::debug!("states.callback | Notice > Unknown command received: '{}'", text);
                    crate::notice::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                NoticeMenu::Done => crate::states::reload(bot.clone(), q.regular_message().unwrap(), dialogue, state.prev_state).await?,
            }
        }
        State::Subscribe(state) => {
            log::debug!("states.callback | Notice > state: {:#?}", state);
            let input = q.data.to_owned().unwrap_or_default();
            log::debug!("states.callback | Notice > Input: {}", input);
            let cmd = SubscribeMenu::parse(&input, 0);
            log::debug!("states.callback | Notice > Cmd: {:?}", cmd);
            match cmd {
                SubscribeMenu::Group(group) => {
                    log::debug!("states.callback | Notice > Notice Will send to the: '{}' group", group);
                    let state = SubscribeState {
                        prev_state: state.prev_state,
                        group,
                        user_id: state.user_id,
                        user: q.from.clone()
                    };
                    crate::subscribe::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                SubscribeMenu::Unknown(text) => {
                    log::debug!("states.callback | Notice > Unknown command received: '{}'", text);
                    crate::subscribe::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                SubscribeMenu::Done => crate::states::reload(bot.clone(), q.regular_message().unwrap(), dialogue, state.prev_state).await?,
            }            
        }
        State::Help(state) => {
            log::debug!("states.callback | Help > state: {:#?}", state);
            crate::states::reload(bot.clone(), q.regular_message().unwrap(), dialogue, state.prev_state).await?
        }
        // State::GeneralMessage(state) => {
        //     log::debug!("states.callback | GeneralMessage > receiver: {}", state.receiver);
        // },
    }
    Ok(())
}
// ///
// /// 
// pub fn main_menu_markup(_loc_tag: LocaleTag) -> ReplyMarkup {
//     let commands = vec![
//         loc("🛒"),
//         loc("Все"),
//         loc("Открыто"),
//         loc("⚙"),
//     ];
//     kb_markup(vec![commands])
// }
///
///
async fn update_last_seen_full(user: &User) -> Result<(), String> {
    log::debug!("states.update_last_seen_full | user: {} ({})", user.full_name(), user.id);
    let user_id = user.id.0;
    // Collect info about the new user and store in database
    let name = if let Some(last_name) = &user.last_name {
        format!("{} {}", user.first_name, last_name)
    } else {user.first_name.clone()};
    let contact = if let Some(username) = &user.username {
        format!("{}", username)
    } else {String::from("-")};
    db::user_insert(user_id, name, Some(contact), None).await?;
    Ok(())
}
// ///
// /// Frequently used menu
// pub fn cancel_markup(_loc_tag: LocaleTag) -> ReplyMarkup {
//     kb_markup(vec![vec![loc("/")]])
// }
// ///
// /// Construct keyboard from strings
// pub fn kb_markup(keyboard: Vec<Vec<String>>) -> ReplyMarkup {
//     let kb: Vec<Vec<KeyboardButton>> = keyboard.iter()
//         .map(|row| {
//             row.iter()
//             .map(|label| KeyboardButton::new(label))
//             .collect()
//         })
//         .collect();
//     let markup = KeyboardMarkup::new(kb)
//         .resize_keyboard();
//     ReplyMarkup::Keyboard(markup)
// }