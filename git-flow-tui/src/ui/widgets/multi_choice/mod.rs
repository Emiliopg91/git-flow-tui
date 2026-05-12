use ratatui::{
    layout::{Constraint, Layout},
    style::Modifier,
    widgets::{Paragraph, StatefulWidget, Widget},
};

pub struct MultiChoiceState {
    selected: usize,
    available: Option<usize>,
}

impl MultiChoiceState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            available: None,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn select_next(&mut self) {
        if let Some(max) = self.available {
            self.selected = (self.selected + 1).min(max.saturating_sub(1));
        }
    }

    pub fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn set_available(&mut self, size: usize) {
        self.available = Some(size)
    }
}

pub struct MultiChoice {
    title: String,
    options: Vec<String>,
}

impl MultiChoice {
    pub fn new(title: &str, options: Vec<String>) -> Self {
        Self {
            title: title.to_owned(),
            options: options.clone(),
        }
    }
}

impl StatefulWidget for MultiChoice {
    type State = MultiChoiceState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        state.set_available(self.options.len());

        let layout_fl = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [question_area, answer_area] = area.layout(&layout_fl);

        let txt = format!("{}:", self.title);
        let par = Paragraph::new(txt);
        par.render(question_area, buf);

        let mut constraints: Vec<Constraint> = Vec::new();
        constraints.push(Constraint::Length(3));
        for i in 0..self.options.len() {
            constraints.push(Constraint::Length((self.options[i].len() + 2) as u16));
        }

        let layout_sl = Layout::horizontal(constraints);
        let areas = answer_area.layout_vec(&layout_sl);

        for i in 0..self.options.len() {
            let mut par = Paragraph::new(format!(" {}", self.options[i].clone()));
            if i == state.selected {
                par = par.style(Modifier::REVERSED);
            }

            par.render(areas[i + 1], buf);
        }
    }
}
