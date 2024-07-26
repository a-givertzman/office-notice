mod config;
mod general;
mod loc;
mod environment;
mod links;
mod notice;
mod subscribe;
mod callback;
mod menu;
mod subscription;
mod db;
mod user;
mod states;
//
use std::{env, fmt::Debug, process::Command, sync::Arc};
use futures::future::BoxFuture;
use config::AppConfig;
use states::State;
use teloxide::{dispatching::dialogue::InMemStorage, error_handlers::ErrorHandler, prelude::*, types::UpdateKind};
///
///
struct MyErrorHandler {}
impl<E> ErrorHandler<E> for MyErrorHandler
where
    E: Debug,
{
    fn handle_error(self: Arc<Self>, error: E) -> BoxFuture<'static, ()> {
        let text = format!("main::handle_error: {:?}", error);
        log::error!("{}", text);
        let fut = async move {
            if environment::log(&text).await.is_none() {
                log::info!("main::Unable to send message to the service chat");
            };
        };

        Box::pin(fut)
   }
}
///
///
#[tokio::main]
async fn main() {
    clear_console();
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");
    let config = AppConfig::read("", "./config.yaml");
    log::info!("config: {:#?}", config);
    env::set_var("TELOXIDE_TOKEN", config.bot.connection.token);
    // let bot = Bot::new(config.bot.connection.token);
    let bot = Bot::from_env();
    // Settings from environments
    let vars = environment::Vars::from_env(bot.clone()).await;
    match environment::VARS.set(vars) {
        Ok(_) => {environment::log("Bot restarted").await;},
        _ => log::info!("Something wrong with TELEGRAM_LdOG_CHAT"),
    }    
    Dispatcher::builder(bot.clone(), states::schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .default_handler(|upd| async move {
            match &upd.kind {
                UpdateKind::MyChatMember(chat_member) => {
                    if chat_member.new_chat_member.is_member() {    //m.old_chat_member.is_left() && 
                        crate::states::new_chat_member(chat_member).await;
                    } else if chat_member.new_chat_member.is_left() { // m.old_chat_member.is_member() && 
                        crate::states::left_chat_member(chat_member).await;
                    }
                }
                _ => {
                    log::warn!("main | Unhandled update: {:?}", upd);
                    // environment::log(&format!("main::Unhandled update: {:?}", upd)).await;
                }
            }
        })
        // If the dispatcher fails for some reason, execute this handler.
        .error_handler(Arc::new(MyErrorHandler{}))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
///
/// 
fn clear_console() {
    let cmd = Command::new("/bin/bash").arg("-c").arg("clear").spawn();
    match cmd {
        Ok(mut cmd) => {
            match cmd.wait() {
                Ok(_) => {}
                Err(err) => log::warn!("main.clear_console | Failed to execute CLI command 'clear', \n\terror: {:#?}", err),
            }
        }
        Err(err) => log::warn!("main.clear_console | Failed to execute CLI command 'clear', \n\terror: {:#?}", err),
    };
}