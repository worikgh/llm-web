// use std::{error::Error, fmt};

// #[derive(Debug)]
// pub enum CliError {
//     Error(String),
// }
// impl CliError {}
// impl fmt::Display for CliError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             CliError::Error(ref msg) => write!(f, "Error: {msg}"),
//         }
//     }
// }

// impl Error for CliError {
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         None
//     }
// }
