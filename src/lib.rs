//! This is a simple library for implementing command-line-style debug consoles within an application.
//!
//! It doesn't handle rendering, or the logic of any individual commands. All it does is model the
//! state of the console, including command history, and whether or not it is active. You define
//! your own command struct/enum, and drive the inputs to the console. After that, you can use the
//! `confirm` method to parse the text entered so far into a command.
//!
//! When running in debug mode (when the `debug_assertions` flag is set) the entire Console is
//! disabled. This is to avoid shipping something and accidentally including the debug console.
//!
//! To override this, you can use the `force-enabled` feature, which will allow the console to
//! work in release mode.
//!
//! # Examples
//!
//! ## Input
//!
//! ```rust
//! use std::str::FromStr;
//! use dbgcmd::Console;
//!
//! #[derive(Debug, PartialEq, Eq)]
//! enum Command {
//!     SayHi,
//!     SayBye,
//! }
//!
//! impl FromStr for Command {
//!     type Err = ();
//!
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         match s {
//!             "hi" => Ok(Command::SayHi),
//!             "bye" => Ok(Command::SayBye),
//!             _ => Err(())
//!         }
//!     }
//! }
//!
//! let mut console = Console::new();
//!
//! // Your application passes along key inputs
//! console.receive_char('h');
//! console.receive_char('i');
//!
//! if console.enabled() {
//!     assert_eq!(console.confirm(), Ok(Command::SayHi));
//! }
//! ```
//!
//! ## History
//!
//! ```rust
//! use dbgcmd::Console;
//!
//! type Command = String;
//!
//! let mut console = Console::new();
//!
//! // Set the entire entry instead of individual characters
//! console.set_entry("first".to_owned());
//! console.confirm::<Command>();
//!
//! console.set_entry("second".to_owned());
//! console.confirm::<Command>();
//!
//! console.set_entry("third".to_owned());
//! console.confirm::<Command>();
//!
//! if console.enabled() {
//!     // Scroll up or down the history
//!     console.up();
//!     assert_eq!(console.entry(), "third".to_owned());
//!
//!     console.up();
//!     assert_eq!(console.entry(), "second".to_owned());
//!
//!     console.up();
//!     assert_eq!(console.entry(), "first".to_owned());
//!
//!     console.down();
//!     console.down();
//!     console.down();
//!
//!     assert!(console.entry().is_empty());
//! }
//! ```
#[cfg(any(debug_assertions, feature = "force-enabled"))]
use std::collections::VecDeque;

#[cfg(any(debug_assertions, feature = "force-enabled"))]
use itertools::Itertools;

#[derive(Default, Clone, PartialEq, Eq)]
#[cfg(any(debug_assertions, feature = "force-enabled"))]
pub struct Console {
    shown: bool,
    entry: String,
    history: VecDeque<String>,
    cursor: Option<usize>,
}

#[derive(Default, Clone, PartialEq, Eq)]
#[cfg(not(any(debug_assertions, feature = "force-enabled")))]
pub struct Console;

impl Console {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(any(debug_assertions, feature = "force-enabled"))]
impl Console {
    /// Whether the console is enabled. This will be `false` in release
    /// mode unless the `force-enabled` feature is switched on.
    ///
    /// When this is `false`, all methods of the `Console` struct will
    /// do nothing and return only default values.
    pub const fn enabled(&self) -> bool {
        true
    }

    /// Tries to parse the text entered so far as the given type, and clear the entry.
    ///
    /// This uses the `FromStr` trait to parse the entry. You should implement this
    /// trait on the type you're using for your commands.
    pub fn confirm<Cmd: std::str::FromStr>(&mut self) -> Result<Cmd, Cmd::Err> {
        let entry = self.entry();
        let result = entry.parse();

        self.history
            .push_front(std::mem::replace(&mut self.entry, String::new()));
        self.cursor = None;

        result
    }

    /// Returns a reference to the text entered so far.
    pub fn entry(&self) -> &str {
        match self.cursor {
            Some(n) => &self.history[n],
            None => &self.entry,
        }
    }

    /// Returns an iterator over the previously confirmed entries. This yields
    /// items in the order from most recent to least recent.
    pub fn history(&self) -> impl Iterator<Item = &str> {
        self.history.iter().map(String::as_ref)
    }

    /// The same as `history`, but will not yield the same entry twice in a row.
    pub fn history_deduped(&self) -> impl Iterator<Item = &str> {
        self.history.iter().map(String::as_ref).dedup()
    }

    /// Returns the number of items in the history. Note that this includes
    /// duplicate entries.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Clears and sets the value of the entire command entry.
    pub fn set_entry(&mut self, entry: String) {
        self.entry = entry;
        self.cursor = None;
    }

    /// Receive an individual character and append it to the command entry.
    pub fn receive_char(&mut self, ch: char) {
        if self.cursor.is_some() {
            self.entry = self.entry().to_owned();
            self.cursor = None;
        }
        self.entry.push(ch)
    }

    /// Receive text and append it to the command entry.
    pub fn receive_text(&mut self, text: &str) {
        if self.cursor.is_some() {
            self.entry = self.entry().to_owned();
            self.cursor = None;
        }
        self.entry.push_str(text)
    }

    /// Receive an individual character and append it to the command entry
    /// if the `filter` argument returns true for it.
    ///
    /// Useful for limiting which characters can be entered.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dbgcmd::Console;
    /// let mut console = dbgcmd::Console::new();
    ///
    /// let ch = '?';
    /// let accepted = console.receive_char_if(ch, char::is_alphanumeric);
    ///
    /// assert!(!accepted);
    /// assert!(console.entry().is_empty());
    /// ```
    pub fn receive_char_if<F: Fn(char) -> bool>(&mut self, ch: char, filter: F) -> bool {
        let accept = filter(ch);
        if accept {
            self.receive_char(ch);
        }
        accept
    }

    /// Receive text and append it to the command entry
    /// if the `filter` argument returns true for it.
    pub fn receive_text_if<F: Fn(&str) -> bool>(&mut self, text: &str, filter: F) -> bool {
        let accept = filter(text);
        if accept {
            self.receive_text(text);
        }
        accept
    }

    /// Removes the last character of the command entry.
    pub fn backspace(&mut self) {
        if self.cursor.is_some() {
            self.entry = self.entry().to_owned();
            self.cursor = None;
        }
        self.entry.pop();
    }

    /// Clears the command entry.
    pub fn clear(&mut self) {
        self.entry.clear();
    }

    /// Clears the entire command history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Cycles through the command history towards older entries.
    ///
    /// Returns `true` if there was an older entry, and `false` if not. If there
    /// is no older entry, the cursor does not move.
    pub fn up(&mut self) -> bool {
        let (cursor, moved) = match self.cursor {
            None => (Some(0), true),
            Some(n) if n < (self.history.len() - 1) => (Some(n + 1), true),
            same => (same, false),
        };
        self.cursor = cursor;
        moved
    }

    /// Cycles through the command history towards older entries, ignoring duplicates.
    ///
    /// Returns `true` if there was an older entry that differs from the current entry.
    pub fn up_deduped(&mut self) -> bool {
        let starting_entry = self.entry().to_owned();
        while self.up() && self.entry() == starting_entry {}
        starting_entry != self.entry()
    }

    /// Cycles through the command history towards newer entries.
    ///
    /// Returns `true` if there was a newer entry, including the current input.
    /// Returns `false` if there was no newer entry.
    pub fn down(&mut self) -> bool {
        let (cursor, moved) = match self.cursor {
            Some(n) if n > 0 => (Some(n - 1), true),
            prev => (None, prev.is_some()),
        };
        self.cursor = cursor;
        moved
    }

    /// Cycles through the command history towards newer entries, ignoring duplicates.
    ///
    /// Returns `true` if there was a newer entry that differs from the current entry.
    pub fn down_deduped(&mut self) -> bool {
        let starting_entry = self.entry().to_owned();
        while self.down() && self.entry() == starting_entry {}
        starting_entry != self.entry()
    }

    /// Whether or not the Console is in a visible state. This does not affect the
    /// functionality of any other method, and is intended to be used by you to
    /// decide whether or not to render the console.
    pub fn shown(&self) -> bool {
        self.shown
    }

    /// Sets the Console to be visible.
    pub fn show(&mut self) {
        self.shown = true;
    }

    /// Sets the Console to be hidden.
    pub fn hide(&mut self) {
        self.shown = false;
    }

    /// Toggles the visible state of the Console.
    pub fn toggle_shown(&mut self) {
        self.shown = !self.shown;
    }
}

#[cfg(not(any(debug_assertions, feature = "force-enabled")))]
impl Console {
    pub const fn enabled(&self) -> bool {
        false
    }

    pub fn confirm<Cmd: std::str::FromStr>(&mut self) -> Result<Cmd, Cmd::Err> {
        "".parse()
    }

    pub fn entry(&self) -> &str {
        ""
    }

    pub fn history(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }

    pub fn history_deduped(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }

    pub fn history_len(&self) -> usize {
        0
    }

    pub fn shown(&self) -> bool {
        false
    }

    pub fn set_entry(&mut self, _entry: String) {}

    pub fn receive_char(&mut self, _ch: char) {}

    pub fn receive_char_if<F: Fn(char) -> bool>(&mut self, _ch: char, _filter: F) -> bool {
        false
    }

    pub fn backspace(&mut self) {}
    pub fn clear(&mut self) {}
    pub fn clear_history(&mut self) {}
    pub fn up(&mut self) -> bool {
        false
    }
    pub fn down(&mut self) -> bool {
        false
    }
    pub fn up_deduped(&mut self) -> bool {
        false
    }
    pub fn down_deduped(&mut self) -> bool {
        false
    }
    pub fn show(&mut self) {}
    pub fn hide(&mut self) {}
    pub fn toggle_shown(&mut self) {}
}

#[cfg(all(feature = "winit", any(debug_assertions, feature = "force-enabled")))]
impl Console {
    pub fn handle_winit_event(&mut self, event: &winit::event::Event<()>) {
        use winit::{
            event::{ElementState, Event, WindowEvent},
            keyboard::{Key, NamedKey},
        };

        const VALID_CHARS: &str =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 ._-\"'/\\~";

        if self.shown() {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {
                            match event.logical_key {
                                Key::Named(NamedKey::Backspace) => {
                                    self.backspace();
                                }
                                Key::Named(NamedKey::ArrowUp) => {
                                    self.up_deduped();
                                }
                                Key::Named(NamedKey::ArrowDown) => {
                                    self.down_deduped();
                                }
                                _ => {
                                    if let Some(text) = &event.text {
                                        let text = text.as_ref();
                                        if VALID_CHARS.contains(text) {
                                            self.receive_text(text);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
}

#[cfg(all(
    feature = "winit",
    not(any(debug_assertions, feature = "force-enabled"))
))]
impl Console {
    pub fn handle_winit_event(&mut self, _event: &winit::event::Event<()>) {}
}

#[cfg(test)]
mod universal_tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let console = Console::new();

        assert!(!console.shown());
        assert_eq!(console.entry(), "");
        assert!(console.history().next().is_none());
    }
}

#[cfg(test)]
#[cfg(any(debug_assertions, feature = "force-enabled"))]
mod debug_tests {
    use super::*;

    #[test]
    fn show_hide_toggle() {
        let mut console = Console::new();

        console.show();
        assert!(console.shown());

        console.hide();
        assert!(!console.shown());

        console.toggle_shown();
        assert!(console.shown());

        console.toggle_shown();
        assert!(!console.shown());
    }

    #[test]
    fn history_grows() {
        let mut console = Console::new();

        console.set_entry("1".into());
        console.confirm::<String>().unwrap();
        assert_eq!(console.history().collect::<Vec<_>>(), vec!["1"]);

        console.set_entry("2".into());
        console.confirm::<String>().unwrap();
        assert_eq!(console.history().collect::<Vec<_>>(), vec!["2", "1"]);

        console.set_entry("3".into());
        console.confirm::<String>().unwrap();
        assert_eq!(console.history().collect::<Vec<_>>(), vec!["3", "2", "1"]);

        console.set_entry("3".into());
        console.confirm::<String>().unwrap();
        assert_eq!(
            console.history().collect::<Vec<_>>(),
            vec!["3", "3", "2", "1"]
        );
        assert_eq!(
            console.history_deduped().collect::<Vec<_>>(),
            vec!["3", "2", "1"]
        );
    }

    #[test]
    fn cursor_movement() {
        let mut console = Console::new();

        {
            console.set_entry("1".into());
            console.confirm::<String>().unwrap();

            console.set_entry("2".into());
            console.confirm::<String>().unwrap();

            console.set_entry("3".into());
            console.confirm::<String>().unwrap();
        }

        assert_eq!(console.entry(), "");

        console.up();
        assert_eq!(console.entry(), "3");

        console.up();
        assert_eq!(console.entry(), "2");

        console.down();
        assert_eq!(console.entry(), "3");

        console.down();
        console.down();
        console.down();
        console.down();
        assert_eq!(console.entry(), "");

        console.up();
        console.up();
        console.up();
        console.up();
        assert_eq!(console.entry(), "1");
    }

    #[test]
    fn deduped_cursor_movement() {
        let mut console = Console::new();

        {
            console.set_entry("1".into());
            console.confirm::<String>().unwrap();

            console.set_entry("2".into());
            console.confirm::<String>().unwrap();
            console.set_entry("2".into());
            console.confirm::<String>().unwrap();
            console.set_entry("2".into());
            console.confirm::<String>().unwrap();

            console.set_entry("3".into());
            console.confirm::<String>().unwrap();
        }

        assert_eq!(console.entry(), "");

        console.up_deduped();
        assert_eq!(console.entry(), "3");

        console.up_deduped();
        assert_eq!(console.entry(), "2");

        console.up_deduped();
        assert_eq!(console.entry(), "1");

        console.down_deduped();
        assert_eq!(console.entry(), "2");

        console.down_deduped();
        assert_eq!(console.entry(), "3");
    }

    #[test]
    fn character_entry() {
        let mut console = Console::new();

        assert_eq!(console.entry(), "");
        console.backspace();
        assert_eq!(console.entry(), "");

        console.receive_char('a');
        console.receive_char('b');
        console.receive_char('c');
        assert_eq!(console.entry(), "abc");

        console.receive_char_if('d', char::is_alphabetic);
        assert_eq!(console.entry(), "abcd");

        console.receive_char_if('0', char::is_alphabetic);
        assert_eq!(console.entry(), "abcd");

        console.backspace();
        console.backspace();
        assert_eq!(console.entry(), "ab");
    }

    #[test]
    fn confirm() {
        let mut console = Console::new();
        console.set_entry("100".into());

        assert_eq!(console.confirm::<usize>(), Ok(100));
        assert_eq!(console.entry(), "");

        console.set_entry("1a0".into());
        assert_eq!(console.confirm::<usize>(), "1a0".parse::<usize>());
        assert_eq!(console.entry(), "");
    }

    #[test]
    fn can_edit_history_items() {
        let mut console = Console::new();
        console.set_entry("100".into());
        assert_eq!(console.confirm::<usize>().unwrap(), 100);

        console.up();
        console.receive_char('0');
        assert_eq!(console.confirm::<usize>().unwrap(), 1000);

        console.up();
        console.backspace();
        console.backspace();
        assert_eq!(console.confirm::<usize>().unwrap(), 10);
    }
}

#[cfg(test)]
#[cfg(not(any(debug_assertions, feature = "force-enabled")))]
mod release_tests {
    use super::*;

    #[test]
    fn console_is_zst() {
        assert_eq!(std::mem::size_of::<Console>(), 0);
    }

    #[test]
    fn methods_do_nothing() {
        let mut console = Console::new();
        console.set_entry("command".into());
        assert_eq!(console.entry(), "");

        console.receive_char('a');
        console.backspace();
        console.clear();
        console.clear_history();
        assert!(!console.up());
        assert!(!console.down());
        assert!(!console.up_deduped());
        assert!(!console.down_deduped());
        console.show();
        console.hide();
        console.toggle_shown();

        assert_eq!(console.receive_char_if('a', |_| true), false);

        assert_eq!(console.confirm::<String>(), "".parse());

        assert_eq!(console.history_len(), 0);
        assert!(console.history().next().is_none());
        assert!(console.history_deduped().next().is_none());
    }
}
