pub mod handler;
mod chat_server;
mod db;

use std::{sync::Arc};
use std::env::var;
use teloxide::prelude::*;

use crate::{
    handler::handle,
    chat_server::ChatServer
};


#[tokio::main]
async fn main() {
    let log_path = var("LOG_PATH").unwrap();
    log4rs::init_file(log_path, Default::default()).unwrap();
    run().await;
}

async fn run() {
    log::info!("Starting group-motivation-bot");

    let bot = Bot::from_env();
    let db_path = var("DB_PATH").unwrap();
    let chat_server = Arc::new(ChatServer::new(db_path));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![chat_server])
        .build()
        .dispatch()
        .await;

    log::info!("Closing bot... Goodbye!");
}