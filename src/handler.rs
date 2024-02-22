use std::{error::Error, sync::Arc};
use teloxide::{
    prelude::*,
    utils::command::BotCommands,
    types::{
        ParseMode,
    },
};
use std::env::var;

use crate::chat_server::{ChatServer, UserData};

#[derive(BotCommands, PartialEq, Debug)]
#[command(rename_rule = "lowercase")]
enum Command {
    Top,
    Registration,
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

    if let Ok(command) = Command::parse(text, var("BOT_NAME")?.as_str()) {
        response = match command {
            Command::Registration => {
                let user = m.from().unwrap().clone();
                let sender = UserData::get_new_user(user.clone());
                let username = &sender.username.to_string();
                let mut str =
                    "*Регистрация НЕ выполнена*\\.\n\
                    Необходимо установить в профиле Имя пользователя\\(Username\\)\\.\n\
                    При смене имени пользователя необходимо снова запустить команду регистрации\\.";
                if !username.is_empty() {
                    //Искать пользователя не по ID, а по username
                    if !cs.user_exist(username)? {
                        cs.add_user(&sender)?;
                        str = "Регистрация прошла успешно";
                    } else {
                        str = "Вы уже зарегистрированны";
                    }
                }

                str.to_string()
            },
            Command::Top => {
                let str = cs.get_top()?;
                str
            },
        }
    } else {

        if m.text()
            .unwrap()
            .to_lowercase()
            .contains(var("KEY_WORD")?.as_str())
        {

            let sender = m.from().unwrap().clone();
            let sender_username = sender.username.unwrap_or_else(|| String::from(""));
            for word in text.split(" ") {
                if word.contains("@") {
                    let username = word.replace("@", "");
                    if &username != &sender_username
                        && &username != &var("BOT_USERNAME")? {
                        cs.raise_units(&username)?;
                        response = format!("@{} получил {}", &username, var("COIN")?.as_str());

                        let id= cs.get_id_by_username(&username)?;
                        bot.send_message(UserId(id), format!("Вам передали {} от @{}", var("COIN")?.as_str(), &sender_username)).await?;
                    }
                }
            }
        }
    }

    if !response.is_empty() {
        bot.send_message(m.chat.id, response).parse_mode(ParseMode::MarkdownV2).await?;
    }

    Ok(())
}