use crate::data_store::get_user_records;
use crate::session::Session;
use base64::{engine::general_purpose, Engine as _};
use bcrypt::verify;
use chrono::DateTime;
use chrono::{Duration, NaiveDateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use simple_crypt::decrypt;
use simple_crypt::encrypt;
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
/// Hierarchical.  Admin has all rights.  Chat can chat, NoRights....
pub enum UserRights {
    NoRights,
    Chat,
    Admin,
}

/// Get a list of usernames
pub async fn users() -> io::Result<Vec<String>> {
    Ok(get_user_records()
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
/// for them.  If they are not authorised return None.
pub async fn login(
    username: String,
    password: String,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
) -> io::Result<Option<LoginResult>> {
    // Process array of `AuthorisationRecord`
    let records: Vec<AuthorisationRecord> = get_user_records().await?;
    eprintln!("login({username}, {password}, sessions)");
    match records.iter().find(|&x| x.name == username) {
        Some(record) => {
            eprintln!("login({username}, {password}) Found");
            // TODO: Is this forced unwrap OK?  What about perverse
            // passwords?
            if verify(&password, &(record.password)).unwrap() {
                eprintln!("login({username}, {password}) Verified");
                // Successful login.
                // Initialise session and a result
                let expiry: DateTime<Utc> = Utc::now() + Duration::hours(6);
                let key = record.key.clone();
                let uuid: Uuid = record.uuid;
                let token = generate_token(&uuid, &expiry, &key);
                let level = record.level;
                sessions.lock().unwrap().insert(
                    token.clone(),
                    Session {
                        uuid: record.uuid,
                        expire: expiry,
                        token: token.clone(),
                        credit: 0.0,
                        level,
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

/// The data stored about a user.
/// The `name`, and  `password` are supplied by the user
/// The `uuid` is used to identify a user and is generated in
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthorisationRecord {
    pub name: String,
    pub password: String,
    pub uuid: Uuid,
    pub level: UserRights,
    pub credit: f64,
    pub key: Vec<u8>,
}

/// Handle tokens
pub fn generate_token(uuid: &Uuid, expiry: &DateTime<Utc>, key: &Vec<u8>) -> String {
    general_purpose::STANDARD
        .encode(encrypt(format!("{uuid}{expiry}").as_bytes(), key).expect("Encrypt a token"))
}

#[allow(dead_code)]
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
    let datetime = DateTime::<Utc>::from_naive_utc_and_offset(
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
        let sessions = Arc::new(Mutex::new(HashMap::<String, Session>::new()));
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
