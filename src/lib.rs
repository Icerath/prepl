use std::{
    io::{self, Write, stdout},
    mem,
};

use crossterm::{
    cursor::MoveToColumn,
    event::{Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

#[derive(Default)]
pub struct Repl {
    lhs: String,
    rhs: String,
    history: Vec<String>,
    future: Vec<String>,
}

impl Repl {
    pub fn read_line(&mut self) -> io::Result<String> {
        // disable raw mode even if read_line_internal errors
        enable_raw_mode()?;
        let output = self.read_line_interal();
        disable_raw_mode()?;
        output
    }
    fn read_line_interal(&mut self) -> io::Result<String> {
        let mut stdout = stdout();
        loop {
            let Event::Key(event) = crossterm::event::read()? else { continue };
            if !event.is_press() {
                continue;
            };
            let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
            match event.code {
                KeyCode::Up => {
                    let Some(new) = self.history.pop() else { continue };
                    let line = self.finish_line();
                    self.future.push(line);
                    self.lhs = new;
                    self.render(&stdout)?;
                }
                KeyCode::Down => {
                    let Some(new) = self.future.pop() else { continue };
                    let line = self.finish_line();
                    self.history.push(line);
                    self.lhs = new;
                    self.render(&stdout)?;
                }
                KeyCode::Left if ctrl => {
                    let trimmed = self.lhs.trim_end();
                    let till_ws = trimmed.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
                    if till_ws >= self.lhs.len() {
                        continue;
                    }
                    let rhs = self.lhs.split_off(till_ws);
                    self.rhs.insert_str(0, &rhs);
                    self.render(&stdout)?;
                }
                KeyCode::Left => {
                    let Some(c) = self.lhs.pop() else { continue };
                    self.rhs.insert(0, c);
                    self.render(&stdout)?;
                }
                KeyCode::Right if ctrl => {
                    let trimmed = self.rhs.trim_start();
                    let till_ws = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
                    let till_ws = till_ws + (self.rhs.len() - trimmed.len());
                    if till_ws > self.rhs.len() {
                        continue;
                    }
                    let mut rhs = self.rhs.split_off(till_ws);
                    mem::swap(&mut rhs, &mut self.rhs);
                    self.lhs.push_str(&rhs);
                    self.render(&stdout)?;
                }
                KeyCode::Right => {
                    if self.rhs.is_empty() {
                        continue;
                    }
                    let c = self.rhs.remove(0);
                    self.lhs.push(c);
                    self.render(&stdout)?;
                }
                KeyCode::Char('w') | KeyCode::Backspace if ctrl => {
                    let trimmed = self.lhs.trim_end();
                    let till_ws = trimmed.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
                    if till_ws >= self.lhs.len() {
                        continue;
                    }
                    self.lhs.truncate(till_ws);
                    self.render(&stdout)?;
                }
                KeyCode::Backspace => {
                    let Some(_) = self.lhs.pop() else { continue };
                    self.render(&stdout)?
                }
                KeyCode::Enter => {
                    writeln!(stdout)?;
                    let line = self.finish_line();
                    if !self.future.is_empty() {
                        self.history.push(line.clone());
                    }
                    self.history
                        .extend(self.future.drain(..).rev().filter(|line| !line.trim().is_empty()));
                    if !line.trim().is_empty() {
                        self.history.push(line.clone());
                    }
                    break Ok(line);
                }
                KeyCode::Char('c') if ctrl => std::process::exit(0),
                KeyCode::Char(c) => {
                    self.lhs.push(c);
                    self.render(&stdout)?;
                }
                _ => {}
            }
        }
    }
    fn finish_line(&mut self) -> String {
        let mut line = mem::take(&mut self.lhs);
        line.push_str(&self.rhs);
        self.rhs.clear();
        line
    }
    fn render(&self, mut stdout: impl Write) -> io::Result<()> {
        execute!(
            stdout,
            Print("\r"),
            Clear(ClearType::CurrentLine),
            Print(&self.lhs),
            Print(&self.rhs),
            MoveToColumn(self.lhs.len() as u16)
        )
    }
}
