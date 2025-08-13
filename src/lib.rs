#[cfg(test)]
mod tests;

use std::{
    io::{self, Write, stdout},
    mem,
};

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
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
        let output = self.read_line_internal();
        disable_raw_mode()?;
        output
    }
    /// Clears the entire terminal (including history)
    pub fn clear(&mut self) -> io::Result<()> {
        execute!(stdout(), Clear(ClearType::All), Clear(ClearType::Purge), MoveTo(0, 0))
    }
    fn read_line_internal(&mut self) -> io::Result<String> {
        let stdout = stdout();
        loop {
            let Event::Key(event) = crossterm::event::read()? else { continue };
            if let Some(line) = self.process_event(event, &stdout)? {
                break Ok(line);
            }
        }
    }
    fn process_event(
        &mut self,
        event: KeyEvent,
        mut stdout: impl Write,
    ) -> io::Result<Option<String>> {
        if !event.is_press() {
            return Ok(None);
        }
        let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
        let alt = event.modifiers.contains(KeyModifiers::ALT);
        match event.code {
            KeyCode::Up => {
                let Some(new) = self.history.pop() else { return Ok(None) };
                let line = self.finish_line();
                self.future.push(line);
                self.lhs = new;
                self.render(stdout)?;
            }
            KeyCode::Down => {
                let Some(new) = self.future.pop() else { return Ok(None) };
                let line = self.finish_line();
                self.history.push(line);
                self.lhs = new;
                self.render(stdout)?;
            }
            KeyCode::Left if ctrl => {
                let word = self.jump_word_left();
                self.rhs.insert_str(0, &word);
                self.render(stdout)?;
            }
            KeyCode::Left => {
                let Some(c) = self.lhs.pop() else { return Ok(None) };
                self.rhs.insert(0, c);
                self.render(stdout)?;
            }
            KeyCode::Right if ctrl => {
                let word = self.jump_word_right();
                self.lhs.push_str(&word);
                self.render(stdout)?;
            }
            KeyCode::Right => {
                if !self.rhs.is_empty() {
                    let c = self.rhs.remove(0);
                    self.lhs.push(c);
                    self.render(stdout)?;
                }
            }
            KeyCode::Char('h' | 'w') | KeyCode::Backspace if ctrl => {
                _ = self.jump_word_left();
                self.render(stdout)?;
            }
            KeyCode::Backspace => {
                if self.lhs.pop().is_some() {
                    self.render(stdout)?;
                }
            }
            KeyCode::Char('d') if alt => {
                _ = self.jump_word_right();
                self.render(stdout)?;
            }
            KeyCode::Delete if ctrl => {
                _ = self.jump_word_right();
                self.render(stdout)?;
            }
            KeyCode::Delete => {
                if !self.rhs.is_empty() {
                    self.rhs.remove(0);
                    self.render(stdout)?;
                }
            }
            KeyCode::Enter => {
                execute!(stdout, Print("\n"), MoveToColumn(0))?;
                let line = self.finish_line();
                if !self.future.is_empty() {
                    self.history.push(line.clone());
                }
                self.history
                    .extend(self.future.drain(..).rev().filter(|line| !line.trim().is_empty()));
                if !line.trim().is_empty() {
                    self.history.push(line.clone());
                }
                return Ok(Some(line));
            }
            KeyCode::Char('c') if ctrl => {
                _ = disable_raw_mode();
                std::process::exit(0);
            }
            KeyCode::Char(c) if !alt && !ctrl => {
                self.lhs.push(c);
                self.render(stdout)?;
            }
            _ => {}
        }
        Ok(None)
    }
    #[must_use]
    fn jump_word_left(&mut self) -> String {
        let trimmed = self.lhs.trim_end_matches(not_word_char);
        let end = trimmed.rfind(not_word_char).map_or(0, |i| i + 1);
        self.lhs.split_off(end)
    }
    #[must_use]
    fn jump_word_right(&mut self) -> String {
        let trimmed = self.rhs.trim_start_matches(not_word_char);
        let end = trimmed.find(not_word_char).unwrap_or(trimmed.len());
        let end = end + (self.rhs.len() - trimmed.len());
        // split_off_start
        let word = self.rhs[..end].to_string();
        self.rhs.drain(..end);
        word
    }
    fn finish_line(&mut self) -> String {
        let mut line = mem::take(&mut self.lhs);
        line.push_str(&self.rhs);
        self.rhs.clear();
        line
    }
    #[expect(clippy::cast_possible_truncation)]
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

fn not_word_char(c: char) -> bool {
    !c.is_ascii_alphanumeric()
}
