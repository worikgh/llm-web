use rustyline::completion::FilenameCompleter;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::HistoryHinter;
//use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::Validator;
//use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, Event, EventHandler, KeyEvent};
use rustyline::{Completer, Helper, Hinter};
use std::borrow::Cow::{self, Borrowed, Owned};

/// `MyHelper` is copied from the examples in `RustyLine` crate
#[derive(Helper, Completer, Hinter, Validator)]
pub struct MyHelper {
    #[rustyline(Completer)]
    pub completer: FilenameCompleter,
    pub highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    pub validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    pub hinter: HistoryHinter,
    pub colored_prompt: String,
}

/// `MyHelper` is copied from the examples in `RustyLine` crate
impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}
