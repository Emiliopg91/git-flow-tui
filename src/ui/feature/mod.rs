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
pub struct FeatureList {
    state: ListState,
    list: Option<Vec<String>>,
}

impl FeatureList {
    pub fn new() -> Self {
        Self {
            state: ListState::default().with_selected(Some(0)),
            list: None,
        }
    }
}

impl UiIface for FeatureList {
    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame) {
        self.list = Some(GitWrapper::global().lock().unwrap().get_features().unwrap());

        let list = List::new(self.list.clone().unwrap())
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol(" ");

        frame.render_stateful_widget(list, body, &mut self.state);

        self.set_text("Feature management".to_string(), header, frame);

        self.set_text(
            "+: start new feature | del: finish feature | Esc: back".to_string(),
            footer,
            frame,
        );
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppState> {
        match key {
            KeyCode::Esc => return Some(AppState::MainMenu),
            KeyCode::Char('+') => return Some(AppState::FeatureStart),
            KeyCode::Up => self.state.select_previous(),
            KeyCode::Down => self.state.select_next(),
            KeyCode::Delete => {
                if let Some(selected) = self.state.selected()
                    && let Some(list) = &self.list
                        && let Some(branch) = list.get(selected) {
                            WHITEBOARD
                                .get()
                                .unwrap()
                                .lock()
                                .unwrap()
                                .insert("branch".to_string(), branch.clone());
                            return Some(AppState::FeatureFinish);
                        }
            }
            _ => (),
        }

        None
    }
}
