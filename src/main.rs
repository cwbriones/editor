use std::fs::File;
use std::env;
use std::io::prelude::*;

use client::{Client, Event, Key, KeyEvent};
use gapbuffer::GapBuffer;

mod client;

extern crate gapbuffer;
#[macro_use]
extern crate log;
extern crate env_logger;

const CHEIGHT: usize = 14;
const CWIDTH:  usize = 8;

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
            KeyEvent::Down(Key::Char('s')) if self.ctrl => Action::Save,
            KeyEvent::Down(Key::Char('q')) if self.ctrl => Action::Quit,
            KeyEvent::Down(Key::Char(c)) => Action::Insert(c),
            KeyEvent::Down(Key::BackSpace) => Action::Delete,
            KeyEvent::Down(Key::Down) => Action::CursorDown,
            KeyEvent::Down(Key::Up) => Action::CursorUp,
            KeyEvent::Down(Key::Left) => Action::CursorLeft,
            KeyEvent::Down(Key::Right) => Action::CursorRight,
            KeyEvent::Down(Key::Return) => Action::Insert('\n'),
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
    // Position within the buffer
    line: usize,
    ins: usize,
    top_ins: usize,
    // Logical coordinates within the window
    top: usize,
    pos: usize,
    dirty: bool,

    total_lines: usize,
    width: usize,
    height: usize,
    buffer: GapBuffer<char>,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            top: 0,
            top_ins: 0,
            line: 0,
            ins: 0,
            pos: 0,
            total_lines: 1,
            width: 0,
            height: 0,
            dirty: true,
            buffer: GapBuffer::new(),
        }
    }

    fn clear(&mut self) {
        self.line = 0;
        self.ins = 0;
        self.pos = 0;
        self.total_lines = 1;
    }

    pub fn load(&mut self, file: String) {
        self.clear();
        self.buffer.reserve(file.len());
        for (i, c) in file.chars().enumerate() {
            if c == '\n' {
                self.total_lines += 1;
            }
            self.buffer.insert(i, c);
        }
    }

    fn last_newline(&self, scan: usize) -> usize {
        let scan = if scan > self.buffer.len() {
            self.buffer.len()
        } else {
            scan
        };
        for i in (0..scan).rev() {
            if self.buffer[i] == '\n' {
                return i;
            }
        }
        0
    }

    fn next_newline(&self, scan: usize) -> usize {
        for i in scan..self.buffer.len() {
            if self.buffer[i] == '\n' {
                return i;
            }
        }
        self.buffer.len()
    }

    fn get_insertion_point(&self) -> usize {
        self.ins
    }

    fn move_up(&mut self) {
        if self.line > 0 {
            // Move ins to the start of the previous line
            let start = self.last_newline(self.ins);
            let start = self.last_newline(start - 1);
            self.ins = if start == 0 { 0 } else { start + 1 };
            let end = self.next_newline(self.ins);
            let len = end - self.ins;

            // Update the cursor
            self.line -= 1;
            if self.pos > len {
                self.pos = len;
            }
            // Move ins back up
            self.ins += self.pos;
            if self.window_line() == 0 {
                self.top -= 1;
                self.top_ins = self.last_newline(self.top_ins);
                if self.top_ins > 0 {
                    self.top_ins -= 1;
                }
            }
        }
    }

    fn move_down(&mut self) {
        if self.total_lines > 1 && self.line != self.total_lines - 1 {
            // Move ins to the start of the next line.
            self.ins = self.next_newline(self.ins) + 1;

            // Update the pos
            let end = self.next_newline(self.ins);
            let len = end - self.ins;
            if self.pos > len {
                self.pos = len;
            }
            self.line += 1;
            // Move the insertion point back up
            self.ins += self.pos;

            if self.window_line() >= self.max_window_lines() {
                self.top += 1;
                self.top_ins = self.next_newline(self.top_ins) + 1;
            }
        }
    }

    pub fn run(mut self, client: &mut Client) {
        let mut mapping = KeyMapping::new();

        self.render(client);
        for ev in client.events() {
            match ev {
                Event::Key(k) => {
                    let action = mapping.get_action(k);
                    debug!("Editing: {:?}", action);
                    match action {
                        Action::Insert(c) => {
                            let ins = self.get_insertion_point();
                            debug!("ins at {}", ins);
                            self.buffer.insert(ins, c);
                            self.ins += 1;
                            if c == '\n' {
                                self.pos = 0;
                                self.line += 1;
                                self.total_lines += 1;
                            } else {
                                self.pos += 1;
                            }
                        },
                        Action::Delete => {
                            if self.ins > 0 {
                                self.ins -= 1;
                            }
                            match self.buffer.remove(self.ins) {
                                Some('\n') => {
                                    self.total_lines -= 1;
                                    self.line -= 1;

                                    let n = self.last_newline(self.ins);
                                    self.pos = self.ins - n;
                                },
                                Some(_) if self.pos > 0 => {
                                    self.pos -= 1;
                                },
                                _ => {},
                            }
                        },
                        Action::CursorUp => self.move_up(),
                        Action::CursorDown => self.move_down(),
                        Action::CursorLeft => {
                            if self.pos > 0 {
                                self.pos -= 1;
                                self.ins -= 1;
                            }
                        },
                        Action::CursorRight => {
                            let end = self.next_newline(self.ins);
                            let remaining = end - self.ins;
                            if remaining > 0 {
                                self.pos += 1;
                                self.ins += 1;
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
            self.render(client);
        }
    }

    fn max_window_lines(&self) -> usize {
        self.height / CHEIGHT
    }

    fn window_line(&self) -> usize {
        self.line - self.top
    }

    pub fn render(&mut self, client: &mut Client) {
        client.clear();
        let height = self.height as usize;
        let start = self.last_newline(self.top_ins);
        for (y, line) in self.lines(start).take(height).enumerate() {
            client.text(0, y * CHEIGHT, &line);
        }
        let cursor_x = self.pos * CWIDTH;
        let cursor_y = self.line * CHEIGHT;
        client.rect(cursor_x, cursor_y, 1, CHEIGHT);
        self.render_debug(client);
    }

    fn render_debug(&mut self, client: &mut Client) {
        let win_line = self.window_line();
        let cursor = format!("L {}/{} : {} {}", self.line + 1, self.total_lines, self.pos, win_line);
        let insert = format!("I{}", self.ins);
        let stat_x = self.width - CHEIGHT * cursor.len();
        client.text(stat_x, 0, &cursor);
        client.text(stat_x, CHEIGHT, &insert);
    }

    fn lines(&self, from: usize) -> Lines {
        Lines::new(&self.buffer, from)
    }
}

pub struct Lines<'g> {
    items: &'g GapBuffer<char>,
    pos: usize,
}

impl<'g> Lines<'g> {
    pub fn new(gapbuffer: &'g GapBuffer<char>, start: usize) -> Self {
        Lines {
            items: gapbuffer,
            pos: start,
        }
    }
}

impl<'g> Iterator for Lines<'g> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.items.len() {
            return None;
        }
        let mut line = String::new();
        for i in self.pos..self.items.len() {
            self.pos += 1;
            let c = self.items[i];
            if c == '\n' {
                break;
            }
            line.push(c);
        }
        Some(line)
    }
}

fn main() {
    env_logger::init();

    let mut args = env::args();
    let _app = args.next();

    let mut init = String::new();
    if let Some(filename) = args.next() {
        let mut file = File::open(filename).expect("Could not open file");
        file.read_to_string(&mut init).expect("Could not read file");
    }

    let mut client = Client::new("127.0.0.1:5005");

    let mut editor = Editor::new();
    editor.load(init);
    editor.run(&mut client);
}
