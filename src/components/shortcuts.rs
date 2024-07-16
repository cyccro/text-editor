use std::{collections::HashMap, future};

use iced::{
    keyboard::{KeyCode, Modifiers},
    Command,
};

use crate::editor::EditorMsg;

#[derive(Debug)]
pub struct ShortcutManager {
    shortcuts: HashMap<Vec<KeyCode>, fn(()) -> EditorMsg>,
    keys: Vec<KeyCode>,
    last_fn: Option<fn(()) -> EditorMsg>,
    pub accept: bool,
}
impl ShortcutManager {
    pub fn new() -> Self {
        Self {
            shortcuts: HashMap::new(),
            keys: Vec::with_capacity(6),
            last_fn: None,
            accept: false,
        }
    }
    pub fn current_keys_str(&self) -> String {
        Self::keys_to_string(&self.keys)
    }
    fn keys_to_string(vec: &Vec<KeyCode>) -> String {
        let mut s = String::new();
        let mut iter = vec.iter();
        if let Some(key) = iter.next() {
            s.push_str(&*format!("{:?}", key));
        } else {
            return "empty".to_string();
        };
        for key in iter {
            s.push_str(&*format!("-{:?}", key));
        }
        s
    }
    pub fn register(&mut self, shortcut: Vec<KeyCode>, handler: fn(()) -> EditorMsg) {
        self.shortcuts.insert(shortcut, handler);
    }
    pub fn unregister(&mut self, shortcut: Vec<KeyCode>) -> Option<fn(()) -> EditorMsg> {
        self.shortcuts.remove(&shortcut)
    }
    pub fn reset(&mut self) {
        self.keys.clear();
        self.accept = false;
        self.last_fn = None;
    }
    pub fn is_accepting(&self, key: KeyCode) -> bool {
        self.accept
            || match key {
                KeyCode::LControl | KeyCode::RControl => true,
                _ => false,
            }
    }
    pub async fn send_cmd() -> () {}
    pub fn receive(&mut self, key: KeyCode, _modifiers: Modifiers) -> Command<EditorMsg> {
        match key {
            KeyCode::Enter => {
                if let Some(f) = self.last_fn {
                    let cmd = Command::perform(Self::send_cmd(), f);
                    self.reset();
                    return cmd;
                } else {
                    self.reset();
                }
            }
            KeyCode::Escape => self.reset(),
            KeyCode::Colon => self.accept = true,
            _ => {
                if self.is_accepting(key) {
                    self.accept = true;
                    //using 6 as max keys per cmd by now
                    if self.keys.len() > 6 {
                        self.reset();
                        self.accept = true;
                    }
                    self.keys.push(key);
                    if let Some(f) = self.shortcuts.get(&self.keys) {
                        self.last_fn = Some(*f);
                    } else {
                        if self.last_fn.is_some() {
                            self.last_fn = None;
                        }
                    }
                }
            }
        };
        Command::none()
    }
}
