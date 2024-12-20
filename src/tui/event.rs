use std::io;

use crossterm::event::{self, Event, KeyCode};

use crate::app::{App, SelectedTab};

pub fn handle_events(app: &mut App) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => app.quit(),
            KeyCode::Char('h') | KeyCode::Tab => app.next_tab(),
            KeyCode::Char('l') | KeyCode::BackTab => app.previous_tab(),
            KeyCode::Char('m') => app.current_tab = SelectedTab::MediaSort,
            KeyCode::Char('b') => app.current_tab = SelectedTab::BadKeywords,
            KeyCode::Char('f') => app.current_tab = SelectedTab::Flags,
            KeyCode::Char('p') => app.current_tab = SelectedTab::Profiles,
            _ => {}
        }
    }
    Ok(())
}
