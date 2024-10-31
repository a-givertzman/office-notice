use std::time::Duration;
use chrono::Utc;
use derive_more::From;
use indexmap::IndexMap;
use teloxide::{dispatching::{dialogue::{self, InMemStorage}, UpdateFilterExt, UpdateHandler }, prelude::*, types::{ParseMode, User}};
use tokio::time::sleep;
use crate::{
    db, help::HelpState, kernel::error::HandlerResult, links::{LinksMenu, LinksState}, menu::{self, MainMenu}, message::{edit_markup_message_or_send, edit_text_message_or_send, send_message_with_header}, notice::{self, NoticeMenu, NoticeState}, subscribe::{SubscribeMenu, SubscribeState}, user::{
        grant_access::{GrantAccessMenu, GrantAccessState}, request_access::{RequestAccessMenu, RequestAccessState}, user_role::UserRole
    }, BOT_NAME
};
use crate::loc::*;
pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
///
/// FSM states
#[derive(Debug, Clone, From)]
pub enum State {
   Start(StartState),   // initial state
   Main(MainState),     // main menu state
   Links(LinksState),   // in Links menu
   Notice(NoticeState),     // in Notice menu
   Subscribe(SubscribeState),   // in Subscribe menu
   RequestAccess(RequestAccessState),    // in RequestAccess menu
   GrantAccess(GrantAccessState),    // in RequestAccess menu
   Help(HelpState),                     // In the Halp menu
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
   pub chat_id: ChatId,
}
//
//
impl Default for MainState {
    fn default() -> Self {
        Self { prev_state: StartState::default(), chat_id: ChatId(0) }
    }
}
//
//
impl From<State> for MainState {
    fn from(value: State) -> Self {
        match value {
            State::Start(state) =>  MainState { prev_state: state, chat_id: ChatId(0) },
            State::Main(state) => state,
            State::Links(state) => MainState { prev_state: StartState::default(), chat_id: state.chat_id },
            State::Notice(state) => MainState { prev_state: StartState::default(), chat_id: state.chat_id },
            State::Subscribe(state) => MainState { prev_state: StartState::default(), chat_id: state.chat_id },
            State::RequestAccess(state) => MainState { prev_state: StartState::default(), chat_id: state.user.id },
            State::GrantAccess(state) => MainState { prev_state: StartState::default(), chat_id: state.user.id },
            State::Help(state) => MainState { prev_state: StartState::default(), chat_id: state.user.id },
            // _ => MainState { prev_state: (), chat_id: value }
            // panic!("MainState.try_from | Illegal input state '{:?}'", value)
        }
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
            .branch(dptree::case![State::Notice(state)].endpoint(notice::notice))
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
    // Insert or update info about user
    update_last_seen_full(&user).await?;
    log::debug!("states.start | user {} ({})", user.full_name(), user_id);
    let cmd_raw = msg.text().unwrap_or_default();
    match cmd_raw {
        "/start" | "/Start" => crate::states::enter(&bot, &msg, dialogue, MainState { prev_state: state, chat_id: msg.chat.id }).await,
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
    let user = db::user(&msg.chat.id).await?;
    menu::enter(bot, msg, &user).await?;
    Ok(())
}

///
/// 
pub async fn reload(bot: Bot, msg: &Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    dialogue.update(state).await?;
    let user = db::user(&msg.chat.id).await?;
    menu::reload(&bot, msg, &user).await?;
    Ok(())
}
pub async fn exit(bot: Bot, msg: Message, dialogue: MyDialogue, state: MainState) -> HandlerResult {
    dialogue.update(state.prev_state).await?;
    let user = db::user(&msg.chat.id).await?;
    menu::exit(&bot, &msg, &user).await?;
    Ok(())
}
///
/// Handles command from users
// #[async_recursion]
pub async fn command(bot: Bot, msg: Message, dialogue: MyDialogue, state: State) -> HandlerResult {
    let dbgid = "states";
    let chat_id = msg.chat.id;
    let user = db::user(&msg.chat.id).await?;
    let user_name = format!("{} {}", msg.chat.first_name().unwrap_or(""), msg.chat.first_name().unwrap_or(""));
    let cmd_raw = msg.text().unwrap_or_default();
    log::debug!("{}.command | Input '{}', from: {} ({:?})", dbgid, cmd_raw, user.name, msg.from);
    let cmd = MainMenu::parse(cmd_raw, 0);
    // Try to execute command and if it impossible notify about restart
    match state {
        State::Start(start_state) => {
            // For admin and regular users there is different interface
            log::debug!("{}.command | State: {:?}", dbgid, start_state);
            log::debug!("{}.command | Input {}, user: {} ({})", dbgid, cmd_raw, user_name, chat_id);
            let new_state = MainState {
                prev_state: StartState { restarted: false },
                chat_id,
            };
            // Update FSM
            dialogue.update(new_state.to_owned()).await?;
            crate::states::enter(&bot, &msg, dialogue, new_state).await?;
        }
        State::Main(main_state) => {
            let user_id = main_state.chat_id;
            log::debug!("{}.command | State: {:?}", dbgid, main_state);
            match cmd {
                MainMenu::RequestAccess => {
                    crate::user::request_access::enter(bot, msg, dialogue, RequestAccessState {prev_state: main_state, user: user}).await?;
                }
                MainMenu::Links(level) => {
                    if user.has_role(&[UserRole::Admin, UserRole::Moder, UserRole::Sender, UserRole::Member]) {
                        crate::links::enter(bot, msg, dialogue, LinksState {prev_state: main_state, level, child: IndexMap::new(), chat_id: user_id}).await?
                    } else {
                        send_message_with_header(
                            &bot, chat_id, BOT_NAME,
                            &loc(format!("{}, you can't access shared resources according to your roles: \n{:?}", user.name, user.role))
                        ).await?;
                    }
                }
                MainMenu::Notice => {
                    if user.has_role(&[UserRole::Admin, UserRole::Moder, UserRole::Sender]) {
                        crate::notice::enter(bot, msg, dialogue, NoticeState { prev_state: main_state, chat_id: user_id, ..Default::default()}).await?
                    } else {
                        send_message_with_header(
                            &bot, chat_id, BOT_NAME,
                            &loc(format!("{}, you can't send notice's according to your roles: \n{:?}", user.name, user.role)),
                        ).await?;
                    }
                }
                MainMenu::Subscribe => {
                    if user.has_role(&[UserRole::Admin, UserRole::Moder, UserRole::Sender, UserRole::Member]) {
                        crate::subscribe::enter(bot, msg, dialogue, SubscribeState { prev_state: main_state, chat_id: user_id, ..Default::default() }).await?
                    } else {
                        send_message_with_header(
                            &bot, chat_id, BOT_NAME,
                            &loc(format!("{}, you can't subscribe for notice's according to your roles: \n{:?}", user.name, user.role)),
                        ).await?;
                    }
                }
                MainMenu::Help => crate::help::enter(bot, msg, dialogue, HelpState { prev_state: main_state, user }).await?,
                MainMenu::Done => crate::states::exit(bot, msg, dialogue, main_state).await?,
                MainMenu::Unknown => {
                    log::debug!("{}.command | user: {} ({}), Unknown command {}", dbgid, user_name, user_id, cmd_raw);
                    // Report about a possible restart and loss of context
                    if main_state.prev_state.restarted {
                        let text =  loc(format!("Unknown command '{}'", cmd_raw));
                        bot.send_message(chat_id, text)
                            .await?;
                    } else {
                        // Process general commands without search if restarted (to prevent search submode commands)
                        let text =  loc(format!("Unknown command '{}'", cmd_raw));
                        bot.send_message(chat_id, text)
                        .await?;
                    }
                    sleep(Duration::from_secs(2)).await;
                    dialogue.update(main_state.prev_state).await?;
                    crate::states::reload(bot.clone(), &msg, dialogue, main_state).await?
                }
            };
        }
        State::Links(links_state) => {
            // let user_id = links_state.user_id;
            log::debug!("{}.command | State: {:?}", dbgid, links_state);
            dialogue.update(links_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, links_state.prev_state).await?
        }
        State::Notice(notice_state) => {
            // let user_id = notice_state.user_id;
            log::debug!("{}.command | State: {:?}", dbgid, notice_state);
            dialogue.update(notice_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, notice_state.prev_state).await?
        }
        State::Subscribe(subscribe_state) => {
            // let user_id = subscribe_state.user_id;
            log::debug!("{}.command | State: {:?}", dbgid, subscribe_state);
            dialogue.update(subscribe_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, subscribe_state.prev_state).await?
        }
        State::RequestAccess(ra_state) => {
            // let user_id = subscribe_state.user_id;
            log::debug!("{}.command | State: {:?}", dbgid, ra_state);
            dialogue.update(ra_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, ra_state.prev_state).await?
        }
        State::GrantAccess(ga_state) => {
            // let user_id = subscribe_state.user_id;
            log::debug!("{}.command | State: {:?}", dbgid, ga_state);
            dialogue.update(ga_state.prev_state).await?;
            crate::states::reload(bot.clone(), &msg, dialogue, ga_state.prev_state).await?
        }
        State::Help(help_state) => {
            log::debug!("{}.command | State: {:?}", dbgid, help_state);
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
/// Handles command callbacks
pub async fn callback(bot: Bot, q: CallbackQuery, dialogue: MyDialogue, state: State) -> HandlerResult {
    let dbgid = "states";
    let chat_id = ChatId::from(q.from.id);
    let user = db::user(&chat_id).await?;
    let user_name = q.from.full_name();
    // Determine the language of the user
    let input = q.data.to_owned().unwrap_or_default();
    log::debug!("{}.callback | State: {:?}, User {} ({}) Input: {}", dbgid, state, user_name, chat_id, input);
    match GrantAccessMenu::parse(&input, 0) {
        GrantAccessMenu::Role(role) => {
            match &state {
                State::GrantAccess(ga_state) => {
                    log::debug!("{}.callback | Granting role '{:?}' to user {}", dbgid, role, ga_state.user.name);
                    let state = GrantAccessState { prev_state: ga_state.prev_state, user: ga_state.user.clone(), role: Some(role) };
                    crate::user::grant_access::enter(bot.clone(), q.regular_message().unwrap().to_owned(), dialogue.clone(), state).await?
                }
                _ => {
                    log::debug!("{}.callback | Grant role Invalid state '{:?}'", dbgid, state);
                }
            }
        }
        GrantAccessMenu::Done => {
            let granted_user = match &state {
                State::GrantAccess(ga_state) => {
                    dialogue.update(ga_state.prev_state).await?;
                    ga_state.user.name.clone()
                },
                _ => "-".to_owned(),
            };
            let text = format!("Canceled role granting for user '{}'", granted_user);
            edit_text_message_or_send(&bot, q.regular_message().unwrap(), &text).await?;
        }
        GrantAccessMenu::Unknown(_) => {}
    }
    match state {
        State::Start(state) => {
            log::debug!("{}.callback | State::Start > state: {:#?}", dbgid, state);
            log::debug!("{}.callback | State::Start > Not implemented, return", dbgid);
        }
        State::RequestAccess(state) => {
            log::debug!("{}.callback | State::RequestAccess > state: {:#?}", dbgid, state);
            log::debug!("{}.callback | State::RequestAccess > Not implemented, return", dbgid);
        }
        State::GrantAccess(state) => {
            log::debug!("{}.callback | State::GrantAccess > state: {:#?}", dbgid, state);
            log::debug!("{}.callback | Granting role '{:?}' to user {}", dbgid, state.role, state.user.name);
            match GrantAccessMenu::parse(&input, 0) {
                GrantAccessMenu::Role(role) => {
                    let state = GrantAccessState { prev_state: state.prev_state, user: state.user.clone(), role: Some(role) };
                    crate::user::grant_access::enter(bot.clone(), q.regular_message().unwrap().to_owned(), dialogue.clone(), state).await?
                }
                GrantAccessMenu::Done => {
                    let text = format!("Canceled role granting for user '{}'", state.user.name);
                    dialogue.update(state.prev_state).await?;
                    edit_text_message_or_send(&bot, q.regular_message().unwrap(), &text).await?;
                }
                GrantAccessMenu::Unknown(_) => {}
            }
        }
        State::Main(state) => {
            let input = q.data.to_owned().unwrap_or_default();
            log::debug!("{}.callback | State::Main > Input: {}", dbgid, input);
            let cmd = MainMenu::parse(&input, 0);
            log::debug!("{}.callback | State::Main > Cmd: {:?}", dbgid, cmd);
            match cmd {
                MainMenu::RequestAccess => {
                    let state = RequestAccessState { prev_state: state, user };
                    crate::user::request_access::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Links(level) => {
                    let state = LinksState {prev_state: state, level, child: IndexMap::new(), chat_id };
                    crate::links::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Notice => {
                    let state = NoticeState { prev_state: state, chat_id, ..Default::default() };
                    crate::notice::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Subscribe => {
                    let state = SubscribeState { prev_state: state, chat_id, ..Default::default() };
                    crate::subscribe::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Help => {
                    let state = HelpState { prev_state: state, user };
                    crate::help::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                MainMenu::Done => crate::states::exit(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?,
                MainMenu::Unknown => {
                    log::debug!("{}.callback | State::Main > user: {} ({}), Unknown command {}", dbgid, user_name, chat_id, input);
                }
            }
        }
        State::Links(state) => {
            // log::debug!("{}.callback | State::Links > state: {:#?}", state);
            let input = q.data.to_owned().unwrap_or_default();
            log::debug!("{}.callback | State::Links > Input: {}", dbgid, input);
            let cmd = LinksMenu::parse(&input, 0);
            log::debug!("{}.callback | State::Links > Cmd: {:?}", dbgid, cmd);
            match cmd {
                LinksMenu::Link(level) => {
                    let state = LinksState {
                        prev_state: state.prev_state,
                        // prev_level: Some(state.level),
                        level,
                        child: state.child,
                        chat_id: state.chat_id,
                    };
                    crate::links::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                LinksMenu::Done => crate::states::reload(bot.clone(), q.regular_message().unwrap(), dialogue, state.prev_state).await?,
            }
        }
        State::Notice(state) => {
            log::debug!("{}.callback | State::Notice > state: {:#?}", dbgid, state);
            let input = q.data.to_owned().unwrap_or_default();
            log::debug!("{}.callback | State::Notice > Input: {}", dbgid, input);
            let cmd = NoticeMenu::parse(&input, 0);
            log::debug!("{}.callback | State::Notice > Cmd: {:?}", dbgid, cmd);
            match cmd {
                NoticeMenu::Group(group) => {
                    log::debug!("{}.callback | State::Notice > Notice Will send to the: '{}' group", dbgid, group);
                    let state = NoticeState {
                        prev_state: state.prev_state,
                        group,
                        chat_id: state.chat_id,
                    };
                    crate::notice::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                NoticeMenu::Unknown(text) => {
                    log::debug!("{}.callback | State::Notice > Unknown command received: '{}'", dbgid, text);
                    crate::notice::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                NoticeMenu::Done => crate::states::reload(bot.clone(), q.regular_message().unwrap(), dialogue, state.prev_state).await?,
            }
        }
        State::Subscribe(state) => {
            log::debug!("{}.callback | State::Subscribe > state: {:#?}", dbgid, state);
            let input = q.data.to_owned().unwrap_or_default();
            log::debug!("{}.callback | State::Subscribe > Input: {}", dbgid, input);
            let cmd = SubscribeMenu::parse(&input, 0);
            log::debug!("{}.callback | State::Subscribe > Cmd: {:?}", dbgid, cmd);
            match cmd {
                SubscribeMenu::Group(group) => {
                    log::debug!("{}.callback | State::Subscribe > Notice Will send to the: '{}' group", dbgid, group);
                    let state = SubscribeState {
                        prev_state: state.prev_state,
                        group,
                        chat_id: state.chat_id,
                        user: q.from.clone()
                    };
                    crate::subscribe::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                SubscribeMenu::Unknown(text) => {
                    log::debug!("{}.callback | State::Subscribe > Unknown command received: '{}'", dbgid, text);
                    crate::subscribe::enter(bot, q.regular_message().unwrap().to_owned(), dialogue, state).await?
                }
                SubscribeMenu::Done => crate::states::reload(bot.clone(), q.regular_message().unwrap(), dialogue, state.prev_state).await?,
            }            
        }
        State::Help(state) => {
            log::debug!("{}.callback | State::Help > state: {:#?}", dbgid, state);
            crate::states::reload(bot.clone(), q.regular_message().unwrap(), dialogue, state.prev_state).await?
        }
        // State::GeneralMessage(state) => {
        //     log::debug!("{}.callback | State::GeneralMessage > receiver: {}", state.receiver);
        // },
    }
    Ok(())
}
///
/// Update or insert user
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
    db::user_insert(user_id, name, Some(contact), None, Some(Utc::now()), &[UserRole::Guest]).await?;
    Ok(())
}
