use helpers::my_helper::MyHelper;
use rustyline::completion::FilenameCompleter;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, Event, EventHandler, KeyEvent};
use std::str::FromStr;
extern crate llm_rs;
mod helpers {
    pub mod my_helper;
}

extern crate tempfile;
use clap::Parser;
use llm_rs::openai_interface;

use std::env;
use std::env::current_dir;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf; //::{Editor};
                        // mod json;

use openai_interface::ApiInterface;
use openai_interface::ModelMode;

const DEFAULT_MODEL: &str = "text-davinci-003";
const DEFAULT_TOKENS: u32 = 2_000_u32;
const DEFAULT_TEMPERATURE: f32 = 0.9_f32;
const DEFAULT_MODE: &str = "completions";
const DEFAULT_RECORD_FILE: &str = "reply.txt";

/// Command line argument definitions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// The model to use
    #[arg(long, short = 'm',default_value=DEFAULT_MODEL)]
    model: String,

    /// Maximum tokens to return
    #[arg(long, short = 't', default_value_t=DEFAULT_TOKENS)]
    max_tokens: u32,

    /// Temperature for the model.
    #[arg(long, short = 'T', default_value_t = DEFAULT_TEMPERATURE)]
    temperature: f32,

    /// The secret key.  [Default: environment variable `OPENAI_API_KEY`]
    #[arg(long)]
    api_key: Option<String>,

    /// The initial mode (API endpoint)
    #[arg(long, short='d', default_value=DEFAULT_MODE)]
    mode: String,

    /// The file name that prompts and replies are recorded in
    #[arg(long, short='r', default_value=DEFAULT_RECORD_FILE)]
    record_file: String,

    /// The system prompt sent to the chat model
    #[arg(long, short='p', default_value=None)]
    system_prompt: Option<String>,
}

/// This function was written by Chat-GPT using
/// text-davinci-003. Justifies the output so no line is longer than
/// 80 characters by splitting lines on word breaks
fn justify_string(s: &str) -> String {
    let mut result = String::new();
    let mut line_length = 0;

    for word in s.split_whitespace() {
        //    while let Some(word) = words.next() {
        let word_length = word.len();

        if line_length + word_length + 1 > 80 {
            result.push('\n');
            line_length = 0;
        } else if line_length > 0 {
            result.push(' ');
            line_length += 1;
        }

        result.push_str(word);
        line_length += word_length;
    }

    result
}

fn set_up_read_line() -> rustyline::Result<Editor<MyHelper, FileHistory>> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let h = MyHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };
    let mut read_line = Editor::with_config(config)?;
    read_line.set_helper(Some(h));
    read_line.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    read_line.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
    if read_line.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    // Set control key C-q to quit.  Not really needed.  C-c does this
    // auto-magically
    read_line.bind_sequence(
        Event::KeySeq(vec![KeyEvent::ctrl('q')]),
        EventHandler::Simple(Cmd::Interrupt),
    );
    Ok(read_line)
}

/// Process prompts that are to effect or inspect the programme itself
/// `prommpt` is what the user entered after the initial "!"
fn process_meta(prompt: &str, api_interface: &mut ApiInterface) -> rustyline::Result<String> {
    let mut meta = prompt.split_whitespace();
    // The first word is: "!"
    // The rest of the words are commands for the programme to interpret.

    let response_text: String;
    if let Some(cmd) = meta.nth(1) {
        // Handle commands here
        match cmd {
            "p" => {
                if api_interface.verbose > 1 {
                    response_text = format!("{:?}", api_interface);
                } else {
                    // Display the parameters
                    response_text = format!(
                        "Mode: {:?}\n\
		     Temperature: {}\n\
		     Model: {}\n\
		     Tokens: {}\n\
		     Verbosity: {}\n\
		     Context length: {}\n\
		     System prompt: {}\n\
		     Image focus URI Set: {}\n\
		     Mask: {:?}\n",
                        api_interface.model_mode,
                        api_interface.temperature,
                        api_interface.model,
                        api_interface.tokens,
                        api_interface.verbose,
                        api_interface.context.len(),
                        api_interface.system_prompt,
                        api_interface.focus_image_url.is_some(),
                        match &api_interface.mask {
                            Some(pb) => pb.display().to_string(),
                            None => "<None>".to_string(),
                        },
                    );
                }
            }
            "md" => {
                // Display known models
                let mut model_list: Vec<String> = api_interface.model_list().unwrap();
                model_list.sort();
                response_text = model_list
                    .iter()
                    .fold(String::new(), |a, b| format!("{a}{b}\n"));
            }
            "ms" => {
                // Set a model
                if let Some(model_name) = meta.next() {
                    response_text = format!("New model: {model_name}");
                    api_interface.model = model_name.to_string();
                } else {
                    response_text = "No model".to_string();
                }
            }
            "m" => {
                // Set the mode (effectively the API endpoint at OpenAI
                match meta.next() {
                    // "! m" on its own to get a list of models
                    // "! m <model name>" to change it
                    Some(mode) => match mode {
                        "completions" => {
                            response_text = "Model mode => Completions\n".to_string();
                            api_interface.model_mode = ModelMode::Completions;
                        }
                        "chat" => {
                            // A conversation with the LLM. `system_prompt` sets
                            // the tone of the conversation.  It can be over
                            // ridden here, and there must be some prompt
                            let system_prompt = meta.collect::<Vec<&str>>().join(" ");
                            if system_prompt.is_empty() && api_interface.system_prompt.is_empty() {
                                response_text = "Provide a system prompt for the chat".to_string();
                            } else {
                                api_interface.model_mode = ModelMode::Chat;
                                response_text = "Model mode => Chat\n".to_string();
                                if !system_prompt.is_empty() {
                                    api_interface.system_prompt = system_prompt;
                                }
                            }
                        }
                        "image" => {
                            // Create images from prompts.  If a file is passed in
                            // it is an image to edit, so the mode is set to
                            // `ImageEdit`
                            let file_name: String = meta.collect::<Vec<&str>>().join(" ");
                            if file_name.is_empty() {
                                // User is going to get AI to generate the image
                                api_interface.model_mode = ModelMode::Image;
                                response_text = "Model mode => Image\n".to_string();
                            } else {
                                // User is supplying an image
                                if PathBuf::from(file_name.as_str()).exists() {
                                    api_interface.image = Some(PathBuf::from(file_name));
                                    api_interface.model_mode = ModelMode::ImageEdit;
                                    response_text = "Model mode => ImageEdit\n".to_string();
                                } else {
                                    api_interface.model_mode = ModelMode::Image;
                                    response_text =
                                        "File: {file_name} does not exist.  Model mode => Image\n"
                                            .to_string();
                                }
                            }
                        }
                        "image_edit" => {
                            // Edit an image.
                            match api_interface.model_mode {
                                ModelMode::Image => {
                                    if api_interface.image.is_none()
                                        && api_interface.focus_image_url.is_none()
                                    {
                                        response_text = format!(
                                            "Cannot switch to ImageEdit mode \
					     from {} mode untill you have created \
					     an image.  Enter a prompt",
                                            api_interface.model_mode
                                        );
                                    } else {
                                        response_text = "Edit image".to_string();
                                        api_interface.model_mode = ModelMode::ImageEdit;
                                    }
                                }
                                _ => {
                                    response_text = format!("Cannot switch to ImageEdit mode from {} mode.  Must be in Image mode", api_interface.model_mode);
                                }
                            };
                        }
                        _ => response_text = format!("{mode} not a Model Mode\n"),
                    },
                    None => {
                        response_text = "Model modes\n\
					 completions\n\
					 chat\n\
					 image\n\
					 image_edit\n"
                            .to_string()
                    }
                }
            }
            "cd" => {
                response_text = api_interface.context.join("\n");
            }
            "cc" => {
                response_text = "Clear context".to_string();
                api_interface.clear_context();
            }
            "v" => {
                // set verbosity
                if let Some(v) = meta.next() {
                    response_text = match v.parse::<usize>() {
                        Ok(v) => {
                            api_interface.verbose = v;
                            format!("Verbosity set to {v}\n")
                        }
                        Err(err) => format!("Cannot make a usize from {v} because: {err}\n"),
                    }
                } else {
                    response_text = "No verbosity level passed".to_string();
                }
            }
            "k" => {
                // set tokens
                if let Some(t) = meta.next() {
                    response_text = match t.parse::<u32>() {
                        Ok(t) => {
                            api_interface.tokens = t;
                            format!("New tokens: {t}\n")
                        }
                        Err(err) => format!("Cannot make a float from {t} because: {err}\n"),
                    };
                } else {
                    response_text = "No tokens".to_string();
                }
            }
            "t" => {
                // set temperature
                if let Some(t) = meta.next() {
                    response_text = match t.parse::<f32>() {
                        Ok(t) => {
                            if (0.0_..=2.0).contains(&t) {
                                api_interface.temperature = t;
                                format!("New temperature: {t}\n")
                            } else {
                                "A float between 0 and 2\n".to_string()
                            }
                        }
                        Err(err) => format!("Cannot make a float from {t} because: {err}\n"),
                    }
                } else {
                    response_text = "No temperature".to_string();
                }
            }
            "sp" => {
                if api_interface.model_mode != ModelMode::Chat {
                    response_text = "This only makes sense in Chat mode".to_string();
                } else {
                    let system_prompt = meta.collect::<Vec<&str>>().join(" ");
                    if system_prompt.is_empty() {
                        if api_interface.system_prompt.is_empty() {
                            response_text = "Provide a system prompt for the chat".to_string();
                        } else {
                            response_text =
                                format!("System Prompt {}", api_interface.system_prompt);
                        }
                    } else {
                        response_text = format!("System Prompt {system_prompt}");
                        api_interface.system_prompt = system_prompt;
                    }
                }
            }
            "ci" => {
                // Clear `api_imterface.image` and api_interface.miage_focus_url`
                api_interface.image = None;
                api_interface.focus_image_url = None;

                // If mode is ImageEdit set it to Image
                if api_interface.model_mode == ModelMode::ImageEdit {
                    //		self.api
                    response_text = format!("Image cleared. Mode: {}", api_interface.model_mode);
                } else {
                    response_text = "Image cleared".to_string();
                }
            }
            "mask" => {
                // Set a mask
                let file_name: String = meta.collect::<Vec<&str>>().join(" ");
                if file_name.is_empty() {
                    response_text = format!(
                        "Enter the mask file path relative to: {}",
                        current_dir()?.display()
                    );
                } else if PathBuf::from(file_name.as_str()).exists() {
                    api_interface.mask = Some(PathBuf::from(file_name));
                    response_text =
                        format!("Mask set to: {:?}", api_interface.mask.clone().unwrap());
                } else {
                    response_text = format!(
                        "{file_name} dose not exist.  Paths relative to {}",
                        current_dir()?.display()
                    );
                }
            }
            "?" => {
                response_text = "\
		p  Display settings\n\
		md Display all available models\n\
		ms <model> Change the current model\n\
		m  <mode> Change mode (API endpoint\n\
		cd Display context (for chat)\n\
		cc Clear context\n\
		v  Set verbosity\n\
		k  Set max tokens for completions\n\
		t  Set temperature for completions\n\
		sp Set system prompt (after `! cc`\n\
		mask <path> Set the mask to use in image edit mode.  A 1024x1024 PNG with transparent mask\n\
		ci CLear the image stored for editing
 		?  This text\n"
                    .to_string()
            }
            _ => response_text = format!("Unknown command: {cmd}\n"),
        };
    } else {
        response_text = format!("Prompt: {prompt} Not understood\n");
    }
    Ok(response_text)
}

fn main() -> rustyline::Result<()> {
    // Get the command line options
    let cmd_line_opts = Arguments::parse();

    // API key
    let _key_binding: String;
    let api_key = match cmd_line_opts.api_key.as_deref() {
        Some(key) => key,
        None => {
            _key_binding = env::var("OPENAI_API_KEY").unwrap();
            _key_binding.as_str()
        }
    };

    // The model
    let model = cmd_line_opts.model.as_str();

    // Maximum tokens
    let tokens: u32 = cmd_line_opts.max_tokens;

    let temperature: f32 = cmd_line_opts.temperature;

    // The mode
    let mode: ModelMode = match ModelMode::from_str(cmd_line_opts.mode.as_str()) {
        Ok(m) => m,
        Err(_) => panic!("{} is an invalid mode", cmd_line_opts.mode.as_str()),
    };

    // Keep  record of the conversations in a file called "reply.txt"
    let mut options = OpenOptions::new();
    let mut conversation_record_file: File = options
        .write(true)
        .append(true)
        .create(true)
        .open(cmd_line_opts.record_file.as_str())
        .unwrap();
    let mut read_line: Editor<MyHelper, FileHistory> = set_up_read_line()?;
    let mut prompt: String;
    let mut api_interface = ApiInterface::new(api_key, tokens, temperature, model, mode);
    if let Some(sp) = cmd_line_opts.system_prompt {
        api_interface.system_prompt = sp;
    }
    let mut count = 1;
    loop {
        // Read the input text
        let p = format!("{count}> ");
        read_line.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{p}\x1b[0m");
        let readline = read_line.readline(&p);
        let input = match readline {
            Ok(line) => line,
            Err(err) => {
                // Exit from loop
                eprintln!("Readline {err}");
                break;
            }
        };
        read_line.add_history_entry(input.as_str())?;
        count += 1;

        prompt = input.clone();
        _ = conversation_record_file
            .write(format!("Q: {}\n", prompt).as_bytes())
            .unwrap();

        // The response that will be displayed to the user.
        // It can be from more than one source:
        // * It can be a response from the LLM
        // * It can be information about the state of this programme
        // * It can be the completion message (error or success) for
        //   some change to the state of this programme
        let response_text: String;
        let prompt = prompt.as_str().trim();
        if prompt.is_empty() {
            response_text = "No prompt\n".to_string();
        } else if prompt.starts_with('!') {
            response_text = process_meta(prompt, &mut api_interface)?;
        } else {
            // Send the prompt to the LLM
            response_text = match api_interface.send_prompt(prompt) {
                Ok(response) => response,
                Err(err) => {
                    // There is a bug in `api_interface` or `reqwest;`
                    format!("Failed call to send_prompt: {err}")
                }
            };
        }

        _ = conversation_record_file
            .write(format!("A: {response_text}\n").as_bytes())
            .unwrap();
        println!(
            "{}",
            response_text
                .split_terminator('\n')
                .fold(String::new(), |a, b| format!("{a}{}\n", justify_string(b)))
        );
    }

    read_line.append_history("history.txt").unwrap();
    read_line.clear_history().unwrap();
    Ok(())
}
