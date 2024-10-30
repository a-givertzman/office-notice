use indexmap::IndexMap;
use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode}};
use crate::{db, loc::{loc, LocaleTag}, states::{HandlerResult, MainState, MyDialogue}, user::{user::User, user_role::{UserRole, UserRoles}}};
///
/// Notice menu
#[derive(Debug, Clone, PartialEq)]
pub enum RequestAccessMenu {
   Role(UserRole),    // Selected group to be granted
   Unknown(String), // Unknown command received
   Done,            // Exit menu
}
//
//
impl RequestAccessMenu {
   pub fn parse(s: &str, _loc_tag: LocaleTag) -> Self {
        let input = s.strip_prefix('/').map_or_else(|| ("", s), |input| ("/", input));
        match input {
            ("/", "done" | "Done") => Self::Done,
            ("/", "back" | "Back") => Self::Done,
            ("/", "exit" | "Exit") => Self::Done,
            ("/", input) => {
                match serde_json::from_str(input) {
                    Ok(role) => {
                        let role: UserRole = role;
                        Self::Role(role)
                    }
                    Err(err) => {
                        log::error!("RequestAccessMenu.parse | Unknown Role: '{:?}', \n\t Parsing error: {:#?}", input, err);
                        Self::Unknown(s.to_owned())
                    }
                }
            }
            (_, _) => Self::Unknown(s.to_owned()),
        }
   }
}
///
/// State holding values rquired for request access process
#[derive(Debug, Clone)]
pub struct RequestAccessState {
    pub prev_state: MainState,      // Where to go on Back btn
    pub user: User,                 // User doing request access
    pub role: Option<UserRole>,     // Role to be granted
}
//
//
impl Default for RequestAccessState {
    fn default() -> Self {
        Self { 
            prev_state: MainState::default(), 
            role: None, 
            user: User {
                id: ChatId(0), 
                name: "".to_owned(),
                ..Default::default()
            }
        }
    }
}
///
///  
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: RequestAccessState) -> HandlerResult {
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
            let title = roles.get(&role.to_string()).map_or(role.to_string(), |role| role.title.clone());
            let text = format!("{}, role '{}' granted for you!", user_name, title);
            bot.send_message(user_id, text)
                // .edit_message_media(user_id, message_id, media)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        // New user (state.user_id) requested access
        None => {
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
            dialogue.update(state.clone()).await?;
            view(&bot, &msg, &state, &roles, text, moders).await?;
        }
    }
    Ok(())
}
///
/// Menu buttons to select a role to be granted
pub async fn view(bot: &Bot, msg: &Message, state: &RequestAccessState, roles: &UserRoles, text: impl Into<String>, moders: Vec<User>) -> HandlerResult {
    let _ = state.user.id;
    match moders.first() {
        Some(moder) => {
            let markup = markup(&roles).await?;
            bot.send_message(moder.id, text)
                .reply_markup(markup)
                .parse_mode(ParseMode::Html)
                .await?;
            Ok(())
        }
        None => Err("()".into()),
    }
}
///
/// 
async fn markup(roles: &UserRoles) -> Result<InlineKeyboardMarkup, String> {
    let mut buttons: Vec<InlineKeyboardButton> = roles
        .iter()
        .map(|(role_id, role)| {
            InlineKeyboardButton::callback(
                role.title.clone(),
                format!("/{}", role_id),
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
