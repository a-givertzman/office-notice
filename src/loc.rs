use lazy_static::lazy_static;
use std::collections::HashMap;
///
/// 
pub type LocaleTag = u32;
///
/// 
pub fn loc(key: impl Into<String>) -> String {
    let key = key.into();
    match RU.get(key.as_str()) {
        Some(ru) => ru.to_string(),
        None => key,
    }
}
//
//
lazy_static!{
    static ref RU: HashMap<&'static str, &'static str> = vec![
        ("You are in the main menu", "Вы в главном меню"),
        ("Sorry, the bot has been restarted", "Извините, бот был перезапущен"),
        ("Error, start again", "Ошибка, начните заново"),
        ("Cancel", "Отмена")
    ].into_iter().collect();
}