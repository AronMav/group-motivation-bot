use std::sync::{Arc, Mutex};
use rusqlite::{Connection, params, Result};
use teloxide::prelude::UserId;
use teloxide::types::User;

use crate::db::get_db;


#[derive(Debug)]
pub struct ChatServer {
    pub database: Arc<Mutex<Connection>>
}

#[derive(Debug, PartialEq)]
struct Data {
    first_name: String,
    last_name: String,
    username: String,
    units: f32,
}

#[derive(Debug)]
pub struct UserData {
    pub id: UserId,
    username: String,
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

    pub fn new_only_username(
        id: UserId,
        username: String,
        first_name: String,
        last_name: String) -> UserData {

        UserData {
            id,
            username,
            first_name,
            last_name,
        }
    }

}

impl ChatServer {
    pub fn new(db_path: String) -> Self {
        let conn = get_db(Some(db_path.as_str())).unwrap();

        ChatServer {
            database: Arc::new(Mutex::new(conn))
        }
    }

    pub fn raise_units(&self, user: &UserData) -> Result<()> {
        let user_id = user.id.to_string();
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            INSERT INTO users (id, units, username, first_name, last_name)
            VALUES (?, 1, ?, ?, ?)
            ON CONFLICT (id) DO
            UPDATE SET units = units + 1;")?;

        stmt.execute(params![user_id, user.username, user.first_name, user.last_name])?;

        Ok(())
    }

    pub fn add_user(&self, user: &UserData) -> Result<()> {
        let user_id = user.id.to_string();
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            INSERT INTO users (id, units, username, first_name, last_name)
            VALUES (?, 0, ?, ?, ?);")?;

        stmt.execute(params![user_id, user.username, user.first_name, user.last_name])?;

        Ok(())
    }

    pub fn get_units_by_id(&self, user_id: &String) -> Result<i32> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT units
            from users
            where id = ?;"
        )?;

        let mut units = 0;

        if stmt.exists([user_id])? {
            units = stmt.query_row([user_id], |row| Ok(row.get(0)?)).unwrap();
        }

        Ok(units)
    }

    pub fn user_exist(&self, user_id: &String) -> Result<bool> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT units
            from users
            where id = ?;"
        )?;

        Ok(stmt.exists([user_id])?)
    }

    // pub fn get_user_by_username(&self, username: &String) -> Result<UserData> {
    //     let lock = self.database.lock().unwrap();
    //     let mut stmt = lock.prepare(
    //         "SELECT
    //                 id
    //                 max(first_name) as first_name,
    //                 max(last_name) as last_name,
    //                 username,
    //                 sum(units) as units
    //             FROM users
    //             GROUP BY
    //                 id
    //             WHERE username = ?;"
    //     )?;
    //
    //     if stmt.exists([username])? {
    //         stmt.query_row([username], |row| Ok(row.get(0)?)).unwrap();
    //         user = UserData::new_only_username(username);
    //     }
    //
    //     Ok(user)
    // }

    pub fn get_top(&self) -> Result<String> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT
                first_name,
                last_name,
                username,
                sum(units) as units
                FROM users
            GROUP BY
                first_name,
                last_name,
                username
            ORDER BY
                sum(units) DESC
            LIMIT 10;")?;

        let units_iter = stmt.query_map([], |row| {
            Ok(Data { first_name: row.get(0)?, last_name: row.get(1)?, username: row.get(2)?, units: row.get(3)? })
        }).unwrap();

        let units_vec: Vec<Data> = units_iter.map(|d| { d.unwrap() }).collect();

        let mut message = String::from("⚙️ Рейтинг ⚙️\n");
        for (index, data) in units_vec.iter().enumerate() {
            if index == 0 {
                message.push_str("🥇 ");
            } else if index == 1 {
                message.push_str("🥈 ");
            } else if index == 2 {
                message.push_str("🥉 ");
            } else {
                message.push_str("       ");
            }

            message.push_str(format!("{} - {} {} (@{})\n", data.units, data.first_name, data.last_name, data.username).as_str());
        }

        Ok(message)
    }

    pub fn get_unit_addition_message(&self, sender: &UserData, recipient: &UserData, units: i32) -> Result<String> {

        let message = String::from(
            format!("{} {} (@{})\n{} {} (@{}) поблагодарил тебя\nДержи ⚙️\nТеперь у тебя их {}",
                    sender.first_name,
                    sender.last_name,
                    sender.username,
                    recipient.first_name,
                    recipient.last_name,
                    recipient.username,
                    units)
        );

        Ok(message)
    }
}
