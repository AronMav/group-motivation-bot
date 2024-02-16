use std::{error::Error, sync::Arc};
use teloxide::{
    prelude::*,
    types::MessageKind::Common,
    utils::command::BotCommands,
};
use std::env::var;

use crate::chat_server::{ChatServer, UserData};

#[derive(BotCommands, PartialEq, Debug)]
#[command(rename_rule = "lowercase")]
enum Command {
    Top,
    Reg,
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
            Command::Top => {
                let str = cs.get_top()?;
                str
            },
            Command::Reg => {
                let user = m.from().unwrap().clone();
                let sender = UserData::get_new_user(user);
                let str_sender_id = &sender.id.to_string();
                let mut str = "Пользователь уже существует";
                if !cs.user_exist(str_sender_id)? {
                    cs.add_user(&sender)?;
                    str = "Пользователь создан";
                }

                str.to_string()
            },
        }
    } else {

        let reply = m.reply_to_message();

        if reply != Option::None {
            let user = reply.unwrap().from().unwrap().clone();
            let sender = UserData::get_new_user(user);

            let str_sender_id = &sender.id.to_string();
            match m.kind {
                Common(ref common_msg) => {
                    if let Some(user) = &common_msg.from {
                        if &user.id != &sender.id && str_sender_id != &var("BOT_ID")? {
                            if m.text()
                                .unwrap()
                                .to_lowercase()
                                .contains(var("KEY_WORD")?.as_str()) {

                                cs.raise_units(
                                    &sender,
                                )?;

                                let user = m.from().unwrap().clone();
                                let recipient = UserData::get_new_user(user);

                                let units: i32 = cs.get_units_by_id(str_sender_id)?;
                                response = cs.get_unit_addition_message(&sender, &recipient, units)?;
                            }
                        }
                    }
                }
                _ => {}
            }
        } else {

            // let user = m.from().unwrap().clone();
            // let _recipient = UserData::get_new_user(user);
            //
            // for word in text.split(" ") {
            //     if word.contains("@") {
            //         let username = word.replace("@","");
            //         let sender: i32 = cs.get_user_by_username(&username)?;
            //         // let sender = UserData::new_only_username(username);
            //         //
            //         // cs.raise_units(
            //         //     &sender,
            //         // )?;
            //         //
            //         // response = cs.get_unit_addition_message(&sender, &recipient, units)?;
            //     }
            // }

        }
    }

    if !response.is_empty() {
        bot.send_message(m.chat.id, response).await?;
    }

    Ok(())
}