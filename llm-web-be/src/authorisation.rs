/// Handle user authentication
/// Use a simple file with JSON
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
const FILENAME: &str = "users.txt";
#[derive(Debug, Deserialize, Serialize)]
struct AuthorisationRecord {
    claims: Claims,
    password: String,
}
use llm_web_common::{timestamp_wts, Claims};

fn get_records() -> io::Result<Vec<AuthorisationRecord>> {
    let s = read_file_with_lock(FILENAME)?;
    let records: Vec<AuthorisationRecord> = serde_json::from_str(s.as_str())?;
    Ok(records)
}
/// Check if a user is authorised with `password`.  If so return a
/// `Claims` object for them, else return None.
pub fn authorise(username: String, password: String) -> Option<Claims> {
    match read_file_with_lock(FILENAME) {
        Ok(s) => {
            // Process array of `AuthorisationRecord`
            let records: Vec<AuthorisationRecord> = match get_records() {
                Ok(r) => r,
                Err(err) => {
                    eprintln!("{err}: Failed to process JSON: {s}");
                    return None;
                }
            };
            match records.iter().find(|&x| x.claims.sub() == username) {
                Some(record) => {
                    if record.password == password {
                        Some(record.claims.clone())
                    } else {
                        None
                    }
                }
                None => None,
            }
        }
        Err(err) => {
            eprintln!("{err}: Failed to open/lock file: {FILENAME}");
            None
        }
    }
}

/// Add a user to the system
pub fn add(username: String, password: String) -> io::Result<()> {
    let mut records: Vec<AuthorisationRecord> = get_records()?;
    let auth_rec = AuthorisationRecord {
        claims: Claims::new(username, timestamp_wts() + 24 * 60 * 60),
        password,
    };
    records.push(auth_rec);
    let contents = serde_json::to_string(&records)?;
    write_file_with_lock(FILENAME, contents)?;
    Ok(())
}

fn read_file_with_lock(filename: &str) -> io::Result<String> {
    let file = File::open(filename)?;
    file.lock_exclusive()?;
    let mut lines = BufReader::new(file).lines();
    let mut contents = String::new();
    while let Some(line) = lines.next() {
        contents += line?.as_str();
    }

    Ok(contents)
}

/// `contents` is the
fn write_file_with_lock(filename: &str, contents: String) -> io::Result<()> {
    let file = OpenOptions::new()
        .truncate(true)
        .create(true)
        .open(filename)?;
    file.lock_exclusive()?;
    let mut fw = BufWriter::new(file);
    fw.write_all(contents.as_bytes())?;

    Ok(())
}
