use std::collections::VecDeque;

#[cfg(any(debug_assertions, feature = "enabled_in_release"))]
use itertools::Itertools;

#[derive(Default, Clone, PartialEq, Eq)]
#[cfg(any(debug_assertions, feature = "enabled_in_release"))]
pub struct Console {
    shown: bool,
    entry: String,
    history: VecDeque<String>,
    cursor: Option<usize>,
}

#[derive(Default, Clone, PartialEq, Eq)]
#[cfg(not(any(debug_assertions, feature = "enabled_in_release")))]
pub struct Console;

impl Console {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(any(debug_assertions, feature = "enabled_in_release"))]
impl Console {
    pub fn confirm<Cmd: std::str::FromStr>(&mut self) -> Result<Cmd, Cmd::Err> {
        let entry = self.entry();
        let result = entry.parse();

        self.history
            .push_front(std::mem::replace(&mut self.entry, String::new()));
        self.cursor = None;

        result
    }

    pub fn entry(&self) -> &str {
        match self.cursor {
            Some(n) => &self.history[n],
            None => &self.entry,
        }
    }

    pub fn history(&self) -> impl Iterator<Item = &str> {
        self.history.iter().map(String::as_ref)
    }

    pub fn history_deduped(&self) -> impl Iterator<Item = &str> {
        self.history.iter().map(String::as_ref).dedup()
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn set_entry(&mut self, entry: String) {
        self.entry = entry;
        self.cursor = None;
    }

    pub fn receive_char(&mut self, ch: char) {
        if self.cursor.is_some() {
            self.entry = self.entry().to_owned();
            self.cursor = None;
        }
        self.entry.push(ch)
    }

    pub fn receive_char_if<F: Fn(char) -> bool>(&mut self, ch: char, filter: F) -> bool {
        let accept = filter(ch);
        if accept {
            self.receive_char(ch);
        }
        accept
    }

    pub fn backspace(&mut self) {
        if self.cursor.is_some() {
            self.entry = self.entry().to_owned();
            self.cursor = None;
        }
        self.entry.pop();
    }

    pub fn clear(&mut self) {
        self.entry.clear();
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn up(&mut self) {
        self.cursor = match self.cursor {
            None => Some(0),
            Some(n) if n < (self.history.len() - 1) => Some(n + 1),
            it => it,
        }
    }

    pub fn down(&mut self) {
        self.cursor = match self.cursor {
            Some(n) if n > 0 => Some(n - 1),
            _ => None,
        }
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn show(&mut self) {
        self.shown = true;
    }

    pub fn hide(&mut self) {
        self.shown = false;
    }

    pub fn toggle_shown(&mut self) {
        self.shown = !self.shown;
    }
}

#[cfg(not(any(debug_assertions, feature = "enabled_in_release")))]
impl Console {
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
    pub fn up(&mut self) {}
    pub fn down(&mut self) {}
    pub fn show(&mut self) {}
    pub fn hide(&mut self) {}
    pub fn toggle_shown(&mut self) {}
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
#[cfg(any(debug_assertions, feature = "enabled_in_release"))]
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
#[cfg(not(any(debug_assertions, feature = "enabled_in_release")))]
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
        console.up();
        console.down();
        console.show();
        console.hide();
        console.toggle_shown();

        assert_eq!(console.receive_char_if('a', |_| true), false);

        assert_eq!(console.confirm::<String>(), "".parse());

        assert_eq!(console.history_len(), 0);
        assert!(console.history().empty());
        assert!(console.history_deduped().empty());
    }
}
