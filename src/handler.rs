use std::{error::Error, sync::Arc};
use teloxide::{
    prelude::*,
    utils::command::BotCommands,
    types::{
        ParseMode,
    },
};
use chrono::{Datelike, Utc, NaiveDate};
use chrono::format::strftime::StrftimeItems;
use crate::chat_server::{ChatServer, UserData};

#[derive(BotCommands, PartialEq, Debug)]
#[command(rename_rule = "lowercase", description = "*Описание команд*")]
enum Command {
    #[command(description = "Описание")]
    Start,
    #[command(description = "Активация бота\\.\
    \n`/reg \\<Ключ активации\\>`")]
    Reg(String),
    #[command(description = "Топ")]
    Top,
}

pub async fn handle(
    bot: Bot,
    m: Message,
    cs: Arc<ChatServer>
) -> Result<(), Box<dyn Error + Send + Sync>>
{

    let text =
        match m.text(){
            Some(text) => text,
            None => return Ok(())
        };

    let mut response = String::from("");

    if let Ok(command) = Command::parse(text, cs.bot_username.as_str()) {
        response = match command {
            Command::Start => {
                let str =  Command::descriptions().to_string();
                str
            },
            Command::Reg(key) => {
                let mut str = format!("Регистрация возможна через личное сообщение боту @{}", cs.bot_username);
                if m.chat.id.0 > 0i64 {
                    str = String::from("*Регистрация НЕ выполнена*\\.\n\
                Ключ задан неверно");
                    if key == cs.registration_key {
                        let user = m.from().unwrap().clone();
                        let sender = UserData::get_new_user(user.clone());
                        let username = &sender.username.to_string();
                        str = String::from("*Регистрация НЕ выполнена*\\.\n\
                    Необходимо установить в профиле Имя пользователя\\(Username\\)\\.\n\
                    При смене имени пользователя необходимо снова запустить команду регистрации\\.");
                        if !username.is_empty() {
                            if !cs.user_exist(username)? {
                                cs.add_user(&sender)?;
                                str = String::from("Регистрация прошла успешно");
                            } else {
                                str = String::from("Вы уже зарегистрированны");
                            }
                        }
                    }
                }
                str
            },
            Command::Top => {
                let user = m.from().unwrap().clone();
                let username = user.username.unwrap_or_else(|| String::from(""));
                let mut str = format!("*Вы НЕ зарегистрированны*\\.\n\
                Регистрация возможна через личное сообщение боту @{}", cs.bot_username);
                if cs.user_exist(&username)? {
                    str = cs.get_top()?;
                }
                str
            },
        }
    } else {

        if m.text()
            .unwrap()
            .to_lowercase()
            .contains(cs.key_word.as_str())
        {
            let sender = m.from().unwrap().clone();
            let sender_username = sender.username.unwrap_or_else(|| String::from(""));
            if !cs.user_exist(&sender_username)? {
                response = format!("*Вы НЕ зарегистрированны*\\.\n\
                Регистрация возможна через личное сообщение боту @{}", cs.bot_username);
                bot.send_message(m.chat.id, response.replace("_","\\_")).parse_mode(ParseMode::MarkdownV2).await?;
                return Ok(());
            }
            for word in text.split(" ") {
                
                if !word.contains("@") {
                    continue;
                }
                let username = word.replace("@", "");
                if !cs.user_exist(&username)? {
                    response = format!("{}Пользователь @{} не зарегистрирован", response + "\n", &username);
                    continue;
                }
                if &username == &sender_username
                    || &username == cs.bot_username.as_str() {
                    continue;
                }
                
                let now = Utc::now();
                let mut limitation_data = cs.get_coins_per_day(&sender_username)?;
                let date = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
                let current_date = date.format_with_items(StrftimeItems::new("%Y-%m-%d")).to_string();

                if limitation_data.current_date != current_date {
                    cs.reset_limits(&sender_username, &current_date)?;
                    limitation_data.coins_per_day = 0;
                }

                if limitation_data.coins_per_day < cs.max_by_day_coins {
                    cs.increase_coin_count(&username)?;
                    cs.increase_coin_per_day_count(&sender_username)?;
                    response = format!("{}@{} получил {}", response + "\n", &username, cs.coin);
                    let id = cs.get_id_by_username(&username)?;
                    if m.chat.id.0 > 0i64 {
                        bot.send_message(UserId(id), format!("Вам передали {} от @{}", cs.coin, &sender_username)).await?;
                    }
                } else {
                    response = format!("{}Вы привысили максимальное количество {} в день", response + "\n", cs.coin);
                }
            }   
        }
    }

    if !response.is_empty() {
        bot.send_message(m.chat.id, response.replace("_","\\_")).parse_mode(ParseMode::MarkdownV2).await?;
    }

    Ok(())
}