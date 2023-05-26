use directories::{BaseDirs, ProjectDirs};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{create_dir_all, OpenOptions};

use std::io::{Read, Write};
use std::path::PathBuf;

// Your shared state needs to be serializable and deserializable
#[derive(Serialize, Deserialize)]
pub struct SharedState {
    // Your data fields
    /// Chat interactions have a cost.  Keep the total here.  In cents.
    pub spent: f64,
}

impl SharedState {
    pub fn read_write_atomic(
        mut f: impl FnMut(SharedState) -> SharedState,
    ) -> Result<SharedState, Box<dyn std::error::Error>> {
        let file_path = Self::get_file_path();
        // Create all intermediate directories if they don't exist
        if let Some(parent_dir) = std::path::Path::new(&file_path).parent() {
            create_dir_all(parent_dir)?;
        }
        let mut file = match OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(file_path)
        {
            Ok(f) => f,
            Err(err) => return Err(Box::new(err)),
        };
        file.lock_exclusive()?;

        let mut contents = String::new();
        let sz = match file.read_to_string(&mut contents) {
            Ok(s) => s,
            Err(err) => {
                println!("Got error: {:?}", err);
                return Err(Box::new(err));
            }
        };
        println!("Read {sz} bytes");
        let state: SharedState = if sz != 0 {
            // There was content to read
            serde_json::from_str(&contents)?
        } else {
            // No content.  Initialise it
            SharedState { spent: 0.0 }
        };
        let state: SharedState = f(state);
        file.set_len(0)?;
        let contents = serde_json::to_string(&state)?;
        file.write_all(contents.as_bytes())?;
        file.unlock()?;

        Ok(state)
    }

    fn get_file_path() -> PathBuf {
        let project_dirs = ProjectDirs::from("org", "worik", "root");
        let base_dirs = BaseDirs::new();

        let writable_directory_path = match project_dirs {
            Some(dirs) => dirs.data_local_dir().to_owned(),
            None => base_dirs.unwrap().data_local_dir().to_owned(),
        };

        let mut state_file_path = writable_directory_path;
        state_file_path.push("shared_state.json");

        state_file_path
    }
}
