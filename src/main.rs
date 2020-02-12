use std::io;
use std::time::Duration;

use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::MouseTerminal;
use tui::backend::TermionBackend;
use tui::Terminal;

mod event;
mod app;
mod ux;

use event::{EventHandle, Config, Event};
use app::App;

fn main() -> Result<(), failure::Error> {
    let mut args = std::env::args();
    let tick_rate = if let Some(tick) = args.find(|arg| arg.parse::<u64>().is_ok()) {
        tick.parse()?
    } else {
        250
    };

    let events = EventHandle::with_config(Config {
        tick_rate: Duration::from_millis(tick_rate),
        ..Config::default()
    });
    
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new("Termion demo");
    loop {
        ux::draw(&mut terminal, &app)?;
        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    app.on_key(c);
                }
                Key::Up => {
                    app.on_up();
                }
                Key::Down => {
                    app.on_down();
                }
                Key::Left => {
                    app.on_left();
                }
                Key::Right => {
                    app.on_right();
                }
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
