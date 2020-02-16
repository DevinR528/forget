use std::io;
use std::sync::mpsc;
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
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl EventHandle {
    pub fn with_config(cfg: Config) -> Self {
        let (send, recv) = mpsc::channel();
        let input_handle = {
            let send = send.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for ev in stdin.keys() {
                    match ev {
                        Ok(key) => {
                            if let Err(_e) = send.send(Event::Input(key)) {
                                return;
                            }
                            if key == cfg.exit_key {
                                return;
                            }
                        }
                        Err(e) => panic!("{:?}", e),
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                if let Err(_e) = send.send(Event::Tick) {
                    return;
                }
                thread::sleep(cfg.tick_rate);
            })
        };

        EventHandle {
            recv,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.recv.recv()
    }

    #[allow(dead_code)]
    pub fn shutdown(self) {
        let _ = self.input_handle.join();
        let _ = self.tick_handle.join();
    }
}
