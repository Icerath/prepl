use std::io::{self};

fn main() -> io::Result<()> {
    let mut repl = prepl::Repl::default();
    loop {
        let line = repl.read_line()?;
        println!("{line}");
        if line.trim() == "clear" {
            repl.clear()?;
        }
    }
}
