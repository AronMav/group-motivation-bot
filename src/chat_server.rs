use std::sync::{Arc, Mutex};
use rusqlite::{Connection, params, Result};
use teloxide::prelude::UserId;
use teloxide::types::User;
use crate::db::get_db;

#[derive(Debug)]
pub struct ChatServer {
    pub database: Arc<Mutex<Connection>>,
    pub registration_key: String,
    pub bot_name: String,
    pub bot_username: String,
    pub coin: String,
    pub key_word: String,
    pub max_by_day_coins: i32
}

#[derive(Debug, PartialEq)]
struct Data {
    first_name: String,
    last_name: String,
    username: String,
    coins: f32,
}

#[derive(Debug)]
pub struct LimitationData {
    pub coins_per_day: i32,
    pub current_date: String,
}

#[derive(Debug)]
pub struct UserData {
    pub id: UserId,
    pub username: String,
    first_name: String,
    last_name: String,
}

impl UserData {
    pub fn get_new_user(user: User) -> UserData {
        UserData {
            id: user.id,
            username : user.username.unwrap_or_else(|| String::from("")),
            first_name: user.first_name,
            last_name: user.last_name.unwrap_or_else(|| String::from("")),
        }
    }
}

impl ChatServer {

    pub fn new(db_path: String,
               registration_key: String,
               bot_name: String,
               bot_username: String,
               coin: String,
               key_word: String,
               max_by_day_coins: i32) -> Self {
        let conn = get_db(Some(db_path.as_str())).unwrap();

        ChatServer {
            database: Arc::new(Mutex::new(conn)),
            registration_key,
            bot_name,
            bot_username,
            coin,
            key_word,
            max_by_day_coins
        }
    }

    pub fn get_coins_per_day(&self, username: &String) -> Result<LimitationData> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT
            coinsPerDay,
            currentDate
            FROM users
            WHERE username = ?;"
        )?;
        let limitation_data = stmt.query_row([username], |row|
            Ok(LimitationData{coins_per_day: row.get(0)?, current_date: row.get(1)?}))?;

        Ok(limitation_data)
    }

    pub fn increase_coin_count(&self, username: &String) -> Result<()> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            UPDATE users
            SET coins = coins + 1
            WHERE username = ?1;")?;

        stmt.execute(params![username])?;

        Ok(())
    }

    pub fn increase_coin_per_day_count(&self, username: &String) -> Result<()> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            UPDATE users
            SET coinsPerDay = coinsPerDay + 1
            WHERE username = ?1;")?;

        stmt.execute(params![username])?;

        Ok(())
    }

    pub fn reset_limits(&self, username: &String, current: &String) -> Result<()> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            UPDATE users
            SET coinsPerDay = 0,
            currentDate = ?2
            WHERE username = ?1;")?;

        stmt.execute(params![username, current])?;

        Ok(())
    }

    pub fn add_user(&self, user: &UserData) -> Result<()> {
        let user_id:u64 = user.id.0;
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            INSERT INTO users (id, coins, username, firstName, lastName, currentDate)
            VALUES (?1, 0, ?2, ?3, ?4, DATE('now'))
            ON CONFLICT (id) DO
            UPDATE SET username = ?2, firstName = ?3, lastName = ?4")?;

        stmt.execute(params![user_id, user.username, user.first_name, user.last_name])?;

        Ok(())
    }


    pub fn get_id_by_username(&self, username: &String) -> Result<u64> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT id
            FROM users
            WHERE username = ?;"
        )?;
        let id = stmt.query_row([username], |row| Ok(row.get(0)?))?;

        Ok(id)
    }

    pub fn user_exist(&self, username: &String) -> Result<bool> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT coins
            from users
            where username = ?;"
        )?;
        Ok(stmt.exists([username])?)
    }

    pub fn get_top(&self) -> Result<String> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT
                firstName,
                lastName,
                username,
                sum(coins) as coins
                FROM users
            GROUP BY
                firstName,
                lastName,
                username
            ORDER BY
                sum(coins) DESC
            LIMIT 10;")?;

        let coins_iter = stmt.query_map([], |row| {
            Ok(Data { first_name: row.get(0)?, last_name: row.get(1)?, username: row.get(2)?, coins: row.get(3)? })
        })?;

        let coins_vec: Vec<Data> = coins_iter.map(|d| { d.unwrap() }).collect();

        let mut message = String::from("*Рейтинг*\n");
        for (index, data) in coins_vec.iter().enumerate() {
            if index == 0 {
                message.push_str("🥇 ");
            } else if index == 1 {
                message.push_str("🥈 ");
            } else if index == 2 {
                message.push_str("🥉 ");
            } else {
                message.push_str("       ");
            }

            message.push_str(format!("{} \\- {} {} \\(@{}\\)\n", data.coins, data.first_name, data.last_name, data.username).as_str());
        }

        Ok(message)
    }
}
