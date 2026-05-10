use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

use crate::{
    git::GitWrapper,
    logic::hotfix::hotfix_start,
    ui::{
        AppState, UiIface,
        widgets::{
            popup::Popup,
            text_input::{InputState, TextInput},
        },
    },
};

use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    prelude::{Frame, Rect},
    widgets::Paragraph,
};

#[derive(PartialEq)]
enum StartProcState {
    Fetching,
    EnterName,
    Creating,
    Finished,
}

pub struct HotfixStart {
    name: InputState,
    state: StartProcState,
    messages: Arc<Mutex<Vec<String>>>,
    popup_message: Option<String>,
    worker: Option<JoinHandle<()>>,
    rx: Option<mpsc::Receiver<String>>,
    tx: Option<mpsc::Sender<String>>,
}

impl HotfixStart {
    pub fn new() -> Self {
        Self {
            name: InputState::new(""),
            state: StartProcState::Fetching,
            messages: Arc::new(Mutex::new(
                ["Fetching from remotes...".to_string()].to_vec(),
            )),
            popup_message: None,
            worker: None,
            rx: None,
            tx: None,
        }
    }
}

impl UiIface for HotfixStart {
    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame) {
        self.set_text("Start hotfix".to_string(), header, frame);

        let messages = self.messages.lock().unwrap();

        let text = messages
            .iter()
            .skip(messages.len().saturating_sub(body.height as usize))
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let size = messages.len();

        drop(messages);

        let layout = Layout::vertical([Constraint::Min(size as u16), Constraint::Min(0)]);
        let [msg_area, input_area] = body.layout(&layout);

        match self.state {
            StartProcState::EnterName => {
                let footer_txt = if self.name.value.len() > 4 {
                    "Enter: continue | Esc: back".to_string()
                } else {
                    "Esc: back".to_string()
                };

                self.set_text(footer_txt, footer, frame);
            }
            StartProcState::Finished => {
                self.set_text("Esc: back".to_string(), footer, frame);
            }
            _ => (),
        }

        let widget = Paragraph::new(text);
        frame.render_widget(widget, msg_area);

        if self.state == StartProcState::EnterName {
            let text_input = TextInput::new("Enter hotfix name");
            frame.render_stateful_widget(text_input, input_area, &mut self.name);

            if let Some(popup_message) = &self.popup_message {
                let popup = Popup::new("Error", popup_message);
                frame.render_widget(popup, body);
            }
        }
    }

    fn tick(&mut self) {
        let mut messages = self.messages.lock().unwrap();
        match self.state {
            StartProcState::Fetching => {
                if let Ok(git) = GitWrapper::global().lock() {
                    match git.fetch(true, true) {
                        Ok(_) => {
                            self.state = StartProcState::EnterName;
                        }
                        Err(e) => {
                            messages.push(format!("{}", e));
                            self.state = StartProcState::Finished;
                        }
                    }
                    return;
                }
            }
            StartProcState::Creating => {
                if let Some(rx) = &self.rx {
                    while let Ok(msg) = rx.try_recv() {
                        messages.push(msg.clone());
                    }
                }

                let finished = self
                    .worker
                    .as_ref()
                    .map(|t| t.is_finished())
                    .unwrap_or(false);
                if finished {
                    self.state = StartProcState::Finished
                }
            }
            _ => {}
        }
        drop(messages);
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppState> {
        match key {
            KeyCode::Esc => {
                if self.popup_message.is_some() {
                    self.popup_message = None;
                } else {
                    if self.state == StartProcState::EnterName
                        || self.state == StartProcState::Finished
                    {
                        return Some(AppState::MainMenu);
                    }
                }
            }

            KeyCode::Enter if self.state == StartProcState::EnterName => {
                let git = GitWrapper::global().lock().unwrap();
                let hotfixes = git.get_hotfixes().unwrap();
                let rem_hotfixes = git.get_remote_hotfixes().unwrap();
                drop(git);

                if hotfixes.contains(&self.name.value) || rem_hotfixes.contains(&self.name.value) {
                    self.popup_message = Some("Hotfix already exists".to_string());
                    self.name.value.clear();
                } else {
                    let (tx, rx) = mpsc::channel();
                    let tx_err = tx.clone();

                    self.tx = Some(tx.clone());
                    self.rx = Some(rx);

                    let name = self.name.value.clone();
                    self.worker = Some(thread::spawn(move || {
                        if let Err(e) = hotfix_start(&name, tx) {
                            tx_err.send(format!("{}", e)).unwrap()
                        }
                    }));

                    self.state = StartProcState::Creating;
                }
            }

            KeyCode::Char(c)
                if self.state == StartProcState::EnterName
                    && (c.is_ascii_alphanumeric() || c == '-') =>
            {
                self.name.value.push(c);
            }

            KeyCode::Backspace
                if self.state == StartProcState::EnterName && !self.name.value.is_empty() =>
            {
                self.name.value.pop();
            }

            _ => {}
        }

        None
    }
}
