use std::{collections::HashMap, fs, path::Path};
use serde::de::DeserializeOwned;
use crate::user::User;
///
/// 
pub async fn user_insert(user_id: u64, name: String, contact: String) -> Result<(), String> {
    let path = "./users.json";
    let new_user = User {
        name: name.clone(),
        contact: contact.clone(),
        address: "".to_owned(),
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
/// 
pub async fn user(user_id: u64) -> Result<User, String> {
    let path = "./users.json";
    log::info!("BotConfig.read | config: {:?}", path);
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
// ///
// /// 
// fn store<T: Serialize>(path: impl AsRef<Path>, value: T) -> Result<(), String> {

// }
///
/// 
fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, String> {
    match fs::read_to_string(&path) {
        Ok(yaml_string) => {
            log::info!("BotConfig.read | Path to config: {:?}", path.as_ref());
            match serde_yaml::from_str(&yaml_string) {
                Ok(data) => {
                    let data: T = data;
                    Ok(data)
                }
                Err(err) => {
                    Err(format!("DB.user | Error in config: {:?}\n\terror: {:?}", yaml_string, err))
                }
            }
        }
        Err(err) => {
            Err(format!("DB.user | File '{:?}' reading error: {:?}", path.as_ref(), err))
        }
    }

}
