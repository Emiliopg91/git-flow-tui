use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    git::GitWrapper,
    logic::bugfix::bugfix_start,
    ui::{
        widgets::{
            popup::Popup,
            text_input::{InputState, TextInput},
        },
        AppState, UiIface,
    },
};

use ratatui::{
    crossterm::event::KeyCode,
    prelude::{Frame, Rect},
    widgets::Paragraph,
};

#[derive(PartialEq)]
enum StartProcState {
    EnterName,
    Creating,
    Finished,
}

pub struct BugfixStart {
    name: InputState,
    state: StartProcState,
    messages: Arc<Mutex<Vec<String>>>,
    popup_message: Option<String>,
    worker: Option<JoinHandle<()>>,
    rx: Option<mpsc::Receiver<String>>,
    tx: Option<mpsc::Sender<String>>,
}

impl BugfixStart {
    pub fn new() -> Self {
        Self {
            name: InputState::new(&"".to_string()),
            state: StartProcState::EnterName,
            messages: Arc::new(Mutex::new(Vec::new())),
            popup_message: None,
            worker: None,
            rx: None,
            tx: None,
        }
    }
}

impl UiIface for BugfixStart {
    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame) {
        self.set_text("Start bugfix".to_string(), header, frame);

        let messages = self.messages.lock().unwrap();

        let text = messages
            .iter()
            .skip(messages.len().saturating_sub(body.height as usize))
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        drop(messages);

        match self.state {
            StartProcState::EnterName => {
                let footer_txt = if self.name.value.len() > 4 {
                    format!("Enter: continue | Esc: back")
                } else {
                    "Esc: back".to_string()
                };

                self.set_text(footer_txt, footer, frame);
            }
            StartProcState::Creating => {}
            StartProcState::Finished => {
                self.set_text("Esc: back".to_string(), footer, frame);
            }
        }

        let widget = Paragraph::new(text);
        frame.render_widget(widget, body);

        if self.state == StartProcState::EnterName {
            let text_input = TextInput::new(&"Enter bugfix name".to_string());
            frame.render_stateful_widget(text_input, body, &mut self.name);

            if let Some(popup_message) = &self.popup_message {
                let popup = Popup::new(&"Error".to_string(), popup_message);
                frame.render_widget(popup, body);
            }
        }
    }

    fn tick(&mut self) {
        if self.state == StartProcState::Creating {
            let mut messages = self.messages.lock().unwrap();
            if let Some(rx) = &self.rx {
                while let Ok(msg) = rx.try_recv() {
                    messages.push(msg.clone());
                }
            }
            drop(messages);

            let finished = self
                .worker
                .as_ref()
                .map(|t| t.is_finished())
                .unwrap_or(false);
            if finished {
                self.state = StartProcState::Finished
            }
        }
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
                        return Some(AppState::BugfixList);
                    }
                }
            }

            KeyCode::Enter => {
                if self.state == StartProcState::EnterName {
                    let git = GitWrapper::global().lock().unwrap();
                    let bugfixes = git.get_bugfixes().unwrap();
                    let rem_bugfixes = git.get_remote_bugfixes().unwrap();
                    drop(git);

                    if bugfixes.contains(&self.name.value)
                        || rem_bugfixes.contains(&self.name.value)
                    {
                        self.popup_message = Some("Bugfix already exists".to_string());
                        self.name.value.clear();
                    } else {
                        let (tx, rx) = mpsc::channel();
                        let tx_err = tx.clone();

                        self.tx = Some(tx.clone());
                        self.rx = Some(rx);

                        let name = self.name.value.clone();
                        self.worker = Some(thread::spawn(move || match bugfix_start(&name, tx) {
                            Err(e) => tx_err.send(format!("{}", e)).unwrap(),
                            Ok(()) => (),
                        }));

                        self.state = StartProcState::Creating;
                    }
                }
            }

            KeyCode::Char(c) => {
                if self.state == StartProcState::EnterName {
                    if c.is_ascii_alphanumeric() || c == '-' {
                        self.name.value.push(c);
                    }
                }
            }

            KeyCode::Backspace => {
                if self.state == StartProcState::EnterName {
                    if self.name.value.len() > 0 {
                        self.name.value.pop();
                    }
                }
            }

            _ => {}
        }

        None
    }
}
