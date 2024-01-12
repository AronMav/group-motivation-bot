use std::{error::Error, sync::Arc};
use teloxide::{
    prelude::*,
    types::MessageKind::Common,
    utils::command::BotCommands
};

use crate::chat_server::ChatServer;

#[derive(BotCommands, PartialEq, Debug)]
#[command(rename_rule = "lowercase")]
enum Command {
    Top,
}

pub async fn handle(
    bot: Bot,
    m: Message,
    cs: Arc<ChatServer>
)
    -> Result<(), Box<dyn Error + Send + Sync>> 
{
    let chat_id = m.chat.id.0;
    
    // Telegram uses negative numbers for groups' `chat_id`
    if chat_id > 0 {
        bot.send_message(m.chat.id, "This bot is only useful in groups.").await?;
    }

    let text =
        match m.text(){
            Some(text) => text,
            None => return Ok(())
        };

    let mut response = String::from("");

    if let Ok(command) = Command::parse(text, "Quorra") {
        response = match command {
            Command::Top => {
                let str = cs.get_top()?;
                str
            },
        }
    } else {

        let reply = m.reply_to_message();

        if reply != Option::None {

            let sender = reply.unwrap().from().unwrap().clone();
            let id_helper = &sender.id;
            let str_id_helper = &id_helper.to_string();
            match m.kind {
                Common(ref common_msg) => {
                    if let Some(user) = &common_msg.from {
                        if &user.id != id_helper && str_id_helper != &String::from("6685232640") {
                            if m.text()
                                .unwrap()
                                .to_lowercase()
                                .contains("спасибо") {

                                let sender_username = &sender.username.unwrap_or_else(|| String::from(""));
                                let sender_first_name= &sender.first_name;
                                let sender_last_name = &sender.last_name.unwrap_or_else(|| String::from(""));

                                cs.store_units(
                                    str_id_helper,
                                    sender_username,
                                    sender_first_name,
                                    sender_last_name,
                                )?;
                                let recipient = m.from().unwrap().clone();
                                let recipient_username = &recipient.username.unwrap_or_else(|| String::from(""));
                                let recipient_first_name= &recipient.first_name;
                                let recipient_last_name = &recipient.last_name.unwrap_or_else(|| String::from(""));
                                let units: i32 = cs.get_units(str_id_helper)?;
                                response = String::from(
                                    format!("{} {} (@{}) ➡️ {} {} (@{})\nРепутация повышена: {}",
                                        &recipient_first_name,
                                        &recipient_last_name,
                                        &recipient_username,
                                        &sender_first_name,
                                        &sender_last_name,
                                        &sender_username,
                                        units)
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if !response.is_empty() {
        bot.send_message(m.chat.id, response).await?;
    }

    Ok(())
}