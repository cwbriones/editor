use std::fs::File;
use std::env;
use std::io::prelude::*;

use std::io::BufReader;

use client::{Client, Event, Key, KeyEvent};
use gapbuffer::GapBuffer;

mod client;

extern crate gapbuffer;
#[macro_use]
extern crate log;
extern crate env_logger;

pub struct KeyMapping {
    shift: bool,
    ctrl: bool,
}

impl KeyMapping {
    pub fn new() -> Self {
        KeyMapping {
            shift: false,
            ctrl: false,
        }
    }

    pub fn get_action(&mut self, event: KeyEvent) -> Action {
        // Update state
        match event {
            KeyEvent::Up(Key::LeftShift) => self.shift = false,
            KeyEvent::Down(Key::LeftShift) => self.shift = true,
            KeyEvent::Up(Key::RightShift) => self.shift = false,
            KeyEvent::Down(Key::RightShift) => self.shift = true,

            KeyEvent::Up(Key::LeftControl) => self.ctrl = false,
            KeyEvent::Down(Key::LeftControl) => self.ctrl = true,
            KeyEvent::Up(Key::RightControl) => self.ctrl = false,
            KeyEvent::Down(Key::RightControl) => self.ctrl = true,
            _ => {},
        }

        match event {
            KeyEvent::Down(Key::Char(c)) => Action::Insert(c),
            KeyEvent::Down(Key::BackSpace) => Action::Delete,
            KeyEvent::Down(Key::Char('s')) if self.ctrl => Action::Save,
            KeyEvent::Down(Key::Char('q')) if self.ctrl => Action::Quit,
            KeyEvent::Down(Key::Down) => Action::CursorDown,
            KeyEvent::Down(Key::Up) => Action::CursorUp,
            KeyEvent::Down(Key::Left) => Action::CursorLeft,
            KeyEvent::Down(Key::Right) => Action::CursorRight,
            _ => Action::Noop,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Delete,
    Insert(char),
    CursorUp,
    CursorDown,
    CursorRight,
    CursorLeft,
    Save,
    Quit,
    Noop,
}

pub struct Editor {
    cursor_line: usize,
    cursor_pos: usize,
    width: usize,
    height: usize,
    buffer: GapBuffer<char>,
    client: Client,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            cursor_line: 0,
            cursor_pos: 0,
            width: 0,
            height: 0,
            buffer: GapBuffer::new(),
            client: Client::new("127.0.0.1:5005"),
        }
    }

    pub fn load(&mut self, file: String) {
        for (i, c) in file.chars().enumerate() {
            self.buffer.insert(i, c);
        }
    }

    pub fn run(mut self) {
        let mut mapping = KeyMapping::new();

        for ev in self.client.events() {
            match ev {
                Event::Key(k) => {
                    let action = mapping.get_action(k);
                    debug!("Editing: {:?}", action);
                    match action {
                        Action::Insert(c) => {
                            self.buffer.insert(self.cursor_pos, c);
                            self.cursor_pos += 1;
                        },
                        Action::Delete => {
                            if self.buffer.remove(self.cursor_pos).is_some() {
                                self.cursor_pos -= 1;
                            }
                        },
                        Action::CursorUp => {
                            if self.cursor_line > 0 {
                                self.cursor_line -= 1;
                            }
                        },
                        Action::CursorDown => {
                            if self.cursor_line < self.height / 14 {
                                self.cursor_line -= 1;
                            }
                        },
                        Action::CursorLeft => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                            }
                        },
                        Action::CursorRight => {
                            if self.cursor_pos < self.line_length() {
                                self.cursor_pos += 1;
                            } else if self.cursor_pos < self.buffer.len() {
                                self.cursor_pos = 0;
                                self.cursor_line += 1;
                            }
                        },
                        _ => {},
                    }
                },
                Event::Resize(new_width, new_height) => {
                    self.width = new_width as usize;
                    self.height = new_height as usize;
                },
                _ => {},
            }
        }
    }

    fn keydown(&mut self, key: Key) {
    }

    fn line_length(&mut self) -> usize {
        self.width
    }

    fn render(&mut self) {
        self.client.clear();

        let mut reader = BufReader::new(display_buffer.as_bytes());
        for (y, line) in self.lines().take(height as usize).enumerate() {
            let line = line.unwrap();
            client.text(0, y * 14, &line);
        }

        let cursor_x = self.cursor_pos * 8;
        let cursor_y = self.cursor_line * 14;
        self.client.rect(cursor_x, cursor_y, 1, 14);
    }

    fn lines(&self) -> Lines {
        Lines {
        }
    }
}

struct Lines {
    buf: GapBuffer<char>,
    line: String,
}

impl Lines {
    fn new(gapbuffer: &GapBuffer<char>) -> Self {
        Lines {
            buf: gapbuffer,
            line: String::new(),
        }
    }
}

impl Iterator for Lines {
    type Item = &str;

    fn next(&mut self) -> Option<Self::Item> {
    }
}

fn main() {
    env_logger::init();

    let mut args = env::args();
    let _app = args.next();

    let mut init = String::new();
    if let Some(filename) = args.next() {
        let mut file = File::open(filename).expect("Could not open file");
        file.read_to_string(&mut init);
    }

    let mut editor = Editor::new();
    editor.load(init);
    editor.run();
}
