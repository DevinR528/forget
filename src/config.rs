use std::fs;
use std::io;
use std::path::Path;
use std::ops::{Index, IndexMut};

use chrono::{offset::TimeZone, DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_slice, from_str, to_string, to_vec};
use termion::event::Key;
use tui::style::{Color, Modifier, Style};

use crate::app::{App, ListState, Remind, Todo};

/// A key.
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum AppKey {
    /// Backspace.
    Backspace,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Backward Tab key.
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Function keys.
    ///
    /// Only function keys 1 through 12 are supported.
    F(u8),
    /// Normal character.
    Char(char),
    /// Alt modified character.
    Alt(char),
    /// Ctrl modified character.
    ///
    /// Note that certain keys may not be modifiable with `ctrl`, due to limitations of terminals.
    Ctrl(char),
    /// Null byte.
    Null,
    /// Esc key.
    Esc,

    #[doc(hidden)]
    __IsNotComplete,
}

impl Into<Key> for AppKey {
    fn into(self) -> Key {
        match self {
            Self::Backspace => Key::Backspace,
            Self::Left => Key::Left,
            Self::Right => Key::Right,
            Self::Up => Key::Up,
            Self::Down => Key::Down,
            Self::Home => Key::Home,
            Self::End => Key::End,
            Self::PageUp => Key::PageUp,
            Self::PageDown => Key::PageDown,
            Self::BackTab => Key::BackTab,
            Self::Delete => Key::Delete,
            Self::Insert => Key::Insert,
            Self::F(int) => Key::F(int),
            Self::Char(c) => Key::Char(c),
            Self::Alt(c) => Key::Alt(c),
            Self::Ctrl(c) => Key::Ctrl(c),
            Self::Null => Key::Null,
            Self::Esc => Key::Esc,
            _ => unreachable!("semver broken termion crate"),
        }
    }
}

bitflags::bitflags! {
    #[derive(Deserialize, Serialize)]
    pub struct AppMod: u16 {
        const BOLD = 0b0000_0000_0001;
        const DIM = 0b0000_0000_0010;
        const ITALIC = 0b0000_0000_0100;
        const UNDERLINED = 0b0000_0000_1000;
        const SLOW_BLINK = 0b0000_0001_0000;
        const RAPID_BLINK = 0b0000_0010_0000;
        const REVERSED = 0b0000_0100_0000;
        const HIDDEN = 0b0000_1000_0000;
        const CROSSED_OUT = 0b0001_0000_0000;
    }
}

impl Into<Modifier> for AppMod {
    fn into(self) -> Modifier {
        match self.bits() {
            1 => Modifier::BOLD,
            2 => Modifier::DIM,
            3 => Modifier::ITALIC,
            4 => Modifier::UNDERLINED,
            5 => Modifier::SLOW_BLINK,
            6 => Modifier::RAPID_BLINK,
            7 => Modifier::REVERSED,
            8 => Modifier::HIDDEN,
            9 => Modifier::CROSSED_OUT,
            _ => Modifier::empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum AppColor {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

impl Into<Color> for AppColor {
    fn into(self) -> Color {
        match self {
            Self::Reset => Color::Reset,
            Self::Black => Color::Black,
            Self::Red => Color::Red,
            Self::Green => Color::Green,
            Self::Yellow => Color::Yellow,
            Self::Blue => Color::Blue,
            Self::Magenta => Color::Magenta,
            Self::Cyan => Color::Cyan,
            Self::Gray => Color::Gray,
            Self::DarkGray => Color::DarkGray,
            Self::LightRed => Color::LightRed,
            Self::LightGreen => Color::LightGreen,
            Self::LightYellow => Color::LightYellow,
            Self::LightBlue => Color::LightBlue,
            Self::LightMagenta => Color::LightMagenta,
            Self::LightCyan => Color::LightCyan,
            Self::White => Color::White,
            Self::Indexed(i) => Color::Indexed(i),
            Self::Rgb(r, g, b) => Color::Rgb(r, g, b),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AppStyle {
    fg: AppColor,
    bg: AppColor,
    modifier: AppMod,
}

impl Into<Style> for AppStyle {
    fn into(self) -> Style {
        Style {
            fg: self.fg.into(),
            bg: self.bg.into(),
            modifier: self.modifier.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ColorCfg {
    pub normal: AppStyle,
    pub highlight: AppStyle,
    pub tabs: AppStyle,
    pub titles: AppStyle,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig<'a> {
    pub title: &'a str,
    pub new_sticky_note_ctrl: char,
    pub new_note_ctrl: char,
    pub new_todo_ctrl: char,
    pub mark_done: AppKey,
    pub remove_todo: AppKey,
    pub remove_sticky_note_ctrl: char,
    pub save_state_to_db_ctrl: char,
    pub app_colors: ColorCfg,
}

const TITLE: &str = "Forget It";
pub const CFG: AppConfig = AppConfig {
    title: TITLE,
    new_sticky_note_ctrl: 's',
    new_note_ctrl: 'n',
    new_todo_ctrl: 't',
    mark_done: AppKey::Backspace,
    remove_todo: AppKey::Delete,
    remove_sticky_note_ctrl: 'k',
    save_state_to_db_ctrl: 'p',
    app_colors: ColorCfg {
        normal: AppStyle {
            fg: AppColor::White,
            bg: AppColor::Reset,
            modifier: AppMod::empty(),
        },
        highlight: AppStyle {
            fg: AppColor::Yellow,
            bg: AppColor::Reset,
            modifier: AppMod::BOLD,
        },
        tabs: AppStyle {
            fg: AppColor::Cyan,
            bg: AppColor::Reset,
            modifier: AppMod::BOLD,
        },
        titles: AppStyle {
            fg: AppColor::Red,
            bg: AppColor::Reset,
            modifier: AppMod::BOLD,
        },
    },
};

thread_local! { pub static APP: ListState<Remind> = ListState {
    items: vec![ Remind {
            title: "Note One".into(),
            note: "You can add to the Notes by hitting ctrl-n.".into(),
            list: ListState {
                items: vec![
                    Todo {
                        date: "2020-02-13 22:46:08".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "You can add a Sticky Note by hitting ctrl-s".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 22:46:08".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "You can add a Todo by hitting ctrl-t".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 22:46:08".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "You can check off a Todo by hitting Backspace".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 22:46:08".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "You can delete a Todo by hitting Delete".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 22:46:08".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "You can delete a Sticky by hitting ctrl-k".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 22:46:08".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "You can save to the data base by hitting ctrl-p".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 22:46:08".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "Oh you can exit by ctrl-q or Esc".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 23:28:13".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "You can add commands to run when selected with Enter.".into(),
                        cmd: "sensible-browser https://github.com/DevinR528/forget".into(),
                        completed: false
                    }
                ],
                selected: 7
            }
        },
        Remind {
            title: "Note Two".into(),
            note: "".into(),
            list: ListState {
                items: vec![
                    Todo {
                        date: "2020-02-13 23:17:43".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "First".into(),
                        cmd: "".into(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 23:17:45".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "Second".into(),
                        cmd: "".into(),
                        completed: false
                    },
                    Todo {
                        date: "2020-02-13 23:17:49".parse::<chrono::DateTime<chrono::Local>>().expect("failed"),
                        task: "Third".into(),
                        cmd: "".into(),
                        completed: false
                    }
                ],
                selected: 0
            }
        }
    ],
    selected: 0
}}

pub fn save_cfg_file(cfg: &AppConfig) -> io::Result<()> {
    use std::io::Write;

    let mut home = dirs::home_dir().expect("home dir not found");
    home.push(".forget");
    home.push("config.json");

    // TODO make it negative
    if !Path::new("./config.json").exists() {
        let json_str = serde_json::to_string_pretty(cfg).expect("serialization failed");

        let mut fd = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("./config.json")
            .expect("open file failed");

        fd.write_all(json_str.as_bytes())
    } else {
        Ok(())
    }
}

pub fn open_cfg_file<'a>() -> AppConfig<'a> {
    let mut home = dirs::home_dir().expect("home dir not found");
    home.push(".forget");
    let json_raw = fs::read_to_string("./config.json").expect("failed to read database");
    let s: &'a str = unsafe { std::mem::transmute(json_raw.as_str()) };
    from_str::<AppConfig<'a>>(s).expect("deserialization failed")
}
