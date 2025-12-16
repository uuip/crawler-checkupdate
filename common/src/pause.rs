use std::io::Write;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> std::io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

pub fn pause() -> std::io::Result<()> {
    println!("Press any key to continue...");
    std::io::stdout().flush()?;

    let _guard = RawModeGuard::new()?;

    loop {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => break Ok(()),
            _ => {}
        }
    }
}
