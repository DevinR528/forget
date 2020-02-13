use std::io;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct EventHandle {
    recv: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Default for EventHandle {
    fn default() -> Self {
        EventHandle::with_config(Config::default())
    }
}

impl EventHandle {
    pub fn with_config(cfg: Config) -> Self {
        let (send, recv) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::default());
        let input_handle = {
            let ignore_exit_key = ignore_exit_key.clone();
            let send = send.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for ev in stdin.keys() {
                    match ev {
                        Ok(key) => {
                            if let Err(e) = send.send(Event::Input(key)) {
                                return;
                            }
                            if !ignore_exit_key.load(Ordering::Relaxed) && key == cfg.exit_key {
                                return;
                            }
                        }
                        Err(e) => panic!("{:?}", e),
                    }
                }
            })
        };
        let tick_handle = {
            let send = send.clone();
            thread::spawn(move || {
                let s = send.clone();
                loop {
                    s.send(Event::Tick).unwrap();
                    thread::sleep(cfg.tick_rate);
                }
            })
        };

        EventHandle {
            recv,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.recv.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed)
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed)
    }
}
