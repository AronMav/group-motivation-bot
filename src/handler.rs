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
}

pub async fn handle(
    bot: Bot,
    m: Message,
    cs: Arc<ChatServer>
) -> Result<(), Box<dyn Error + Send + Sync>>
{
    let chat_id = m.chat.id.0;
    
    // Telegram uses negative numbers for groups' `chat_id`
    if chat_id > 0 {
        bot.send_message(m.chat.id, "Этот бот используется только в группах.").await?;
    }

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
        }
    } else {

        let reply = m.reply_to_message();

        if reply != Option::None {

            let user = reply.unwrap().from().unwrap().clone();
            let sender = UserData::new(user);

            let str_sender_id = &sender.id.to_string();
            match m.kind {
                Common(ref common_msg) => {
                    if let Some(user) = &common_msg.from {
                        if &user.id != &sender.id && str_sender_id != &var("BOT_ID")? {
                            if m.text()
                                .unwrap()
                                .to_lowercase()
                                .contains(var("KEY_WORD")?.as_str()) {

                                cs.store_units(
                                    &sender,
                                )?;

                                let user = m.from().unwrap().clone();
                                let recipient = UserData::new(user);

                                let units: i32 = cs.get_units(str_sender_id)?;
                                response = cs.get_unit_addition_message(&sender, &recipient, units)?;
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