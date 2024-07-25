use std::{collections::HashMap, fs, path::Path};
use serde::de::DeserializeOwned;
use teloxide::types::{ChatId, UserId};
use crate::{links::Links, subscription::{Subscription, Subscriptions}, user::User};
///
/// 
pub async fn user_insert(user_id: u64, name: String, contact: Option<String>, address: Option<String>) -> Result<(), String> {
    let path = "./users.json";
    let new_user = User {
        id: ChatId(user_id as i64),
        name: name.clone(),
        contact: contact.clone(),
        address: address.clone(),
        subscriptions: vec![],
    };
    match fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path) {
        Ok(f) => {
            let mut users = match serde_json::from_reader(f.try_clone().unwrap()) {
                Ok(value) => {
                    let users: HashMap<u64, User> = value;
                    users
                }
                Err(_) => {
                    HashMap::<u64, User>::new()
                }
            };
            let name = &name;
            users.entry(user_id)
                .and_modify(|user| {
                    user.name = name.to_owned();
                    user.contact = contact;
                    user.address = address;
                })
                .or_insert(new_user);
            match serde_json::to_writer(f, &users) {
                Ok(_) => Ok(()),
                Err(err) => Err(format!("DB.user_insert | User '{}' ({}) - Error {:#?}", name, user_id, err)),
            }
        },
        Err(_) => todo!(),
    }
}
///
/// Returns user from storage
pub async fn user(user_id: u64) -> Result<User, String> {
    let path = "./users.json";
    log::info!("DB.user | config: {:?}", path);
    match load(path) {
        Ok(users) => {
            let users: HashMap<u64, User> = users;
            match users.get(&user_id) {
                Some(user) => Ok(user.to_owned()),
                None => Err(format!("DB.user | User with id '{}' - not found", user_id)),
            }
        }
        Err(err) => Err(format!("DB.user | Error: {:#?}", err)),
    }
}
///
/// Returns subscriptions from storage
pub async fn subscriptions(user_id: UserId) -> Result<Subscriptions, String> {
    let path = "./subscription.json";
    log::info!("DB.subscriptions | load subscriptions from: {:?}", path);
    match load(path) {
        Ok(groups) => {
            let groups: HashMap<String, Subscription> = groups;
            Ok(groups)
            // Ok(NodeResult::Groups(groups))
        }
        Err(err) => Err(format!("DB.subscriptions | Error: {:#?}", err)),
    }
    //  Err("DB.subscriptions | Not implemented".to_owned())
}
///
/// Returns Links
pub async fn links(user_id: UserId) -> Result<Links, String> {
    let _ = user_id;
    let path = "./links.json";
    log::info!("DB.links | load links from: {:?}", path);
    match load(path) {
        Ok(links) => {
            let links: Links = links;
            Ok(links)
        }
        Err(err) => Err(format!("DB.links | Error: {:#?}", err)),
    }
}
// }
///
/// 
fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, String> {
    match fs::read_to_string(&path) {
        Ok(yaml_string) => {
            log::info!("DB.load | Path to config: {:?}", path.as_ref());
            match serde_yaml::from_str(&yaml_string) {
                Ok(data) => {
                    let data: T = data;
                    Ok(data)
                }
                Err(err) => {
                    Err(format!("DB.load | Error in: {:?}\n\terror: {:?}", yaml_string, err))
                }
            }
        }
        Err(err) => {
            Err(format!("DB.load | File '{:?}' reading error: {:?}", path.as_ref(), err))
        }
    }

}
// ///
// /// 
// fn store<T: Serialize>(path: impl AsRef<Path>, value: T) -> Result<(), String> {
