use indexmap::IndexMap;
use teloxide::prelude::*;
use crate::{db, kernel::error::HandlerResult, loc::LocaleTag, states::{MainState, MyDialogue}, user::{grant_access::{self, GrantAccessState}, user::User, user_role::UserRole}};
///
/// RequestAccess menu
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
    // pub role: Option<UserRole>,     // Role to be granted
}
//
//
impl Default for RequestAccessState {
    fn default() -> Self {
        Self { 
            prev_state: MainState::default(), 
            // role: None, 
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
///
/// New user (state.user_id) requested access
pub async fn enter(bot: Bot, msg: Message, dialogue: MyDialogue, state: RequestAccessState) -> HandlerResult {
    // let user_id = state.user.id;
    let user_name = state.user.name.clone();
    log::debug!("request_access.enter | state: {:#?}", state);
    // let roles =  match db::user_roles(user_id).await {
    //     Ok(roles) => roles,
    //     Err(err) => {
    //         log::warn!("request_access.enter | Error: {:#?}", err);
    //         IndexMap::new()
    //     }
    // };
    log::debug!("request_access.enter | User '{}' requested access...", user_name);
    // let text = format!("Select a Role to be granted for user '{}'", user_name);
    // let users = db::users(None::<&str>).await?;
    // // Moderator avaliable in the DB
    // let moders: Vec<User> = users.into_iter().filter_map(|(_, user)| {
    //     if user.role.contains(&UserRole::Moder) {
    //         Some(user)
    //     } else {
    //         None
    //     }
    // }).collect();
    let state = GrantAccessState { prev_state: MainState::default(), user: state.user, role: None };
    grant_access::enter(bot, msg, dialogue, state);
    Ok(())
}
