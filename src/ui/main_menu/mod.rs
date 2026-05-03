use std::process::exit;

use ratatui::{
    Frame,
    crossterm::{
        cursor::Show,
        event::KeyCode,
        execute,
        terminal::{LeaveAlternateScreen, disable_raw_mode},
    },
    layout::Rect,
    style::{Color, Modifier},
    widgets::{List, ListItem, ListState},
};

use crate::{
    git::GitWrapper,
    others::exit_code::ExitCode,
    ui::{AppState, UiIface},
};

pub struct MainMenu {
    entry: ListState,
}

impl MainMenu {
    const ENTRIES: &'static [&'static str] = &["Feature", "Release", "Bugfix", "Hotfix"];
    pub fn new() -> Self {
        Self {
            entry: ListState::default().with_selected(Some(0)),
        }
    }
}

impl UiIface for MainMenu {
    fn handle_input(&mut self, key: ratatui::crossterm::event::KeyCode) -> Option<AppState> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                let _ = disable_raw_mode();
                let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
                exit(ExitCode::Ok.code());
            }
            KeyCode::Enter => return AppState::try_from(Self::ENTRIES[self.entry.selected()?]),
            KeyCode::Up => self.entry.select_previous(),
            KeyCode::Down => self.entry.select_next(),
            _ => (),
        }

        None
    }

    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame) {
        let list = List::new(Self::ENTRIES.iter().map(|s| ListItem::from(*s)))
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, body, &mut self.entry);

        self.set_text("Main menu".to_string(), header, frame);

        self.set_text(
            "Up/Down: navigate | Enter: go to selection | Esc: exit".to_string(),
            footer,
            frame,
        );
    }
}
