use std::{ops::Add, process::exit};

use ratatui::{
    Frame,
    crossterm::{
        cursor::Show,
        event::KeyCode,
        execute,
        terminal::{LeaveAlternateScreen, disable_raw_mode},
    },
    layout::{Alignment, Offset, Rect},
    style::{Color, Modifier, Style},
    widgets::{Paragraph, Tabs},
};

use crate::{
    git::GitWrapper,
    others::exit_code::ExitCode,
    ui::{
        AppState, UiIface, bugfix::BugfixList, feature::FeatureList, hotfix::HotfixList,
        release::ReleaseList,
    },
};

pub struct MainMenu {
    tab: usize,
    page: Box<dyn UiIface>,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            tab: 0,
            page: Box::new(FeatureList::new()),
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
            KeyCode::Left | KeyCode::Right => {
                let old_tab = self.tab;
                if key == KeyCode::Left {
                    self.tab = self.tab.saturating_sub(1);
                } else {
                    self.tab = self.tab.add(1).min(3);
                }

                if old_tab != self.tab {
                    self.page = match self.tab {
                        0 => Box::new(FeatureList::new()),
                        1 => Box::new(ReleaseList::new()),
                        2 => Box::new(BugfixList::new()),
                        3 => Box::new(HotfixList::new()),
                        _ => unreachable!(),
                    }
                }
            }
            _ => return self.page.handle_input(key),
        }

        None
    }

    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame) {
        let tabs = Tabs::new(vec!["Features", "Releases", "Bugfixes", "Hotfixes"])
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .select(self.tab)
            .divider("-")
            .padding(" ", " ");
        frame.render_widget(tabs, body + Offset::new(1, -1));

        self.page.render(header, body, footer, frame);
        {
            let git = GitWrapper::global().lock().unwrap();
            let branch: Paragraph<'_> = Paragraph::new(format!("  {} ", git.get_branch()))
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(branch, footer + Offset::new(0, -1));
        }
        self.set_text("Main Menu".to_string(), header, frame);
        self.set_text(
            "Arrows: navigate | +: create | Del: finish | Esc: exit".to_string(),
            footer,
            frame,
        );
    }
}
