use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

pub struct InputState {
    pub value: String,
}

impl InputState {
    pub fn new(value: &String) -> Self {
        Self {
            value: value.clone(),
        }
    }
}

pub struct TextInput {
    pub title: String,
}

impl TextInput {
    pub fn new(title: &String) -> Self {
        Self {
            title: title.clone(),
        }
    }
}

impl StatefulWidget for TextInput {
    type State = InputState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let txt = format!("{}: {}_", self.title, state.value);
        let par = Paragraph::new(txt);

        par.render(area, buf);
    }
}
