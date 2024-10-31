use indexmap::IndexMap;
use teloxide::{payloads::SendMessageSetters, prelude::Requester, types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode}, Bot};
use crate::{db, kernel::error::HandlerResult, loc::{loc, LocaleTag}, states::{MainState, MyDialogue, StartState, State}};
use super::{user::User, user_role::{UserRole, UserRoles}};
///
/// RequestAccess menu
#[derive(Debug, Clone, PartialEq)]
pub enum GrantAccessMenu {
   Role((UserRole, ChatId)),      // Selected Role to be granted to User
   Unknown(String),     // Unknown command received
   Done,                // Exit menu
}
//
//
impl GrantAccessMenu {
   pub fn parse(s: &str, _loc_tag: LocaleTag) -> Self {
        let input = s.strip_prefix('/').map_or_else(|| ("", s), |input| ("/", input));
        match input {
            ("/", "done" | "Done") => Self::Done,
            ("/", "back" | "Back") => Self::Done,
            ("/", "exit" | "Exit") => Self::Done,
            ("/", input) => {
                let mut input = input.split(":");
                match input.next() {
                    Some(role) => match input.next() {
                        Some(user_id) => {
                            match user_id.parse() {
                                Ok(user_id) => {
                                    // let user_id = user_id;
                                    match role.to_lowercase().as_str() {
                                        "GrantRole/Admin"  | "grantrole/admin"  | "Admin"  | "admin" => Self::Role((UserRole::Admin, ChatId(user_id))),
                                        "GrantRole/Moder"  | "grantrole/moder"  | "Moder"  | "moder" => Self::Role((UserRole::Moder, ChatId(user_id))),
                                        "GrantRole/Sender" | "grantrole/sender" | "Sender" | "sender" => Self::Role((UserRole::Sender, ChatId(user_id))),
                                        "GrantRole/Member" | "grantrole/member" | "Member" | "member" => Self::Role((UserRole::Member, ChatId(user_id))),
                                        "GrantRole/Guest"  | "grantrole/guest"  | "Guest"  | "guest" => Self::Role((UserRole::Guest, ChatId(user_id))),
                                        _ => {
                                            log::error!("GrantAccessMenu.parse | Unknown Role: '{:?}'", input);
                                            Self::Unknown(s.to_owned())
                                        }
                                    }
                                },
                                Err(err) => {
                                    log::error!("GrantAccessMenu.parse | User Id '{}' parse error: '{:?}'", user_id, err);
                                    Self::Unknown(s.to_owned())
                                }
                            }
                        },
                        None => {
                            log::error!("GrantAccessMenu.parse | User Id not found in '{:?}'", input);
                            Self::Unknown(s.to_owned())
                        }
                    },
                    None => {
                        log::error!("GrantAccessMenu.parse | Role not found in '{:?}'", input);
                        Self::Unknown(s.to_owned())
                    }
                }
            }
            (_, _) => Self::Unknown(s.to_owned()),
        }
   }
}
///
/// State holding values rquired for grant access process
#[derive(Debug, Clone)]
pub struct GrantAccessState {
    /// Where to go on Back btn
    pub prev_state: Box<State>,
    /// User doing request access
    pub user: User,
    /// Role to be granted
    pub role: Option<UserRole>,
}
//
//
impl Default for GrantAccessState {
    fn default() -> Self {
        Self { 
            prev_state: Box::new(State::Start(StartState::default())), 
            role: None, 
            user: User {
                id: ChatId(0), 
                name: "".to_owned(),
                contact: None,
                address: None,
                subscriptions: vec![],
                last_seen: "".to_owned(),
                role: vec![],
            }
        }
    }
}
//
//
impl From<State> for GrantAccessState {
    fn from(value: State) -> Self {
        match value {
            State::GrantAccess(state) => state,
            _ => panic!("GrantAccessState.try_from | Illegal input state '{:?}'", value)
        }
    }
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: GrantAccessState) -> HandlerResult {
    let user_id = state.user.id;
    let user_name = state.user.name.clone();
    log::debug!("request_access.enter | state: {:#?}", state);
    let roles =  match db::user_roles(user_id).await {
        Ok(roles) => roles,
        Err(err) => {
            log::warn!("request_access.enter | Error: {:#?}", err);
            IndexMap::new()
        }
    };
    match &state.role {
        // Moder granted a role
        Some(role) => {
            log::debug!("request_access.enter | Moder granting a role: {:?}...", role);
            let mut to_user = state.user;
            to_user.add_role(role.to_owned());
            db::user_update(to_user).await?;
            let title = roles.get(&role.to_string()).map_or(role.to_string(), |role| role.title.clone());
            let text = format!("{}, role '{}' granted for you!", user_name, title);
            dialogue.update(*state.prev_state).await?;
            bot.send_message(user_id, text)
                // .edit_message_media(user_id, message_id, media)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        // New user (state.user_id) requested access
        None => {
            log::debug!("request_access.enter | User '{}' requested access...", user_name);
            let text = format!("Select a Role to be granted for user '{}'", user_name);
            let users = db::users(None::<&str>).await?;
            // Moderator avaliable in the DB
            let moders: Vec<User> = users.into_iter().filter_map(|(_, user)| {
                if user.role.contains(&UserRole::Moder) {
                    Some(user)
                } else {
                    None
                }
            }).collect();
            match moders.first() {
                Some(moder) => {
                    // dialogue.update(state.clone()).await?;
                    view(&bot, &state, &roles, text, moder).await?;
                }
                None => return Err(format!("request_access.enter | No moderators found to grant access for User '{}'", user_name).into()),
            }
        
        }
    }
    Ok(())
}
///
/// Menu buttons to select a role to be granted
pub async fn view(bot: &Bot, state: &GrantAccessState, roles: &UserRoles, text: impl Into<String>, moder: &User) -> HandlerResult {
    let _ = state.user.id;
    let markup = markup(&roles, &state.user).await?;
    bot.send_message(moder.id, text)
        .reply_markup(markup)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
///
/// 
async fn markup(roles: &UserRoles, to_user: &User) -> Result<InlineKeyboardMarkup, String> {
    let mut buttons: Vec<InlineKeyboardButton> = roles
        .iter()
        .map(|(role_id, role)| {
            InlineKeyboardButton::callback(
                role.title.clone(),
                format!("/GrantRole/{}:{}", role.role.to_string(), to_user.id),
        )})
        .collect();
    let button_back = InlineKeyboardButton::callback(
        loc("⏪Back"), // "⏪Back"
        format!("/back")
    );
    buttons.push(button_back);
    let markup = buttons.into_iter()
    .fold(InlineKeyboardMarkup::default(), |acc, item| acc.append_row(vec![item]));
    Ok(markup)
}
