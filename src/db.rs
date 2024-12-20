use std::{fs, path::{Path, PathBuf}};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use teloxide::types::ChatId;
use crate::{links::Links, menu::MenuItem, subscribe::subscription::{Subscription, Subscriptions}, user::{user::User, user_role::{UserRole, UserRoleDb, UserRoles}}};
///
/// 
pub async fn menu() -> Result<IndexMap<String, MenuItem>, String> {
    let menu: IndexMap<String, MenuItem> = IndexMap::from([
        ("Links", "/Links"),
        ("Notice", "/Notice"),
        ("RequestAccess", "/RequestAccess"),
        ("Subscribe", "/Subscribe"),
        ("Help", "/Help"),
    ])
        .into_iter()
        .map(|(title, command)| (title.to_owned(), MenuItem { title: title.to_owned(), command: command.to_owned()}))
        .collect();
    Ok(menu)
}
///
/// Inserts a user
pub async fn user_insert(user_id: u64, name: String, contact: Option<String>, address: Option<String>, last_seen: Option<DateTime<Utc>>, role: &[UserRole]) -> Result<(), String> {
    let path = "./assets/users.json";
    let mut users = match users(Some(path)).await {
        Ok(users) => users,
        Err(err) => {
            log::info!("db.user | error: {:#?}", err);
            IndexMap::<String, User>::new()
        }
    };
    let last_seen = last_seen.map_or("".to_owned(), |t| t.to_rfc3339());
    match users.get_mut(&user_id.to_string()) {
        Some(user) => {
            user.name = name.to_owned();
            user.contact = contact.clone();
            user.address = address.clone();
            user.last_seen = last_seen;
        }
        None => {
            users.insert(
                user_id.to_string(),
                User {
                    id: ChatId(user_id as i64),
                    name: name.clone(),
                    contact: contact,
                    address: address,
                    subscriptions: vec![],
                    last_seen,
                    role: role.into(),
                } 
            );
        }
    };
    match fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path) {
        Ok(f) => {
            match serde_json::to_writer_pretty(f, &users) {
                Ok(_) => Ok(()),
                Err(err) => Err(format!("db.user_insert | User '{}' ({}) - Error {:#?}", name, user_id, err)),
            }
        }
        Err(err) => Err(format!("db.user_insert | Error: {:#?}", err)),
    }
}
///
/// Updates or Inserts a user
pub async fn user_update(user: User) -> Result<(), String> {
    let path = "./assets/users.json";
    let mut users = match users(Some(path)).await {
        Ok(users) => users,
        Err(err) => {
            log::info!("db.user | error: {:#?}", err);
            IndexMap::<String, User>::new()
        }
    };
    let user_id = user.id.to_string();
    let user_name = user.name.clone();
    match users.get_mut(&user_id) {
        Some(db_usr) => {
            db_usr.update(user);
        }
        None => {
            users.insert(
                user.id.to_string(),
                user,
            );
        }
    };
    match fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path) {
        Ok(f) => {
            match serde_json::to_writer_pretty(f, &users) {
                Ok(_) => Ok(()),
                Err(err) => Err(format!("db.user_insert | User '{}' ({}) - Error {:#?}", user_name, user_id, err)),
            }
        }
        Err(err) => Err(format!("db.user_insert | Error: {:#?}", err)),
    }
}
///
/// Returns user from storage
#[allow(unused)]
pub async fn user(chat_id: &ChatId) -> Result<User, String> {
    let path = "./assets/users.json";
    log::info!("db.user | config: {:?}", path);
    match load(path) {
        Ok(users) => {
            let users: IndexMap<String, User> = users;
            match users.get(&format!("{}", chat_id.0)) {
                Some(user) => Ok(user.to_owned()),
                None => Err(format!("db.user | User with id '{}' - not found", chat_id)),
            }
        }
        Err(err) => Err(format!("db.user | Error: {:#?}", err)),
    }
}
///
/// Returns users from storage
pub async fn users(path: Option<impl AsRef<Path>>) -> Result<IndexMap<String, User>, String> {
    let path: PathBuf = match path {
        Some(path) => path.as_ref().to_owned(),
        None => PathBuf::from("./assets/users.json"),
    };
    log::info!("db.users | load users from: {:?}", path);
    match load(path) {
        Ok(users) => {
            Ok(users)
        }
        Err(err) => Err(format!("db.users | Error: {:#?}", err)),
    }
}

///
/// 
pub async fn insert_subscription(chat_id: &str, chat_title: &str) -> Result<(), String> {
    log::debug!("db.insert_subscription | Trying register chat {} ({}) ...", chat_title, chat_id);
    match subscriptions().await {
        Ok(mut subscriptions) => {
            match subscriptions.get_mut(chat_id) {
                Some(subscription) => {
                    log::debug!("db.insert_subscription | Chat {} ({}) already registered", chat_title, chat_id);
                    subscription.title = chat_title.to_owned();
                    Ok(())
                }
                None => {
                    log::debug!("db.insert_subscription | Regictering chat {} ({})...", chat_title, chat_id);
                    let subscription = Subscription {
                        id: Some(chat_id.to_owned()),
                        title: chat_title.to_owned(),
                        members: IndexMap::new(),
                    };
                    subscriptions.insert(chat_id.to_string(), subscription);
                    match update_subscriptions(&subscriptions).await {
                        Ok(_) => Ok(()),
                        Err(err) => {
                            let err = format!("db.insert_subscription | Error regictering chat {} ({}): {:#?}", chat_title, chat_id, err);
                            log::warn!("{}", err);
                            Err(err)
                        }
                    }
                }
            }
        }
        Err(err) => {
            let err = format!("db.insert_subscription | Error: {:#?}", err);
            log::debug!("{}", err);
            Err(err)
        }
    }    
}
///
/// 
pub async fn remove_subscription(chat_id: ChatId) -> Result<(), String> {
    let _ = chat_id;
    let err = format!("db.remove_subscription | Not implemented yet");
    log::debug!("{}", err);
    Err(err)
}
///
/// 
pub async fn update_subscriptions(subscriptions: &Subscriptions) -> Result<(), String> {
    let path = "./assets/subscription.json";
    match fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path) {
        Ok(f) => {
            match serde_json::to_writer_pretty(f, &subscriptions) {
                Ok(_) => Ok(()),
                Err(err) => Err(format!("db.update_subscriptions | Error {:#?}", err)),
            }
        },
        Err(err) => Err(format!("db.update_subscriptions | Error: {:#?}", err)),
    }
}
///
/// Returns subscriptions from storage
pub async fn subscriptions() -> Result<Subscriptions, String> {
    let path = "./assets/subscription.json";
    log::info!("db.subscriptions | load subscriptions from: {:?}", path);
    match load(path) {
        Ok(groups) => {
            let groups: IndexMap<String, Subscription> = groups;
            Ok(groups)
        }
        Err(err) => Err(format!("db.subscriptions | Error: {:#?}", err)),
    }
}
///
/// Returns Links
pub async fn links(user_id: ChatId) -> Result<Links, String> {
    let _ = user_id;
    let path = "./assets/links.json";
    log::info!("db.links | load links from: {:?}", path);
    match load(path) {
        Ok(links) => {
            let links: Links = links;
            Ok(links)
        }
        Err(err) => Err(format!("db.links | Error: {:#?}", err)),
    }
}
///
/// Returns UserRoles
pub async fn user_roles(user_id: ChatId) -> Result<UserRoles, String> {
    let _ = user_id;
    let path = "./assets/user-roles.json";
    log::info!("db.user_roles | load roles from: {:?}", path);
    match load(path) {
        Ok(roles) => {
            let roles: IndexMap<String, UserRoleDb> = roles;
            Ok(roles)
        }
        Err(err) => Err(format!("db.user_roles | Error: {:#?}", err)),
    }
}
///
/// 
fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, String> {
    match fs::read_to_string(&path) {
        Ok(yaml_string) => {
            log::info!("db.load | Path to config: {:?}", path.as_ref());
            match serde_json::from_str(&yaml_string) {
                Ok(data) => {
                    let data: T = data;
                    Ok(data)
                }
                Err(err) => {
                    Err(format!("db.load | Error in: {:?}\n\terror: {:?}", yaml_string, err))
                }
            }
        }
        Err(err) => {
            Err(format!("db.load | File '{:?}' reading error: {:?}", path.as_ref(), err))
        }
    }
}
