mod main_menu;

use std::path::Path;

use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode},
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::{git::GitWrapper, ui::main_menu::MainMenu};

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
    // layout vertical
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(3),
        Constraint::Length(1),
    ]);

    let area = frame.area().inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let [header_area, body_area, footer_area] = area.layout(&layout);

    // layout horizontal para centrar
    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Percentage(75),
        Constraint::Fill(1),
    ]);

    let [_, centered_body, _] = body_area.layout(&horizontal);

    // 👇 crear bloque con borde
    let block = Block::default().borders(Borders::ALL);

    // 👇 renderizar el borde
    frame.render_widget(block, centered_body);

    // 👇 obtener el área interna del bloque (sin bordes)
    let inner = centered_body.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    // pasar área interna a tu render
    match app.state {
        AppState::MainMenu => app.page.render(header_area, inner, footer_area, frame),
        _ => (),
    }
}
