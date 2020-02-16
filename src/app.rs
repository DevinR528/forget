use std::cell::RefCell;
use std::io;
use std::ops::{Index, IndexMut};
use std::process::{Child, Command, Stdio};
use std::thread;

use chrono::{offset::TimeZone, DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::config::{self, AppConfig};

#[derive(Clone, Debug)]
pub struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
}

impl TabsState {
    pub fn new(titles: Vec<String>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if !self.titles.is_empty() {
            if self.index > 0 {
                self.index -= 1;
            } else {
                self.index = self.titles.len() - 1;
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListState<I> {
    pub items: Vec<I>,
    pub selected: usize,
}

impl<I> Default for ListState<I> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<I> ListState<I> {
    pub fn new(items: Vec<I>) -> ListState<I> {
        ListState { items, selected: 0 }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn select_previous(&mut self) {
        if self.selected != 0 {
            self.selected -= 1;
        }
    }

    pub fn select_next(&mut self) {
        if self.is_empty() {
            return;
        }
        if self.selected < self.len() - 1 {
            self.selected += 1
        }
    }
    pub fn get_selected(&self) -> Option<&I> {
        self.items.get(self.selected)
    }
    pub fn get_selected_mut(&mut self) -> Option<&mut I> {
        self.items.get_mut(self.selected)
    }

    pub fn iter(&self) -> impl Iterator<Item = &I> {
        self.items.iter()
    }
}

impl<I> Index<usize> for ListState<I> {
    type Output = I;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.items[idx]
    }
}
impl<I> IndexMut<usize> for ListState<I> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.items[idx]
    }
}
#[derive(Clone, Debug)]
pub struct AddTodo {
    pub date: DateTime<Local>,
    pub question_index: usize,
    pub task: String,
    pub cmd: String,
}

impl Default for AddTodo {
    fn default() -> Self {
        Self {
            date: chrono::Local::now(),
            question_index: 0,
            task: String::default(),
            cmd: String::default(),
        }
    }
}

impl AddTodo {
    pub fn next(&mut self) {
        if self.question_index != 1 {
            self.question_index += 1
        }
    }
    pub fn previous(&mut self) {
        if self.question_index != 0 {
            self.question_index -= 1
        }
    }
}

#[derive(Clone, Debug)]
pub struct AddRemind {
    pub title: String,
}

impl Default for AddRemind {
    fn default() -> Self {
        Self {
            title: String::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Todo {
    #[serde(with = "date_fmt")]
    pub date: DateTime<Local>,
    pub task: String,
    pub cmd: String,
    pub completed: bool,
}

impl Todo {
    pub fn as_str(&self) -> &str {
        &self.task
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Remind {
    pub title: String,
    pub note: String,
    pub list: ListState<Todo>,
}

impl Default for Remind {
    fn default() -> Self {
        Self {
            title: String::default(),
            note: String::default(),
            list: ListState::default(),
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub title: String,
    pub tabs: TabsState,
    pub add_todo: AddTodo,
    pub add_remind: AddRemind,
    pub should_quit: bool,
    pub new_reminder: bool,
    pub new_todo: bool,
    pub edit_todo: bool,
    pub new_note: bool,
    pub sticky_note: ListState<Remind>,
    pub cmd_handle: RefCell<Vec<thread::JoinHandle<Result<Child, io::Error>>>>,
    pub cmd_err: String,
    pub config: AppConfig,
}

impl App {
    pub fn new() -> io::Result<Self> {
        // this will return early if already present
        // this creates the directory if needed
        config::save_cfg_file()?;

        // this will also save a new copy from
        // `src/config.rs` thread_local APP
        // if the file is not found
        // also checks if the directory is needed
        let sticky_note = config::open_db()?;
        let config = config::open_cfg_file()?;

        Ok(App {
            title: config.title.clone(),
            add_todo: AddTodo::default(),
            add_remind: AddRemind::default(),
            should_quit: false,
            new_reminder: false,
            new_note: false,
            new_todo: false,
            edit_todo: false,
            tabs: TabsState::new(sticky_note.items.iter().map(|n| n.title.clone()).collect()),
            sticky_note,
            cmd_handle: RefCell::new(Vec::default()),
            cmd_err: String::default(),
            config,
        })
    }

    pub fn on_up(&mut self) {
        if self.new_todo {
            self.add_todo.previous()
        } else if self.new_reminder || self.new_note {
            // do nothing TODO how to do this idomaticaly
        } else if !self.sticky_note.is_empty() {
            self.sticky_note[self.tabs.index].list.select_previous()
        }
    }

    pub fn on_down(&mut self) {
        if self.new_todo {
            self.add_todo.next()
        } else if self.new_reminder || self.new_note {
            // do nothing TODO how to do this idomaticaly
        } else if !self.sticky_note.is_empty() {
            self.sticky_note[self.tabs.index].list.select_next();
        }
    }
    /// TODO should any addition be reset here?
    pub fn on_right(&mut self) {
        self.reset_addition();
        self.tabs.next();
    }

    /// TODO should any addition be reset here?
    pub fn on_left(&mut self) {
        self.reset_addition();
        self.tabs.previous();
    }

    fn reset_addition(&mut self) {
        self.add_remind.title.clear();

        self.add_todo.cmd.clear();
        self.add_todo.task.clear();
        self.add_todo.question_index = 0;
    }

    fn run_cmd(&self, cmd: String) {
        self.cmd_handle.borrow_mut().push(thread::spawn(move || {
            let cmd_args = &cmd.split_whitespace().collect::<Vec<_>>();
            let mut cmd = Command::new(&cmd_args[0]);
            let cmd = cmd
                .args(&cmd_args[1..])
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            cmd.spawn()
        }));
    }

    fn add_char(&mut self, c: char) {
        if self.new_reminder {
            if c == '\n' {
                self.sticky_note.items.push(Remind {
                    title: self.add_remind.title.clone(),
                    note: String::default(),
                    list: ListState::default(),
                });
                self.tabs.titles.push(self.add_remind.title.clone());
                self.add_remind.title.clear();
                self.new_reminder = false;
                return;
            }
            self.add_remind.title.push(c);
        } else if self.new_todo && !self.sticky_note.is_empty() {
            if c == '\n' {
                self.sticky_note[self.tabs.index].list.items.push(Todo {
                    date: chrono::Local::now(),
                    task: self.add_todo.task.clone(),
                    cmd: self.add_todo.cmd.clone(),
                    completed: false,
                });
                self.add_todo.task.clear();
                self.add_todo.cmd.clear();
                self.add_todo.question_index = 0;
                self.new_todo = false;
            }

            if self.add_todo.question_index == 0 {
                self.add_todo.task.push(c)
            } else {
                self.add_todo.cmd.push(c)
            }
        } else if self.edit_todo && !self.sticky_note.is_empty() {
            if c == '\n' {
                let idx = self.sticky_note[self.tabs.index].list.selected;
                let todo_len = self.sticky_note[self.tabs.index].list.items.len();
                let todo_items = &mut self.sticky_note[self.tabs.index].list.items;

                todo_items.push(Todo {
                    date: chrono::Local::now(),
                    task: self.add_todo.task.clone(),
                    cmd: self.add_todo.cmd.clone(),
                    completed: false,
                });
                todo_items.swap(idx, todo_len);
                todo_items.pop();

                self.add_todo.task.clear();
                self.add_todo.cmd.clear();
                self.add_todo.question_index = 0;
                self.new_todo = false;
            }

            if self.add_todo.question_index == 0 {
                self.add_todo.task.push(c)
            } else {
                self.add_todo.cmd.push(c)
            }
        } else if self.new_note && !self.sticky_note.is_empty() {
            self.sticky_note[self.tabs.index].note.push(c);
        }
        if c == '\n' && !self.sticky_note.is_empty() {
            if let Some(todo) = self.sticky_note[self.tabs.index].list.get_selected() {
                if !todo.cmd.trim().is_empty() {
                    self.run_cmd(todo.cmd.clone());
                }
            }
        }
    }

    pub fn on_key(&mut self, c: char) {
        self.add_char(c)
    }

    pub fn on_backspace(&mut self) {
        if self.new_reminder {
            self.add_remind.title.pop();
        } else if self.new_todo || self.edit_todo {
            if self.add_todo.question_index == 0 {
                self.add_todo.task.pop();
            } else {
                self.add_todo.cmd.pop();
            }
        } else if self.new_note && !self.sticky_note.is_empty() {
            self.sticky_note[self.tabs.index].note.pop();
        } else if !self.sticky_note.is_empty() {
            if let Some(todo) = self.sticky_note[self.tabs.index].list.get_selected() {
                let flag = todo.completed;

                self.sticky_note[self.tabs.index]
                    .list
                    .get_selected_mut()
                    .unwrap()
                    .completed = !flag;
            }
        }
    }

    pub fn on_delete(&mut self) {
        if self.new_reminder || self.new_todo {
            self.reset_addition();
        } else if self.new_note && !self.sticky_note.is_empty() {
            self.sticky_note[self.tabs.index].note.pop();
        } else if !self.sticky_note.is_empty() {
            let idx = self.sticky_note[self.tabs.index].list.selected;
            if idx > 0 {
                self.sticky_note[self.tabs.index].list.selected -= 1;
            }
            if self.sticky_note[self.tabs.index].list.is_empty() {
                return;
            }
            self.sticky_note[self.tabs.index].list.items.remove(idx);
        }
    }

    pub fn reset_new_flag(&mut self) {
        self.new_note = false;
        self.new_reminder = false;
        self.new_todo = false;
        self.edit_todo = false;
    }

    pub fn on_ctrl_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
                for hndl in self.cmd_handle.get_mut().drain(..) {
                    if let Ok(Ok(mut thread)) = hndl.join() {
                        let _ = thread.kill();
                    }
                }
            }
            // New Todo
            c if c == self.config.new_todo_char_ctrl => {
                let flag = self.new_todo;
                self.reset_new_flag();
                self.new_todo = !flag;
            }
            // Edit Todo
            c if c == self.config.edit_todo_char_ctrl => {
                let flag = self.edit_todo;
                self.reset_new_flag();
                self.edit_todo = !flag;

                if self.edit_todo {
                    self.add_todo.task = self
                        .sticky_note
                        .items
                        .get(self.tabs.index)
                        .map(|n| n.list.get_selected().map(|t| t.task.clone()))
                        .flatten()
                        .unwrap_or_default();

                    self.add_todo.cmd = self
                        .sticky_note
                        .items
                        .get(self.tabs.index)
                        .map(|n| n.list.get_selected().map(|t| t.cmd.clone()))
                        .flatten()
                        .unwrap_or_default();
                }
            }
            // New Sticky Note
            c if c == self.config.new_sticky_note_char_ctrl => {
                let flag = self.new_reminder;
                self.reset_new_flag();
                self.new_reminder = !flag;
            }
            // Add to or New Note
            c if c == self.config.new_note_char_ctrl => {
                let flag = self.new_note;
                self.reset_new_flag();
                self.new_note = !flag;
            }
            // Remove Sticky Note
            c if c == self.config.remove_sticky_note_char_ctrl => {
                if !self.sticky_note.is_empty() {
                    let tab_idx = self.tabs.index;
                    self.sticky_note.items.remove(tab_idx);
                    self.sticky_note.select_previous();
                    self.tabs.titles.remove(tab_idx);
                    self.tabs.previous();
                }
            }
            // Save current Sticky Notes to DB
            c if c == self.config.save_state_to_db_char_ctrl => {
                config::save_db(&self.sticky_note).expect("save to DB failed");
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        // self.cmd_handle
    }
}

mod date_fmt {
    use super::*;

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Local
            .datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
