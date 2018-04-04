use std::io::{self, stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use termion::{clear, cursor};

#[derive(PartialEq)]
pub enum LineType {
    Enter,
    Help,
    Complete,
}

pub struct LineReader {
    buf: String,
    offset: usize,
}

impl LineReader {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            offset: 0,
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.offset = 0;
    }

    fn backspace(&mut self, stdout: &mut RawTerminal<Stdout>) -> io::Result<()> {
        if self.offset == 0 {
            return Ok(());
        }
        if self.offset == self.buf.len() {
            self.buf.pop();
        } else {
            self.buf.remove(self.offset - 1);
        }
        let left = cursor::Left(1 as u16);
        write!(stdout, "{}{}", left, clear::AfterCursor)?;
        self.offset -= 1;
        if self.offset < self.buf.len() {
            let tail = &self.buf[self.offset..];
            let left = cursor::Left((self.buf.len() - self.offset) as u16);
            write!(stdout, "{}{}", tail, left)?;
        }
        Ok(())
    }

    fn delete_word(&mut self, stdout: &mut RawTerminal<Stdout>) -> io::Result<()> {
        if self.buf.len() == 0 {
            return Ok(());
        }
        let left = cursor::Left(self.buf.len() as u16);
        write!(stdout, "{}{}", left, clear::AfterCursor)?;
        self.buf.clear();
        self.offset = 0;
        Ok(())
    }

    pub fn read_line(&mut self) -> io::Result<(&str, LineType)> {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Backspace => self.backspace(&mut stdout)?,
                Key::Char('\t') => {
                    return Ok((&self.buf, LineType::Complete));
                }
                Key::Char('\n') => {
                    write!(stdout, "\r\n")?;
                    return Ok((&self.buf, LineType::Enter));
                }
                Key::Char('?') => {
                    write!(stdout, "?")?;
                    return Ok((&self.buf, LineType::Help));
                }
                Key::Char(c) => {
                    write!(stdout, "{}", c)?;
                    self.buf.insert(self.offset, c);
                    self.offset += 1
                }
                Key::Ctrl('w') => self.delete_word(&mut stdout)?,
                Key::Ctrl('d') => {
                    write!(stdout, "\r\n")?;
                    return Ok(("exit".into(), LineType::Enter));
                }
                Key::Left => {
                    if self.offset > 0 {
                        write!(stdout, "{}", cursor::Left(1 as u16))?;
                        self.offset -= 1
                    }
                }
                Key::Right => {
                    if self.offset < self.buf.len() {
                        write!(stdout, "{}", cursor::Right(1 as u16))?;
                        self.offset += 1
                    }
                }
                _ => {}
            }
            stdout.flush()?;
        }
        Ok((&self.buf, LineType::Enter))
    }
}
