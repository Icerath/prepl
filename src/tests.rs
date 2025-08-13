use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::Repl;

impl Repl {
    fn e(&mut self, event: impl Into<KeyEvent>) {
        _ = self.process_event(event.into(), io::sink());
    }
    fn write(&mut self, text: &str) {
        text.chars().for_each(|key| self.e(KeyCode::Char(key)));
    }
    fn move_left(&mut self, n: usize) {
        (0..n).for_each(|_| self.e(KeyCode::Left));
    }
    fn move_right(&mut self, n: usize) {
        (0..n).for_each(|_| self.e(KeyCode::Right));
    }
}

#[test]
fn hello_world() {
    let mut repl = Repl::default();
    repl.write("Hello, World!");
    assert_eq!(repl.finish_line(), "Hello, World!");
}

#[test]
fn backspace() {
    let mut repl = Repl::default();
    repl.write("Hello, World!");
    repl.e(KeyCode::Backspace);
    repl.move_left(5);
    repl.e(KeyCode::Backspace);
    repl.move_left(2);
    repl.e(KeyCode::Backspace);
    repl.move_right(3);
    repl.e(KeyCode::Delete);
    assert_eq!(repl.finish_line(), "Helo,Wrld");
}
