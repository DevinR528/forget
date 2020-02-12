use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
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
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
    pub fn select_next(&mut self) {
        if self.selected < self.items.len() - 1 {
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

#[derive(Clone, Debug)]
pub struct Todo<'a> {
    pub date: DateTime<Local>,
    pub task: &'a str,
    pub cmd: &'a str,
}

#[derive(Clone, Debug)]
pub struct Note<'a> {
    pub title: &'a str,
    pub list: ListState<Todo<'a>>
}

impl<'a> Note<'a> {
    pub fn to_list(&'a self) -> impl Iterator<Item = &'a str> {
        self.list.iter().map(|t| t.task)
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub tabs: TabsState<'a>,
    pub should_quit: bool,
    pub sticky_note: ListState<Note<'a>>
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> Self {
        let sticky_note = ListState::new(vec![
            Note {
                title: "Note One",
                list: ListState::new(Vec::default()),
            },
            Note {
                title: "Note Two",
                list: ListState::new(Vec::default()),
            },
        ]);
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(sticky_note.items.iter().map(|n| n.title).collect()),
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

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            't' => {
                println!{"WHAT HAPPENS"};
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        // Update UI if needed
    }
}
