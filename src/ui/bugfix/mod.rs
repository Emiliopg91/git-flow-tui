pub mod finish;
pub mod start;

use crate::{
    git::GitWrapper,
    others::whiteboard::WHITEBOARD,
    ui::{AppState, UiIface},
};

use ratatui::{
    crossterm::event::KeyCode,
    prelude::{Frame, Rect},
    style::{Color, Modifier},
    widgets::{List, ListState},
};
pub struct BugfixList {
    state: ListState,
    list: Option<Vec<String>>,
}

impl BugfixList {
    pub fn new() -> Self {
        Self {
            state: ListState::default().with_selected(Some(0)),
            list: None,
        }
    }
}

impl UiIface for BugfixList {
    fn render(&mut self, _header: Rect, body: Rect, _footer: Rect, frame: &mut Frame) {
        self.list = Some(GitWrapper::global().lock().unwrap().get_bugfixes().unwrap());

        let list = List::new(self.list.clone().unwrap())
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol(" ");

        frame.render_stateful_widget(list, body, &mut self.state);
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppState> {
        match key {
            KeyCode::Esc => return Some(AppState::MainMenu),
            KeyCode::Char('+') => return Some(AppState::BugfixStart),
            KeyCode::Up => self.state.select_previous(),
            KeyCode::Down => self.state.select_next(),
            KeyCode::Delete => {
                if let Some(selected) = self.state.selected()
                    && let Some(list) = &self.list
                    && let Some(branch) = list.get(selected)
                {
                    WHITEBOARD
                        .get()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .insert("branch".to_string(), branch.clone());
                    return Some(AppState::BugfixFinish);
                }
            }
            _ => (),
        }

        None
    }
}
