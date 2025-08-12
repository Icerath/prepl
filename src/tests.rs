use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::Repl;

trait TestExt {
    fn write(&mut self, key: &str);
}

impl TestExt for Repl {
    fn write(&mut self, text: &str) {
        for key in text.chars() {
            let event = KeyEvent::new(KeyCode::Char(key), KeyModifiers::empty());
            _ = self.process_event(event, io::sink());
        }
    }
}

#[test]
fn hello_world() {
    let mut repl = Repl::default();
    repl.write("Hello, World!");
    assert_eq!(repl.finish_line(), "Hello, World!");
}
