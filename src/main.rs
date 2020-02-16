use std::io;
use std::time::Duration;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

mod app;
mod config;
mod event;
mod ux;
mod widget;

use app::App;
use event::{Config, Event, EventHandle};

fn main() -> Result<(), failure::Error> {
    let mut args = std::env::args();
    let tick_rate = if let Some(tick) = args.find(|arg| arg.parse::<u64>().is_ok()) {
        tick.parse()?
    } else {
        60
    };

    let mut app = App::new().expect("error from `forget`");

    let events = EventHandle::with_config(Config {
        tick_rate: Duration::from_millis(tick_rate),
        exit_key: termion::event::Key::Ctrl(app.config.exit_key_char_ctrl),
    });

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        ux::draw(&mut terminal, &mut app)?;
        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => app.on_key(c),
                Key::Up => app.on_up(),
                Key::Down => app.on_down(),
                Key::Left => app.on_left(),
                Key::Right => app.on_right(),
                Key::Esc => app.on_ctrl_key('q'),
                Key::Backspace => app.on_backspace(),
                Key::Delete => app.on_delete(),
                Key::Ctrl(c) => app.on_ctrl_key(c),
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            terminal.clear()?;
            break;
        }
    }

    Ok(())
}
