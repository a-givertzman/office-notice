mod config;
mod general;
mod loc;
mod environment;
mod links;
mod notice;
mod subscribe;
mod callback;
mod navigation;
mod subscription;
mod db;
mod user;
mod states;

use std::{env, fmt::Debug, sync::Arc};
use futures::future::BoxFuture;
use config::AppConfig;
use states::State;
use teloxide::{dispatching::dialogue::InMemStorage, error_handlers::ErrorHandler, prelude::*};

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
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");
    let config = AppConfig::read("", "./config.yaml");
    log::info!("config: {:#?}", config);
    // let bot = Bot::new(config.bot.connection.token);
    let bot = Bot::from_env();
    Dispatcher::builder(bot.clone(), states::schema())
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    // .default_handler(|upd| async move {
    //    environment::log(&format!("main::Unhandled update: {:?}", upd)).await;
    // })
    // If the dispatcher fails for some reason, execute this handler.
    .error_handler(Arc::new(MyErrorHandler{}))
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    // .dispatch_with_listener(
    //    update_listener,
    //    LoggingErrorHandler::with_custom_text("main::An error from the update listener"),
    // )
    .await;
}
