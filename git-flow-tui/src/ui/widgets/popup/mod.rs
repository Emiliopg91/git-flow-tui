use ratatui::{
    prelude::{Constraint, Layout},
    widgets::{Block, Clear, Paragraph, Widget},
};

pub struct Popup {
    title: String,
    content: String,
}

impl Popup {
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.to_owned(),
            content: content.to_owned(),
        }
    }
}

impl Widget for Popup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ]);
        let [_, popup_area, _] = area.layout(&layout);
        let popup_block = Block::bordered().title(self.title);

        Clear.render(popup_area, buf);

        let paragraph = Paragraph::new(self.content).block(popup_block);
        paragraph.render(popup_area, buf);
    }
}
