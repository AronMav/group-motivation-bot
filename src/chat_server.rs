use std::sync::{Arc, Mutex};
use rusqlite::{Connection, params, Result};

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

impl ChatServer {
    pub fn new(db_path: String) -> Self {
        let conn = get_db(Some(db_path.as_str())).unwrap();

        ChatServer {
            database: Arc::new(Mutex::new(conn))
        }
    }

    pub fn store_units(&self,
                       user_id: &str,
                       username: &String,
                       first_name: &String,
                       last_name: &String,
    ) -> Result<()> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare("
            INSERT INTO users (id, units, username, first_name, last_name)
            VALUES (?, 1, ?, ?, ?)
            ON CONFLICT (id) DO
            UPDATE SET units = units + 1;")?;

        stmt.execute(params![user_id, username, first_name, last_name])?;

        Ok(())
    }

    pub fn get_units(&self, user_id: &String) -> Result<i32> {
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
                sum(units) DESC;")?;

        let percents_iter = stmt.query_map([], |row| {
            Ok(Data { first_name: row.get(0)?, last_name: row.get(1)?, username: row.get(2)?, units: row.get(3)? })
        }).unwrap();

        let perc_vec: Vec<Data> = percents_iter.map(|d| { d.unwrap() }).collect();

        let mut message = String::from("");
        for (index, data) in perc_vec.iter().enumerate() {
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
}
