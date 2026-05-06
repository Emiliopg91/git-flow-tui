mod bugfix;
mod feature;
mod main_menu;
mod release;
mod widgets;

use std::time::Duration;

use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::{
    feature::{finish::FeatureFinish, start::FeatureStart, FeatureList},
    main_menu::MainMenu,
    release::{finish::ReleaseFinish, start::ReleaseStart, ReleaseList},
};

use self::bugfix::{finish::BugfixFinish, start::BugfixStart, BugfixList};

pub enum AppState {
    MainMenu,
    FeatureList,
    FeatureStart,
    FeatureFinish,
    ReleaseList,
    ReleaseStart,
    ReleaseFinish,
    BugfixList,
    BugfixStart,
    BugfixFinish,
    //    Hotfix,
}

pub trait UiIface {
    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame);
    fn handle_input(&mut self, key: KeyCode) -> Option<AppState>;
    fn tick(&mut self) {}
    fn set_text(&mut self, txt: String, area: Rect, frame: &mut Frame) {
        let widget = Paragraph::new(txt)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(widget, area);
    }
}

pub struct App {
    state: AppState,
    page: Box<dyn UiIface>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::MainMenu,
            page: Box::new(MainMenu::new()),
        }
    }
}

pub fn main_loop() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut app = App::new();

    ratatui::run(|terminal| loop {
        if event::poll(Duration::from_millis(50))?
            && let Some(key) = event::read()?.as_key_press_event()
                && let Some(next) = app.page.handle_input(key.code) {
                    app.state = next;
                    match app.state {
                        AppState::MainMenu => app.page = Box::new(MainMenu::new()),
                        AppState::FeatureList => app.page = Box::new(FeatureList::new()),
                        AppState::FeatureStart => app.page = Box::new(FeatureStart::new()),
                        AppState::FeatureFinish => app.page = Box::new(FeatureFinish::new()),
                        AppState::ReleaseList => app.page = Box::new(ReleaseList::new()),
                        AppState::ReleaseStart => app.page = Box::new(ReleaseStart::new()),
                        AppState::ReleaseFinish => app.page = Box::new(ReleaseFinish::new()),
                        AppState::BugfixList => app.page = Box::new(BugfixList::new()),
                        AppState::BugfixStart => app.page = Box::new(BugfixStart::new()),
                        AppState::BugfixFinish => app.page = Box::new(BugfixFinish::new()),
                    }
                }

        app.page.tick();

        terminal.draw(|frame| render(frame, &mut app))?;
    })
}

fn render(frame: &mut Frame, app: &mut App) {
    let outer = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]);

    let area = frame.area().inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let [header_area, body_area, footer_area] = area.layout(&outer);

    let content_width = body_area.width.saturating_sub(2).min(120);

    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(content_width),
        Constraint::Fill(1),
    ]);

    let [_, centered_body, _] = body_area.layout(&horizontal);

    let block = Block::default().borders(Borders::ALL);
    frame.render_widget(block, centered_body);

    let inner = centered_body.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    app.page.render(header_area, inner, footer_area, frame);
}
