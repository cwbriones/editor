use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::io::prelude::*;
use std::io::{BufReader,BufWriter};
use std::str::FromStr;

pub struct Client {
    stream: TcpStream,
    output: BufWriter<TcpStream>,
}

impl Client {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Self {
        let stream = TcpStream::connect(addr).unwrap();
        let output = BufWriter::new(stream.try_clone().unwrap());
        Client {
            stream,
            output,
        }
    }

    pub fn text(&mut self, x: usize, y: usize, s: &str) {
        let s = s.replace(',', ",,");
        let com = format!("text,{},{},#000000,{}\n", x, y, s);
        self.output.write(com.as_bytes()).unwrap();
        self.output.flush().unwrap();
    }

    pub fn rect(&mut self, x: usize, y: usize, width: usize, height: usize) {
        let com = format!("rect,{},{},{},{},#000000\n", x, y, width, height);
        self.output.write(com.as_bytes()).unwrap();
        self.output.flush().unwrap();
    }

    pub fn rect_white(&mut self, x: usize, y: usize, width: usize, height: usize) {
        let com = format!("rect,{},{},{},{},#ffffff\n", x, y, width, height);
        self.output.write(com.as_bytes()).unwrap();
        self.output.flush().unwrap();
    }

    pub fn clear(&mut self) {
        self.output.write(b"clear\n").unwrap();
        self.output.flush().unwrap();
    }

    pub fn events(&self) -> Events {
        Events::new(self.stream.try_clone().unwrap())
    }
}

pub struct Events {
    stream: BufReader<TcpStream>,
    buf: String,
}

impl Events {
    pub fn new(stream: TcpStream) -> Self {
        Events {
            stream: BufReader::new(stream),
            buf: String::new(),
        }
    }
}

impl Iterator for Events {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.clear();
        self.stream.read_line(&mut self.buf)
            .ok()
            .and_then(|_| {
                trace!("WIRE: {}", self.buf);
                self.buf.parse::<Event>().ok()
            })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Resize(isize, isize),
    Mouse(MouseEvent),
    Key(KeyEvent),
}

#[derive(Debug, Copy, Clone)]
pub enum MouseEvent {
    Move(isize, isize),
    Down(isize, isize),
    Up(isize, isize),
}

#[derive(Debug, Copy, Clone)]
pub enum KeyEvent {
    Up(Key),
    Down(Key),
}

#[derive(Debug, Copy, Clone)]
pub enum Key {
    Char(char),
    Return,
    Tab,
    Space,
    Comma,
    Up,
    Down,
    Left,
    Right,
    BackSpace,
    Escape,
    LeftShift,
    LeftControl,
    LeftAlt,
    LefCommand,
    RightCommand,
    RightAlt,
    RightControl,
    RightShift,
    CapsLock,
    Unknown,
}

impl FromStr for Event {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(',').map(|s| s.trim());
        match split.next().unwrap() {
            "resize" => {
                let (x, y) = parse_coords(&mut split).unwrap();
                Ok(Event::Resize(x, y))
            },
            "mousedown" => {
                let (x, y) = parse_coords(&mut split).unwrap();
                Ok(Event::Mouse(MouseEvent::Down(x, y)))
            },
            "mouseup" => {
                let (x, y) = parse_coords(&mut split).unwrap();
                Ok(Event::Mouse(MouseEvent::Up(x, y)))
            },
            "mousemove" => {
                let (x, y) = parse_coords(&mut split).unwrap();
                Ok(Event::Mouse(MouseEvent::Move(x, y)))
            },
            "keyup" => {
                let key = split.next()
                    .and_then(|s| s.parse::<Key>().ok())
                    .unwrap_or(Key::Unknown);
                Ok(Event::Key(KeyEvent::Up(key)))
            },
            "keydown" => {
                let key = split.next()
                    .and_then(|s| s.parse::<Key>().ok())
                    .unwrap_or(Key::Unknown);
                Ok(Event::Key(KeyEvent::Down(key)))
            },
            _ => Err(())
        }
    }
}

fn parse_coords<'a, I: Iterator<Item=&'a str>>(split: &mut I) -> Option<(isize, isize)> {
    parse_isize(split)
        .and_then(|x| parse_isize(split).map(|y| (x, y)))
}

fn parse_isize<'a, I: Iterator<Item=&'a str>>(split: &mut I) -> Option<isize> {
    split.next()
        .map(|s| s.trim())
        .and_then(|x| x.parse::<isize>().ok())
}

impl FromStr for Key {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let size = s.chars().count();
        match s {
            _ if size == 1 => Ok(Key::Char(s.chars().next().unwrap())),
            "Return" => Ok(Key::Return),
            "Tab" => Ok(Key::Tab),
            "Space" => Ok(Key::Space),
            "Comma" => Ok(Key::Comma),
            "Up" => Ok(Key::Up),
            "Down" => Ok(Key::Down),
            "Left" => Ok(Key::Left),
            "Right" => Ok(Key::Right),
            "BackSpace" => Ok(Key::BackSpace),
            "Escape" => Ok(Key::Escape),
            "LeftShift" => Ok(Key::LeftShift),
            "LeftControl" => Ok(Key::LeftControl),
            "LeftAlt" => Ok(Key::LeftAlt),
            "LefCommand" => Ok(Key::LefCommand),
            "RightCommand" => Ok(Key::RightCommand),
            "RightAlt" => Ok(Key::RightAlt),
            "RightControl" => Ok(Key::RightControl),
            "RightShift" => Ok(Key::RightShift),
            "Caps_Lock" => Ok(Key::CapsLock),
            "space" => Ok(Key::Char(' ')),
            "leftparen" => Ok(Key::Char('(')),
            "rightparen" => Ok(Key::Char(')')),
            "period" => Ok(Key::Char('.')),
            _ => Err(()),
        }
    }
}

