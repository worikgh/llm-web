/// Functions to implement
// Each is `pub async` and they all lock the file for their duration
// use io::Result;
// * - add_user(username:&str, password:&str) -> Result<bool>
// * - delete_user(username: &str) -> Result<bool>
// * - get_user_records() -> Result<Vec<AuthorisationRecord>>
// * - update_user(session: &Session) -> Result<()>
use crate::authorisation::UserRights;
use crate::session::Session;
// use std::fs::File;
// use std::io;
// use std::io::SeekFrom;
use crate::authorisation::AuthorisationRecord;
// use crate::session::Session;
// use base64::{engine::general_purpose, Engine as _};
use bcrypt::{hash, DEFAULT_COST};
// use chrono::DateTime;
// use chrono::{Duration, NaiveDateTime, Utc};
use fs2::FileExt;
use rand::Rng;
// use serde::Deserialize;
// use serde::Serialize;
// use simple_crypt::decrypt;
// use simple_crypt::encrypt;
// use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
// use std::sync::{Arc, Mutex};
use uuid::Uuid;
const FILENAME: &str = "users.txt";

// pub async fn save_user(session: Session) -> io::Result<()> {
//     Ok(())
// }

fn get_locked_handle() -> io::Result<File> {
    let file = match OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(FILENAME)
    {
        Ok(f) => f,
        Err(err) => panic!("{}: Filename: {}", err, FILENAME),
    };
    file.lock_exclusive()?;
    Ok(file)
}

/// Remove a users record
/// Return
/// * `true` if record deleted
/// * `false if record not found
pub async fn delete_user(username: &str) -> io::Result<bool> {
    let username = username.to_string();
    tokio::task::spawn_blocking(move || -> io::Result<bool> {
        let mut file: File = get_locked_handle()?;
        file.seek(SeekFrom::Start(0))?;

        // Got file of data locked and ready to read
        // Read it into `contents` and transform to records
        let mut contents = String::new();
        for line in BufReader::new(&file).lines() {
            contents += line?.as_str();
        }
        let mut records: Vec<AuthorisationRecord> = if contents.is_empty() {
            // No users yet
            return Ok(false);
        } else {
            match serde_json::from_str(contents.as_str()) {
                Ok(s) => s,
                Err(err) => panic!("{}", err),
            }
        };

        // Search for user record. If it is there delete it and over
        // write the user file
        if let Some(pos) = records.iter().position(|x| x.name == username) {
            records.remove(pos);

            let contents = serde_json::to_string(&records)?;
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            let mut fw = BufWriter::new(file);
            fw.write_all(contents.as_bytes())?;
            Ok(true)
        } else {
            // Not found
            Ok(false)
        }
    })
    .await?
}

/// Add a user to the system.  Set them up with a record in the
/// Authorisation DB with: User name, password, UUID, encryption key,
/// and UserRights.  (The encryption key is used to encrypt thier
/// session tokens).  Return false if already in the system.  True
/// otherwise
pub async fn add_user(username: &str, password: &str) -> io::Result<bool> {
    eprintln!("add_user({username}, {password})");
    // No white space in passwords
    let hashed_password = hash(password.trim(), DEFAULT_COST).unwrap();
    let rng = rand::thread_rng();
    let key: Vec<u8> = rng
        .sample_iter(rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
        .as_bytes()
        .to_vec();

    let auth_rec = AuthorisationRecord {
        name: username.to_string(),
        level: UserRights::Chat,
        password: hashed_password,
        uuid: Uuid::new_v4(),
        key,
        credit: 0.0,
    };

    tokio::task::spawn_blocking(move || -> io::Result<bool> {
        let mut file = get_locked_handle()?;
        file.seek(SeekFrom::Start(0))?;
        let lines = BufReader::new(&file).lines();
        let mut contents = String::new();
        for line in lines {
            contents += line?.as_str();
        }

        let mut records: Vec<AuthorisationRecord> = if contents.is_empty() {
            // No users yet
            vec![]
        } else {
            match serde_json::from_str(contents.as_str()) {
                Ok(s) => s,
                Err(err) => panic!("{}", err),
            }
        };

        if records.iter().any(|x| x.name == auth_rec.name) {
            // Record exists
            // `false` means do not need to add user
            Ok(false)
        } else {
            records.push(auth_rec);
            let contents = serde_json::to_string(&records)?;
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            let mut fw = BufWriter::new(file);
            fw.write_all(contents.as_bytes())?;
            Ok(true)
        }
    })
    .await?
}

/// Get all the authorisation records for a read only purpose
pub async fn get_user_records() -> io::Result<Vec<AuthorisationRecord>> {
    let mut file: File = get_locked_handle()?;
    let mut s = String::new();
    file.seek(SeekFrom::Start(0))?;
    let lines = BufReader::new(file).lines();
    for line in lines {
        s += line?.as_str();
    }
    if s.is_empty() {
        return Ok(vec![]);
    }
    let records: Vec<AuthorisationRecord> = match serde_json::from_str(s.as_str()) {
        Ok(v) => v,
        Err(err) => panic!("{}: Failed to decode {}", err, s),
    };

    Ok(records)
}

/// Save the user data.  Onlythe credit in the first instance.
/// TODO: Profile this.  Saving one user flag means saving all users
/// Precondition: User identified by `session` must exist
pub async fn update_user(session: &Session) -> io::Result<()> {
    let session = session.clone();
    let uuid = session.uuid;
    tokio::task::spawn_blocking(move || -> io::Result<()> {
        let mut file = get_locked_handle()?;
        let lines = BufReader::new(&file).lines();
        let mut contents = String::new();
        for line in lines {
            contents += line?.as_str();
        }

        let mut records: Vec<AuthorisationRecord> = if contents.is_empty() {
            // No users yet
            panic!("update_user({:?}): No users", session);
        } else {
            match serde_json::from_str(contents.as_str()) {
                Ok(s) => s,
                Err(err) => panic!("{}", err),
            }
        };

        match records.iter_mut().find(|a| a.uuid == uuid) {
            Some(r) => {
                r.credit = session.credit;
                r.level = session.level;
            }
            None => panic!(
                "update_user({:?}): User does not exist in database",
                session
            ),
        };

        let contents = serde_json::to_string(&records)?;
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        let mut fw = BufWriter::new(file);
        fw.write_all(contents.as_bytes())
    })
    .await?
}
