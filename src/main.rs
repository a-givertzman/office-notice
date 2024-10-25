mod config;
mod message;
mod loc;
mod links;
mod notice;
mod subscribe;
mod menu;
mod subscription;
mod db;
mod user;
mod states;
mod help;
mod request_access;
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
            log::info!("main::Unable to send message to the service chat");
            // if environment::log(&text).await.is_none() {
            // };
        };
        Box::pin(fut)
   }
}
///
/// 
async fn default_handler(upd: Arc<Update>) {
    match &upd.kind {
        UpdateKind::MyChatMember(chat_member) => {
            if chat_member.new_chat_member.is_member() {    //m.old_chat_member.is_left() && 
                if let Err(err) = crate::states::new_chat_member(chat_member).await {
                    log::warn!("main | Error in states.new_chat_member: {:?}", err);
                };
            } else if chat_member.new_chat_member.is_left() { // m.old_chat_member.is_member() && 
                if let Err(err) = crate::states::left_chat_member(chat_member).await {
                    log::warn!("main | Error in states.left_chat_member: {:?}", err);
                };
            }
        }
        _ => {
            log::warn!("main | Unhandled update: {:?}", upd);
        }
    }
}
///
/// 
static BOT_NAME: &str = "TKZ Office Notice bot";
///
///
#[tokio::main]
async fn main() {
    clear_console();
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");
    let config = AppConfig::read("./config.yaml");
    log::info!("config: {:#?}", config);
    env::set_var("TELOXIDE_TOKEN", config.bot.connection.token);
    let bot = Bot::from_env();
    Dispatcher::builder(bot.clone(), states::schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        // All unhandled updates redirects to the default_handler
        .default_handler(default_handler)
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