use std::fs;
use std::io;
use std::path::Path;
use std::ops::{Index, IndexMut};

use chrono::{offset::TimeZone, DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_slice, from_str, to_string, to_vec};

use crate::config::{CFG, open_cfg_file, save_cfg_file, AppConfig};

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
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
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

    pub fn select_previous(&mut self) {
        if self.selected != 0 {
            self.selected -= 1;
        }
    }

    pub fn select_next(&mut self) {
        if self.len() == 0 {
            return;
        }
        if self.selected < self.len() - 1 {
            self.selected += 1
        }
    }
    pub fn get_selected(&self) -> &I {
        self.items.get(self.selected).unwrap()
    }
    pub fn get_selected_mut(&mut self) -> &mut I {
        self.items.get_mut(self.selected).unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = &I> {
        self.items.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut I> {
        self.items.iter_mut()
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

impl Remind {
    pub fn to_list(&self) -> impl Iterator<Item = &str> {
        self.list.iter().map(|t| t.as_str())
    }
}

fn open_db() -> ListState<Remind> {
    let mut home = dirs::home_dir().expect("home dir not found");
    home.push(".forget");
    home.push("note_db.json");

    // TODO make it the right file
    if !Path::new("./note_db.json").exists() {
        use std::io::Write;
        crate::config::APP.with(|app| {
            let json_str = serde_json::to_string(&app).expect("serialization failed");
            let mut fd = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open("./note_db.json")
                .expect("open file failed");

            fd.write_all(json_str.as_bytes()).expect("write failed");
        });
    }

    let json_raw = fs::read_to_string("./note_db.json").expect("failed to read database");
    from_str::<ListState<Remind>>(&json_raw).expect("deserialization failed")
}

fn save_db(notes: &ListState<Remind>) -> io::Result<()> {
    use std::io::Write;

    let mut home = dirs::home_dir().expect("home dir not found");
    home.push(".forget");

    let json_str = serde_json::to_string(notes).expect("serialization failed");
    let mut fd = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./note_db.json")
        .expect("open file failed");
    fd.write_all(json_str.as_bytes())
}

#[derive(Debug)]
pub struct App<'a> {
    pub title: String,
    pub tabs: TabsState,
    pub add_todo: AddTodo,
    pub add_remind: AddRemind,
    pub should_quit: bool,
    pub new_reminder: bool,
    pub new_todo: bool,
    pub new_note: bool,
    pub sticky_note: ListState<Remind>,
    pub config: AppConfig<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        save_cfg_file(&CFG).expect("save cfg failed");

        let sticky_note = open_db();
        let config = open_cfg_file();

        App {
            title: config.title.into(),
            add_todo: AddTodo::default(),
            add_remind: AddRemind::default(),
            should_quit: false,
            new_reminder: false,
            new_note: false,
            new_todo: false,
            tabs: TabsState::new(sticky_note.items.iter().map(|n| n.title.clone()).collect()),
            sticky_note,
            config,
        }
    }

    pub fn on_up(&mut self) {
        if self.new_todo {
            self.add_todo.previous()
        } else if self.new_reminder || self.new_note {
            // do nothing TODO how to do this idomaticaly
        } else {
            self.sticky_note[self.tabs.index].list.select_previous()
        }
    }

    pub fn on_down(&mut self) {
        if self.new_todo {
            self.add_todo.next()
        } else if self.new_reminder || self.new_note {
            // do nothing TODO how to do this idomaticaly
        } else {
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
        } else if self.new_todo {
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
                return;
            }
            if self.add_todo.question_index == 0 {
                self.add_todo.task.push(c)
            } else {
                self.add_todo.cmd.push(c)
            }
        } else if self.new_note {
            self.sticky_note[self.tabs.index].note.push(c);
        }
    }

    pub fn on_key(&mut self, c: char) {
        self.add_char(c)
    }

    pub fn on_backspace(&mut self) {
        if self.new_reminder {
            self.add_remind.title.pop();
        } else if self.new_todo {
            if self.add_todo.question_index == 0 {
                self.add_todo.task.pop();
            } else {
                self.add_todo.cmd.pop();
            }
        } else if self.new_note {
            self.sticky_note[self.tabs.index].note.pop();
        } else if self.sticky_note[self.tabs.index].list.len() > 0 {
            self.sticky_note[self.tabs.index]
                .list
                .get_selected_mut()
                .completed = true;
        }
    }

    pub fn on_delete(&mut self) {
        if self.new_reminder || self.new_todo {
            self.reset_addition();
        } else if self.new_note {
            self.sticky_note[self.tabs.index].note.pop();
        } else {
            let idx = self.sticky_note[self.tabs.index].list.selected;
            if idx > 0 {
                self.sticky_note[self.tabs.index].list.selected -= 1;
            }
            self.sticky_note[self.tabs.index].list.items.remove(idx);
        }
    }

    pub fn reset_new_flag(&mut self) {
        self.new_note = false;
        self.new_reminder = false;
        self.new_todo = false;
    }

    pub fn on_ctrl_key(&mut self, c: char) {
        match c {
            'q' => self.should_quit = true,
            c if c == self.config.new_todo_ctrl => {
                let flag = self.new_todo;
                self.reset_new_flag();
                self.new_todo = !flag;
            }
            c if c == self.config.new_sticky_note_ctrl => {
                let flag = self.new_reminder;
                self.reset_new_flag();
                self.new_reminder = !flag;
            }
            c if c == self.config.new_note_ctrl => {
                let flag = self.new_note;
                self.reset_new_flag();
                self.new_note = !flag;
            }
            c if c == self.config.remove_sticky_note_ctrl => {
                if self.sticky_note.len() > 0 {
                    let tab_idx = self.tabs.index;
                    self.sticky_note.items.remove(tab_idx);
                    self.sticky_note.select_next();
                    self.tabs.titles.remove(tab_idx);
                    self.tabs.previous();
                }
            }
            c if c == self.config.save_state_to_db_ctrl => {
                save_db(&self.sticky_note).expect("save to DB failed");
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        // Update UI if needed
    }
}
