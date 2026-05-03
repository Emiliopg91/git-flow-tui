mod main_menu;

use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode},
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::ui::main_menu::MainMenu;

pub enum AppState {
    MainMenu,
    Feature,
    Release,
    Hotfix,
    Bugfix,
}

impl AppState {
    fn try_from(value: &str) -> Option<Self> {
        match value {
            "MainMenu" => Some(AppState::MainMenu),
            "Feature" => Some(AppState::Feature),
            "Release" => Some(AppState::Release),
            "Hotfix" => Some(AppState::Hotfix),
            "Bugfix" => Some(AppState::Bugfix),
            _ => None,
        }
    }
}

pub trait UiIface {
    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame);
    fn handle_input(&mut self, key: KeyCode) -> Option<AppState>;
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

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &mut app))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match app.state {
                    AppState::MainMenu => {
                        if let Some(val) = app.page.handle_input(key.code) {
                            app.state = val;
                        }
                    }
                    _ => (),
                }
            }
        }
    })
}

fn render(frame: &mut Frame, app: &mut App) {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(3),
        Constraint::Length(1),
    ]);
    let area = frame.area().inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    let [header_area, body, footer_area] = area.layout(&layout);
    match app.state {
        AppState::MainMenu => app.page.render(header_area, body, footer_area, frame),
        _ => (),
    }
}
