use std::{collections::HashMap, fs, path::Path};
use serde::de::DeserializeOwned;
use teloxide::types::{ChatId, UserId};
use crate::{subscription::{Owners, Subscription}, user::User};
///
/// 
pub enum LoadNode {
    Links(UserId), // like Id but without disabled
    Groups(UserId), // like EnabledId but without children
    Subscriptions(UserId), // like EnabledId but opened now
}
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
/// 
pub enum NodeResult {
    None,
    Links(Vec<String>),
    Groups(Subscription),
}
impl NodeResult {
    pub fn is_none(&self) -> bool {
        match self {
            NodeResult::None => true,
            _ => false,
        }
    }
    pub fn title(&self) -> String {
        match self {
            NodeResult::None => "Is None".to_owned(),
            NodeResult::Links(_) => "Links".to_owned(),
            NodeResult::Groups(_) => "Groups".to_owned(),
        }
    }
}
///
/// Returns node from storage
pub async fn node(mode: LoadNode) -> Result<NodeResult, String> {
    match &mode {
        LoadNode::Links(user_id) => {
            let path = "./links.json";
            log::info!("DB.node | load links from: {:?}", path);
            match load(path) {
                Ok(links) => {
                    let links: Vec<String> = links;
                    Ok(NodeResult::Links(links))
                }
                Err(err) => Err(format!("DB.node | Error: {:#?}", err)),
            }
        }
        LoadNode::Groups(user_id) => {
            let path = "./groups.json";
            log::info!("BotConfig.node | load links from: {:?}", path);
            match load(path) {
                Ok(groups) => {
                    let groups: HashMap<String, Subscription> = groups;
                    Ok(NodeResult::Groups(Subscription { id: 111, parent: 0, children: vec![], title: "Subscription 111".to_owned(), descr: format!("descr"), enabled: true, banned: false, owners: Owners(UserId(1), UserId(2), UserId(3)), price: 111 }))
                    // Ok(NodeResult::Groups(groups))
                }
                Err(err) => Err(format!("DB.node | Error: {:#?}", err)),
            }
        }
        LoadNode::Subscriptions(user_id) => {
            let path = "./groups.json";
            log::info!("DB.node | load links from: {:?}", path);
            match load(path) {
                Ok(groups) => {
                    let groups: HashMap<String, Subscription> = groups;
                    Ok(NodeResult::Groups(Subscription { id: 111, parent: 0, children: vec![], title: "Subscription 111".to_owned(), descr: format!("descr"), enabled: true, banned: false, owners: Owners(UserId(1), UserId(2), UserId(3)), price: 111 }))
                    // Ok(NodeResult::Groups(groups))
                }
                Err(err) => Err(format!("DB.node | Error: {:#?}", err)),
            }
        }
     }
    //  Err("DB.node | Not implemented".to_owned())
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
            log::info!("DB.load | Path to config: {:?}", path.as_ref());
            match serde_yaml::from_str(&yaml_string) {
                Ok(data) => {
                    let data: T = data;
                    Ok(data)
                }
                Err(err) => {
                    Err(format!("DB.load | Error in config: {:?}\n\terror: {:?}", yaml_string, err))
                }
            }
        }
        Err(err) => {
            Err(format!("DB.load | File '{:?}' reading error: {:?}", path.as_ref(), err))
        }
    }

}
