use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use chrono::Local;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use termion::event::Key;
use tui::style::{Color, Modifier, Style};

use crate::app::{ListState, Remind, Todo};

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

impl AppMod {
    fn modifier(&self) -> &str {
        match self.bits() {
            1 => "BOLD",
            2 => "DIM",
            3 => "ITALIC",
            4 => "UNDERLINED",
            5 => "SLOW_BLINK",
            6 => "RAPID_BLINK",
            7 => "REVERSED",
            8 => "HIDDEN",
            9 => "CROSSED_OUT",
            _ => "RESET",
        }
    }
}

impl Serialize for AppMod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.modifier())
    }
}

impl<'de> Deserialize<'de> for AppMod {
    fn deserialize<D>(deserializer: D) -> Result<AppMod, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AppModVisit;
        impl<'de> Visitor<'de> for AppModVisit {
            type Value = AppMod;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("One of 9 ascii text modifiers `BOLD, ITALIC, DIM, ect")
            }

            fn visit_str<E>(self, value: &str) -> Result<AppMod, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "BOLD" => Ok(AppMod::BOLD),
                    "DIM" => Ok(AppMod::DIM),
                    "ITALIC" => Ok(AppMod::ITALIC),
                    "UNDERLINED" => Ok(AppMod::UNDERLINED),
                    "SLOW_BLINK" => Ok(AppMod::SLOW_BLINK),
                    "RAPID_BLINK" => Ok(AppMod::RAPID_BLINK),
                    "REVERSED" => Ok(AppMod::REVERSED),
                    "HIDDEN" => Ok(AppMod::HIDDEN),
                    "CROSSED_OUT" => Ok(AppMod::CROSSED_OUT),
                    "RESET" => Ok(AppMod::empty()),
                    _ => Err(serde::de::Error::unknown_field(value, &[""])),
                }
            }
        }
        deserializer.deserialize_str(AppModVisit)
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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
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
    pub fg: AppColor,
    pub bg: AppColor,
    pub modifier: AppMod,
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
    pub text: AppStyle,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub title: String,
    pub new_sticky_note_char_ctrl: char,
    pub new_note_char_ctrl: char,
    pub new_todo_char_ctrl: char,
    pub edit_todo_char_ctrl: char,
    pub mark_done: AppKey,
    pub remove_todo: AppKey,
    pub remove_sticky_note_char_ctrl: char,
    pub save_state_to_db_char_ctrl: char,
    pub exit_key_char_ctrl: char,
    pub highlight_string: String,
    pub app_colors: ColorCfg,
}

thread_local! { pub static CFG: AppConfig = AppConfig {
    title: "Forget It".into(),
    new_sticky_note_char_ctrl: 'h',
    new_note_char_ctrl: 'k',
    new_todo_char_ctrl: 'n',
    edit_todo_char_ctrl: 'e',
    mark_done: AppKey::Backspace,
    remove_todo: AppKey::Delete,
    remove_sticky_note_char_ctrl: 'u',
    save_state_to_db_char_ctrl: 's',
    exit_key_char_ctrl: 'q',
    highlight_string: "✔️".into(),
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
        text: AppStyle {
            fg: AppColor::Green,
            bg: AppColor::Reset,
            modifier: AppMod::ITALIC,
        },
    },
}}

thread_local! { pub static APP: ListState<Remind> = ListState {
    items: vec![ Remind {
            title: "Note One".into(),
            note: "You can add to the Notes by hitting ctrl-k.".into(),
            list: ListState {
                items: vec![
                    Todo {
                        date: Local::now(),
                        task: "You can add a Sticky Note by hitting ctrl-h".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
                        task: "You can add a Todo by hitting ctrl-n".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
                        task: "You can check off a Todo by hitting Backspace".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
                        task: "You can delete a Todo by hitting Delete".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
                        task: "You can delete a Sticky by hitting ctrl-u".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
                        task: "You can save to the data base by hitting ctrl-s".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
                        task: "Oh you can exit by ctrl-q or Esc".into(),
                        cmd: String::new(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
                        task: "Todo's can run commands when selected with Enter.".into(),
                        cmd: "sensible-browser https://github.com/DevinR528/forget".into(),
                        completed: false
                    }
                ],
                selected: 0
            }
        },
        Remind {
            title: "Note Two".into(),
            note: "".into(),
            list: ListState {
                items: vec![
                    Todo {
                        date: Local::now(),
                        task: "First".into(),
                        cmd: "".into(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
                        task: "Second".into(),
                        cmd: "".into(),
                        completed: false
                    },
                    Todo {
                        date: Local::now(),
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

pub fn save_cfg_file() -> io::Result<()> {
    let mut home = dirs::home_dir().expect("home dir not found");
    home.push(".forget");
    home.push("config.json");

    if !Path::new(&home).exists() {
        let mut dir = home.clone();
        dir.pop();
        std::fs::create_dir_all(dir)?;

        CFG.with(move |cfg| {
            let home = home;
            println!("{:?}", home);
            let json_str = serde_json::to_string_pretty(cfg).expect("serialization failed");

            let mut fd = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(home)
                .expect("open file failed");

            fd.write_all(json_str.as_bytes())
        })
    } else {
        Ok(())
    }
}

pub fn open_cfg_file() -> io::Result<AppConfig> {
    let mut home = dirs::home_dir().unwrap();
    home.push(".forget");
    home.push("config.json");

    let json_raw = fs::read_to_string(home)?;
    Ok(serde_json::from_str::<AppConfig>(&json_raw).expect("deserialization failed"))
}

pub fn open_db() -> io::Result<ListState<Remind>> {
    let mut home = dirs::home_dir().unwrap();
    home.push(".forget");
    home.push("note_db.json");

    if !Path::new(&home).exists() {
        let mut dir = home.clone();
        dir.pop();
        std::fs::create_dir_all(dir)?;
        APP.with(|app| {
            let json_str = serde_json::to_string(&app).expect("serialization failed");
            let mut fd = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&home)
                .expect("open file failed");

            fd.write_all(json_str.as_bytes()).expect("write failed");
        });
    }
    let json_raw = fs::read_to_string(&home)?;
    Ok(serde_json::from_str::<ListState<Remind>>(&json_raw).expect("deserialization failed"))
}

pub fn save_db(notes: &ListState<Remind>) -> io::Result<()> {
    let mut home = dirs::home_dir().unwrap();
    home.push(".forget");
    home.push("note_db.json");

    let json_str = serde_json::to_string(notes)?;
    let mut fd = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(home)?;
    fd.write_all(json_str.as_bytes())
}
