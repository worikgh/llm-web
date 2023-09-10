use crate::session::Session;
use base64::{engine::general_purpose, Engine as _};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::DateTime;
use chrono::{Duration, NaiveDateTime, Utc};
use fs2::FileExt;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use simple_crypt::decrypt;
use simple_crypt::encrypt;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
// use std::time::{SystemTime, UNIX_EPOCH};
const FILENAME: &str = "users.txt";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum UserRights {
    NoRights,
    Chat,
    Admin,
}

/// Get a list of usernames
pub async fn users() -> io::Result<Vec<String>> {
    Ok(get_records()
        .await?
        .iter()
        .map(|x| x.name.clone())
        .collect())
}

/// Returned to caller on successful login
#[derive(Debug)]
pub struct LoginResult {
    pub rights: UserRights,
    pub uuid: Uuid,
    pub token: String, // Send this back to user.  It must be sent with every request
}

/// Check if a user is authorised with `password`.  If so create an
/// entry in the session database and return a `LoginResult` object
/// for them.  If they are ot authorised return None.
pub async fn login(
    username: String,
    password: String,
    sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
) -> io::Result<Option<LoginResult>> {
    // Process array of `AuthorisationRecord`
    let records: Vec<AuthorisationRecord> = get_records().await?;
    eprintln!("login({username}, {password}, sessions)");
    match records.iter().find(|&x| x.name == username) {
        Some(record) => {
            eprintln!("login({username}, {password}) Found");
            if verify(&password, &(record.password)).unwrap() {
                eprintln!("login({username}, {password}) Verified");
                // Successful login.
                // Initialise session and a result
                let expiry: DateTime<Utc> = Utc::now() + Duration::hours(6);
                let key = record.key.clone();
                let uuid: Uuid = record.uuid;
                let token = generate_token(&uuid, &expiry, &key);
                sessions.lock().unwrap().insert(
                    record.uuid,
                    Session {
                        uuid: record.uuid,
                        expire: expiry,
                        token: token.clone(),
                    },
                );
                Ok(Some(LoginResult {
                    rights: record.level.clone(),
                    uuid,
                    token,
                }))
            } else {
                // Failed login.  Not an error
                eprintln!(
                    "login({username}, {password}) Failed verify: {} ",
                    record.password
                );
                Ok(None)
            }
        }
        None => {
            eprintln!("login({username}, {password}) Not Found");
            Ok(None)
        }
    }
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
    };

    tokio::task::spawn_blocking(move || -> io::Result<bool> {
        let mut file = match OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(FILENAME)
        {
            Ok(f) => f,
            Err(err) => panic!("{}: Filename: {}", err, FILENAME),
        };
        file.lock_exclusive()?;
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
    //    _add_user(auth_rec).await
}

/// Remove a users record
/// Return
/// * `true` if record deleted
/// * `false if record not found
pub async fn delete_user(username: &str) -> io::Result<bool> {
    let username = username.to_string();
    tokio::task::spawn_blocking(move || -> io::Result<bool> {
        let mut file = match OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(FILENAME)
        {
            Ok(f) => f,
            Err(err) => panic!("{}: Filename: {}", err, FILENAME),
        };
        file.lock_exclusive()?;
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

/// Get all the authorisation records
async fn get_records() -> io::Result<Vec<AuthorisationRecord>> {
    let s = tokio::task::spawn_blocking(move || -> io::Result<String> {
        let file = File::open(FILENAME)?;
        file.lock_exclusive()?;
        let lines = BufReader::new(file).lines();
        let mut contents = String::new();
        //while let Some(line) = lines.next() {
        for line in lines {
            contents += line?.as_str();
        }
        Ok(contents)
    })
    .await??;

    if s.is_empty() {
        return Ok(vec![]);
    }
    let records: Vec<AuthorisationRecord> = match serde_json::from_str(s.as_str()) {
        Ok(v) => v,
        Err(err) => panic!("{}: Failed to decode {}", err, s),
    };

    Ok(records)
}

/// The data stored about a user.
/// The `name`, and  `password` are supplied by the user
/// The `uuid` is used to identify a user and is generated in
#[derive(Clone, Debug, Deserialize, Serialize)]
struct AuthorisationRecord {
    name: String,
    password: String,
    uuid: Uuid,
    level: UserRights,
    key: Vec<u8>,
}

/// Handle tokens
pub fn generate_token(uuid: &Uuid, expiry: &DateTime<Utc>, key: &Vec<u8>) -> String {
    general_purpose::STANDARD
        .encode(encrypt(format!("{uuid}{expiry}").as_bytes(), key).expect("Encrypt a token"))
}

pub fn decode_token(
    encoded_uuid_expiry: String,
    key: &Vec<u8>,
) -> Result<(Uuid, DateTime<Utc>), Box<dyn std::error::Error>> {
    let decoded_data = general_purpose::STANDARD.decode(encoded_uuid_expiry)?;
    let decrypted_data = decrypt(&decoded_data, key)?;

    let decrypted_string = String::from_utf8(decrypted_data)?;

    let parts: (&str, &str) = decrypted_string.split_at(36);
    let uuid_part = parts.0;
    let datetime_part = parts.1;

    let uuid = Uuid::parse_str(uuid_part)?;
    let datetime = DateTime::<Utc>::from_utc(
        NaiveDateTime::parse_from_str(datetime_part, "%Y-%m-%d %H:%M:%S%.f %Z")?,
        Utc,
    );

    Ok((uuid, datetime))
}

#[cfg(test)]
pub mod tests {
    //use llm_web_common::communication::LoginRequest;

    use super::*;

    pub async fn get_unique_user(pfx: &str) -> String {
        let user_list = users().await.unwrap();
        let mut name_pfx = pfx.to_string();
        let letters: Vec<char> = (b'a'..=b'z').map(char::from).collect();
        let mut itr = letters.iter().peekable();
        let mut itr2 = letters.iter();
        let mut test_name: String;
        loop {
            test_name = format!("{name_pfx}_{}", itr.next().unwrap());
            if !user_list.contains(&test_name) {
                break;
            }
            if itr.peek().is_none() {
                name_pfx = format!("{name_pfx}{}_", itr2.next().unwrap());
            }
        }
        test_name
    }
    #[tokio::test]
    async fn test_login() {
        let username = get_unique_user("authorisation::tests::test_login").await;
        let password = "123";
        let b: bool = add_user(username.as_str(), "123").await.unwrap();
        eprintln!("`add_user` returns true{b}");
        assert!(b);

        // Test logging the user in
        let sessions = Arc::new(Mutex::new(HashMap::<Uuid, Session>::new()));
        let test: bool = match login(username.clone(), password.to_string(), sessions).await {
            Ok(t) => t.is_some(),
            Err(err) => panic!("{}", err),
        };
        // Test can log user in
        assert!(test);
        assert!(delete_user(username.as_str()).await.unwrap());
    }
    #[tokio::test]
    async fn add_delete_user() {
        let test_name = get_unique_user("authorisation::tests::add_delete_user").await;
        let b = add_user(test_name.as_str(), "123").await.unwrap();
        eprintln!("Adding a user returns true: {b}");
        assert!(b);
        let b = add_user(test_name.as_str(), "123").await.unwrap();
        eprintln!("Repeating adding a user returns false: {b}");
        assert!(!b);

        let b = delete_user(test_name.as_str()).await.unwrap();
        eprintln!("Deleted record was found: {b}");
        assert!(b);

        let b = delete_user(test_name.as_str()).await.unwrap();
        eprintln!("Repeating delete record returns false: {b}");
        assert!(!b);
    }
    #[test]
    fn token_coding() {
        let uuid = Uuid::new_v4();
        let expiry: DateTime<Utc> = Utc::now() + Duration::hours(6);
        let key: Vec<u8> = vec![1, 2, 3, 4];
        let token = generate_token(&uuid, &expiry, &key);
        println!("uuid:{uuid} expiry:{expiry}");

        match NaiveDateTime::parse_from_str(
            "2023-09-10 07:31:29.249939359 UTC",
            "%Y-%m-%d %H:%M:%S%.f %Z",
        ) {
            Ok(_) => (),
            Err(err) => panic!("Failed time: {}", err),
        };

        let (uuid_test, expiry_test) = match decode_token(token, &key) {
            Ok(a) => a,
            Err(err) => panic!("{}", err),
        };
        assert!(uuid == uuid_test);
        assert!(expiry == expiry_test);
    }
}
