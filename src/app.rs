use std::fs;
use std::ops::{Index, IndexMut};

use chrono::{DateTime, Local};

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

#[derive(Clone, Debug)]
pub struct ListState<I> {
    pub items: Vec<I>,
    pub selected: usize,
}

impl<I> ListState<I> {
    pub fn new(items: Vec<I>) -> ListState<I> {
        ListState { items, selected: 0 }
    }
    pub fn select_previous(&mut self) {
        if self.selected != 0 {
            self.selected -= 1;
        }
    }
    pub fn select_next(&mut self) {
        if self.selected < self.items.len() {
            self.selected += 1
        }
    }
    pub fn get_selected(&self) -> Option<&I> {
        self.items.get(self.selected)
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

#[derive(Clone, Debug)]
pub struct Todo {
    pub date: DateTime<Local>,
    pub task: String,
    pub cmd: String,
}

#[derive(Clone, Debug)]
pub struct Remind {
    pub title: String,
    pub note: String,
    pub list: ListState<Todo>,
}

impl Remind {
    pub fn to_list(&self) -> impl Iterator<Item = &str> {
        self.list.iter().map(|t| t.task.as_ref())
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
    pub new_note: bool,
    pub sticky_note: ListState<Remind>,
}

impl App {
    pub fn new(title: &str) -> Self {
        let items = vec![
            "first",
            "second",
            "third",
        ];
        let items = items.into_iter()
            .map(|s| Todo {
                date: chrono::Local::now(),
                task: s.into(),
                cmd: "".into(),
            })
            .collect::<Vec<_>>();
        let sticky_note = ListState::new(vec![
            Remind {
                title: "Note One".into(),
                note: "This that and the other.".into(),
                list: ListState::new(items.clone()),
            },
            Remind {
                title: "Note Two".into(),
                note: "Things to not forget.".into(),
                list: ListState::new(items.clone()),
            },
        ]);

        App {
            title: title.into(),
            add_todo: AddTodo::default(),
            add_remind: AddRemind::default(),
            should_quit: false,
            new_reminder: false,
            new_note: false,
            new_todo: false,
            tabs: TabsState::new(sticky_note.items.iter().map(|n| n.title.clone()).collect()),
            sticky_note,
        }
    }

    pub fn on_up(&mut self) {
        self.sticky_note.select_previous();
    }

    pub fn on_down(&mut self) {
        self.sticky_note.select_next();
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    fn add_char(&mut self, c: char) {
        if self.new_reminder {

        } else if self.new_todo {
            if c == '\n' {
                self.sticky_note[self.tabs.index].list.items.push(Todo {
                    date: chrono::Local::now(),
                    task: self.add_todo.task.clone(),
                    cmd: self.add_todo.cmd.clone(),
                });
                self.add_todo.task.clear();
                self.add_todo.cmd.clear();
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
        match c {
            
            _ => {}
        }
    }

    pub fn on_ctrl_key(&mut self, c: char) {
        match c {
            'q' => self.should_quit = true,
            'a' => self.new_todo = !self.new_todo,
            's' => self.new_reminder = !self.new_todo,
            'd' => self.new_note = !self.new_note,
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        // Update UI if needed
    }
}
