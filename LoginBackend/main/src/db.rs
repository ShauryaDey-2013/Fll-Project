use rusqlite::*;
use serde::Deserialize;
use serde_json::*;
use std::fs;
use std::io;

const DEFAULT_ADMIN_PATH: &str = "../../../../tempData/defaultAdmin.jsonc";

#[derive(Deserialize, Debug)] // Added Debug derive for potential debugging
pub struct User {
    username: String,
    password: String,
    rank: i16,
    email: String,
}

pub fn search_user(username: &str, password: &str) -> (bool, User) {
    // Connect to the database
    let db = Connection::open("login.db")?;

    let is_valid = db.execute(
        "SELECT COUNT(*) FROM users WHERE username = ? AND password = ?",
        &[username, password],
    )? > 0;
    // Query the database for the user
    let mut stmt = db.prepare("SELECT * FROM users WHERE username = ? AND password = ?")?;
    let user = stmt.query_row(&[username, password], |row| {
        Ok(User {
            username: row.get(1)?,
            password: row.get(2)?,
            rank: row.get(3)?,
            email: row.get(4)?,
        })
    });

    match user {
        Ok(user) => Ok(Some(user)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
    (is_valid, user)
}

pub fn init_db() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the data at runtime within main()
    let default_admin_json = fs::read_to_string(DEFAULT_ADMIN_PATH)?;

    // 2. Deserialize the JSON data
    let default_admin: User = serde_json::from_str(&default_admin_json)?;

    // The temporary default_admin_json String is dropped here automatically
    // when it goes out of scope, saving memory.

    // 3. Connect to the database
    let db = Connection::open("login.db")?;

    // 4. Create the table
    db.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            rank INTEGER NOT NULL,
            email TEXT NOT NULL
        )",
        [],
    )?;

    // Optional: Insert the default admin user into the database
    db.execute(
        "INSERT INTO users (username, password, rank, email) VALUES (?1, ?2, ?3, ?4)",
        [
            &default_admin.username,
            &default_admin.password,
            &default_admin.rank.to_string(),
            &default_admin.email,
        ],
    )?;

    // The 'default_admin' variable is automatically dropped at the end of 'main', freeing its memory.

    Ok(())
}
