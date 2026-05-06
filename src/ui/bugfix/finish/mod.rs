use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    logic::bugfix::bugfix_finish,
    others::whiteboard::WHITEBOARD,
    ui::{
        widgets::multi_choice::{MultiChoice, MultiChoiceState},
        AppState, UiIface,
    },
};

use ratatui::{
    crossterm::event::KeyCode,
    prelude::{Frame, Rect},
    widgets::Paragraph,
};

#[derive(PartialEq)]
enum FinishProcState {
    Confirm,
    Finishing,
    Finished,
}

pub struct BugfixFinish {
    name: String,
    entry: MultiChoiceState,
    state: FinishProcState,
    messages: Arc<Mutex<Vec<String>>>,
    worker: Option<JoinHandle<()>>,
    rx: Option<mpsc::Receiver<String>>,
    tx: Option<mpsc::Sender<String>>,
}

impl BugfixFinish {
    pub fn new() -> Self {
        let branch = WHITEBOARD
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .get("branch")
            .cloned()
            .unwrap();
        Self {
            name: branch,
            entry: MultiChoiceState::new(),
            state: FinishProcState::Confirm,
            messages: Arc::new(Mutex::new(Vec::new())),
            worker: None,
            rx: None,
            tx: None,
        }
    }
}

impl UiIface for BugfixFinish {
    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame) {
        self.set_text("Finish bugfix".to_string(), header, frame);

        let messages = self.messages.lock().unwrap();
        let text = messages
            .iter()
            .skip(messages.len().saturating_sub(body.height as usize))
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        drop(messages);

        match self.state {
            FinishProcState::Confirm => {
                let widget = MultiChoice::new(
                    &format!("Are you sure you want to finish '{}' bugfix?", self.name),
                    ["No".to_string(), "Yes".to_string()].to_vec(),
                );

                frame.render_stateful_widget(widget, body, &mut self.entry);

                self.set_text(
                    "Left/Right: navigate | Enter: select | Esc: back".to_string(),
                    footer,
                    frame,
                );
            }
            FinishProcState::Finishing => {
                let widget = Paragraph::new(text);
                frame.render_widget(widget, body);
            }
            FinishProcState::Finished => {
                let widget = Paragraph::new(text);
                frame.render_widget(widget, body);
                self.set_text("Esc: back".to_string(), footer, frame);
            }
        }
    }

    fn tick(&mut self) {
        if self.state == FinishProcState::Finishing {
            if let Some(rx) = &self.rx {
                let mut messages = self.messages.lock().unwrap();
                while let Ok(msg) = rx.try_recv() {
                    messages.push(msg.clone());
                }
                drop(messages);
            }

            let finished = self
                .worker
                .as_ref()
                .map(|t| t.is_finished())
                .unwrap_or(false);
            if finished {
                self.state = FinishProcState::Finished
            }
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppState> {
        match key {
            KeyCode::Esc
                if self.state != FinishProcState::Finishing => {
                    return Some(AppState::BugfixList);
                }

            KeyCode::Enter
                if self.state == FinishProcState::Confirm => {
                    let selected = self.entry.selected();
                    if selected == 0 {
                        return Some(AppState::BugfixList);
                    } else {
                        let (tx, rx) = mpsc::channel();
                        let tx_err = tx.clone();

                        self.tx = Some(tx.clone());
                        self.rx = Some(rx);

                        let name = self.name.clone();
                        self.worker = Some(thread::spawn(move || if let Err(e) = bugfix_finish(&name, tx) { tx_err.send(format!("{}", e)).unwrap() }));

                        self.state = FinishProcState::Finishing;
                    }
                }
            KeyCode::Left
                if self.state == FinishProcState::Confirm => {
                    self.entry.select_previous()
                }
            KeyCode::Right
                if self.state == FinishProcState::Confirm => {
                    self.entry.select_next()
                }

            _ => {}
        }

        None
    }
}
