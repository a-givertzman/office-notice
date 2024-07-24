/* ===============================================================================
Restaurant menu bot.
User interface with inline buttons. 27 May 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2020-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use teloxide::{
    prelude::*,
    types::{InputFile, InlineKeyboardButton, InlineKeyboardMarkup,
       CallbackQuery, InputMedia, ParseMode, InputMediaPhoto,
    },
};
use crate::callback::Command;
use arraylib::iter::IteratorExt;

use crate::environment as env;
use crate::states::*;
use crate::db as db;
use crate::subscription::*;
use crate::loc::*;
 
 pub async fn enter(bot: Bot, msg: Message, state: MainState, mode: WorkTime) -> HandlerResult {
    let tag = state.tag;
    let user_id = state.user_id; // user needs to sync with cart
    let chat_id = msg.chat.id;
    // Load root node with children
    let load_mode = match mode {
        WorkTime::All => db::LoadNode::Links(user_id),
        WorkTime::Now => db::LoadNode::Groups(user_id),
        WorkTime::AllFrom(id) => db::LoadNode::Subscriptions(user_id),
    };
    let node =  db::node(load_mode).await?;
    match node {
        db::NodeResult::None => {
            let text = match mode {
                WorkTime::Now => loc("There is no currently open places"), // "There is no currently open places"
                _ => loc("Error, no entries - contact administrator"), // "Error, no entries - contact administrator"
            };
            bot.send_message(chat_id, text).await?;
        }
        db::NodeResult::Links(_) => {
            let text = loc("markup is not implemented for Links");
            bot.send_message(chat_id, text).await?;
        }
        db::NodeResult::Groups(subscription) => {
            // All is ok, collect and display info
            let user_id = state.user_id; // user needs to sync with cart
            let markup = markup(&subscription, mode, user_id, tag).await?;
            let text = subscription.title;
            bot.send_message(chat_id, text)
            // .caption(text)
            .reply_markup(markup)
            .parse_mode(ParseMode::Html)
            // .disable_notification(true)
            .await?;
        }
    }
    Ok(())
}
 
 
async fn msg(bot: &Bot, user_id: UserId, text: &str) -> Result<(), String> {
    bot.send_message(user_id, text)
    .await
    .map_err(|err| format!("inline::msg {}", err))?;
    Ok(())
}
 
pub async fn view(bot: &Bot, q: CallbackQuery, node_id: i32, mode: WorkTime, tag: LocaleTag) -> Result<(), String> {
 
    let user_id = q.from.id;
 
    // Load node from database
    let load_mode = match mode {
       WorkTime::All | WorkTime::AllFrom(_) => db::LoadNode::Groups(user_id),
       WorkTime::Now => db::LoadNode::Links(user_id),
    };
    let node =  db::node(load_mode).await?;
    if node.is_none() {
       let text = loc("Error, data deleted, start again");
       msg(bot, user_id, &text).await?;
       return Ok(())
    }
 
    // // Collect info
    let subscription = match node {
        db::NodeResult::None => panic!("naviogation.view | NodeResult is None"),
        db::NodeResult::Links(subscription) => panic!("naviogation.view | NodeResult is Links"),
        db::NodeResult::Groups(subscription) => subscription,
    };
    let markup = markup(&subscription, mode, user_id, tag).await?;
 
    let text = format!("Navigation view");
 
    // Message to modify
    let message = q.message;
    if message.is_none() {
       // "Error, update message is invalid, please start again"
       let text = loc("Error, update message is invalid, please start again");
       msg(bot, user_id, &text).await?;
       return Ok(())
    }
    // let chat_id = ChatId::Id(message.chat_id());
    let message_id = message.unwrap().id;
    msg(bot, user_id, &text).await?;
    bot.edit_message_text(user_id, message_id, text)
        // .edit_message_media(user_id, message_id, media)
        .reply_markup(markup)
    .await
    .map_err(|err| format!("inline::view {}", err))?;
    Ok(())
}
 
//  fn node_text(node: &Node, tag: LocaleTag) -> String {
 
//     let mut res = format!("<b>{}</b>", node.title);
 
//     // Do not display description from 1 symbol
//     if node.descr.len() > 1 {
//        res = res + "\n" + node.descr.as_str();
//     };
 
//     // Display time only if it sets
//     if node.is_time_set() {
//        // "{}\nWorking time: {}-{}"
//        let fmt = loc(Key::CommonTimeFormat, tag, &[]);
//        let args: Args = &[
//           &res,
//           &node.time.0.format(&fmt),
//           &node.time.1.format(&fmt)
//        ];
//        res = loc(Key::NavigationNodeText1, tag, args);
//     };
 
//     if node.price != 0 {
//        // "{}\nPrice: {}"
//        res = loc(Key::NavigationNodeText2, tag, &[&res, &env::price_with_unit(node.price)])
//     }
 
//     res
//  }
 
async fn markup(subscribtion: &Subscription, mode: WorkTime, user_id: UserId, tag: LocaleTag) -> Result<InlineKeyboardMarkup, String> {
 
    // Prepare command
    let pas = match mode {
       WorkTime::All | WorkTime::AllFrom(_) => Command::Pass(0),
       WorkTime::Now => Command::PassNow(0),
    };
    let pas = String::from(pas.as_ref());
 
    // Create buttons for each child
    let buttons: Vec<InlineKeyboardButton> = subscribtion.children
    .iter()
    .map(|child| {
       InlineKeyboardButton::callback(
       child.title.clone(),
       format!("{}{}", pas, child.id)
    )})
    .collect();
 
    // Separate into long and short
    let (long, mut short) : (Vec<_>, Vec<_>) = buttons
    .into_iter()
    .partition(|n| n.text.chars().count() > 21);
 
    // If price not null add button for cart with amount
    if subscribtion.price != 0 {
       // Display only title or title with amount
       let amount = 0;
       // "+🛒 ({})", "+🛒"
       let caption = if amount > 0 {
          loc("+🛒 ({})")
       } else {
          loc("+🛒")
       };
 
       let cmd = match mode {
          WorkTime::All | WorkTime::AllFrom(_) => Command::IncAmount(0),
          WorkTime::Now => Command::IncAmountNow(0),
       };
       let cmd = String::from(cmd.as_ref());
       let button_inc = InlineKeyboardButton::callback(
          caption,
          format!("{}{}", cmd, subscribtion.id)
       );
       short.push(button_inc);
 
       // Add decrease button
       if amount > 0 {
          let cmd = match mode {
             WorkTime::All | WorkTime::AllFrom(_) => Command::DecAmount(0),
             WorkTime::Now => Command::DecAmountNow(0),
          };
          let cmd = String::from(cmd.as_ref());
          let button_dec = InlineKeyboardButton::callback(
             loc("-🛒"), // "-🛒"
             format!("{}{}", cmd, subscribtion.id)
          );
          short.push(button_dec);
       }
    }
 
    // Put in vec last unpaired button, if any
    let mut last_row = vec![];
    if short.len() % 2 == 1 {
       let unpaired = short.pop();
       if unpaired.is_some() {
          last_row.push(unpaired.unwrap());
       }
    }
 
    // Long buttons by one in row
    let markup = long.into_iter()
    .fold(InlineKeyboardMarkup::default(), |acc, item| acc.append_row(vec![item]));
 
    // Short by two
    let mut markup = IteratorExt::array_chunks::<[_; 2]>(short.into_iter())
    .fold(markup, |acc, [left, right]| acc.append_row(vec![left, right]));
 
    // Back button
    if subscribtion.id > 0 {
       let button_back = InlineKeyboardButton::callback(
          loc("⏪Back"), // "⏪Back"
          format!("{}{}", pas, subscribtion.parent)
       );
       last_row.push(button_back);
    }
 
    // Add the last unpaired button and the back button
    if !last_row.is_empty() {
       markup = markup.append_row(last_row);
    }
 
    Ok(markup)
}