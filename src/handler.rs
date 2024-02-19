use std::{error::Error, sync::Arc};
use teloxide::{
    prelude::*,
    utils::command::BotCommands,
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
                let sender = UserData::get_new_user(&user);
                let username = &sender.username.to_string();
                let mut str =
                    "Регистрация НЕ выполнена. \
                    Необходимо установить в профиле Имя пользователя(Username). \
                    При смене имени пользователя необходимо снова запустить команду регистрации.";
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
            .contains(var("KEY_WORD")?.as_str()) {

            //let reply = m.reply_to_message();

            // if reply != Option::None {
            //     let user = reply.unwrap().from().unwrap().clone();
            //     let sender = UserData::get_new_user(&user);
            //
            //     let str_sender_id = &sender.id.to_string();
            //     match m.kind {
            //         Common(ref common_msg) => {
            //             if let Some(user) = &common_msg.from {
            //                 if &user.id != &sender.id && str_sender_id != &var("BOT_ID")? {
            //
            //                     cs.raise_units(
            //                         &sender.username,
            //                     )?;
            //
            //                     // let user = m.from().unwrap().clone();
            //                     // let recipient = UserData::get_new_user(&user);
            //                     //
            //                     // let units: i32 = cs.get_units_by_id(str_sender_id)?;
            //                     // response = cs.get_unit_addition_message(&sender, &recipient, units)?;
            //
            //                 }
            //             }
            //         }
            //         _ => {}
            //     }
            // } else {

            let sender = m.from().unwrap().clone();
            let username_sender = sender.username.clone().unwrap_or_else(|| String::from(""));
            for word in text.split(" ") {
                if word.contains("@") {
                    let username = word.replace("@", "");
                    if &username != &username_sender
                        && &username != &var("BOT_USERNAME")? {
                        cs.raise_units(
                            &username,
                        )?;
                    }
                }
            }
        }
    }

    if !response.is_empty() {
        bot.send_message(m.chat.id, response).await?;
    }

    Ok(())
}