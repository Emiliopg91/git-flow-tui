use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

use crate::{
    git::GitWrapper,
    logic::feature_start,
    ui::{AppState, UiIface},
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

pub struct FeatureStart {
    name: String,
    state: StartProcState,
    messages: Arc<Mutex<Vec<String>>>,
    worker: Option<JoinHandle<()>>,
    rx: Option<mpsc::Receiver<String>>,
    tx: Option<mpsc::Sender<String>>,
}

impl FeatureStart {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            state: StartProcState::EnterName,
            messages: Arc::new(Mutex::new(Vec::new())),
            worker: None,
            rx: None,
            tx: None,
        }
    }
}

impl UiIface for FeatureStart {
    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame) {
        self.set_text("Start feature".to_string(), header, frame);

        let messages = self.messages.lock().unwrap();

        let text = messages
            .iter()
            .skip(messages.len().saturating_sub(body.height as usize))
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        drop(messages);

        let mut final_text = text;

        match self.state {
            StartProcState::EnterName => {
                if final_text.trim().len() > 0 {
                    final_text.push_str("\n");
                }

                final_text.push_str(&format!("Enter feature name: {}_", self.name));

                let footer_txt = if self.name.len() > 4 {
                    format!("Enter: continue | Esc: back")
                } else {
                    "Esc: back".to_string()
                };

                self.set_text(footer_txt, footer, frame);
            }
            StartProcState::Creating => {
                let mut messages = self.messages.lock().unwrap();
                if let Some(rx) = &self.rx {
                    while let Ok(msg) = rx.try_recv() {
                        messages.push(msg.clone());
                        final_text.push_str(&format!("\n{msg}"));
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
            StartProcState::Finished => {
                self.set_text("Esc: back".to_string(), footer, frame);
            }
        }

        let widget = Paragraph::new(final_text);
        frame.render_widget(widget, body);
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppState> {
        match key {
            KeyCode::Esc => {
                if self.state == StartProcState::EnterName || self.state == StartProcState::Finished
                {
                    return Some(AppState::FeatureList);
                }
            }

            KeyCode::Enter => {
                if self.state == StartProcState::EnterName {
                    let features = GitWrapper::global().lock().unwrap().get_features().unwrap();

                    if features.contains(&self.name) {
                        let mut msgs = self.messages.lock().unwrap();

                        let msg = "Feature already exists".to_string();
                        if !msgs.contains(&msg) {
                            msgs.push(msg);
                        }

                        self.name.clear();
                    } else {
                        let (tx, rx) = mpsc::channel();
                        let tx_err = tx.clone();

                        self.tx = Some(tx.clone());
                        self.rx = Some(rx);

                        let name = self.name.clone();
                        self.worker = Some(thread::spawn(move || match feature_start(&name, tx) {
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
                        self.name.push(c);
                    }
                }
            }

            KeyCode::Backspace => {
                if self.state == StartProcState::EnterName {
                    if self.name.len() > 0 {
                        self.name.pop();
                    }
                }
            }

            _ => {}
        }

        None
    }
}
