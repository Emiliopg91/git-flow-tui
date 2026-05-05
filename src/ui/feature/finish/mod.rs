use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

use crate::{
    logic::feature_finish,
    others::whiteboard::WHITEBOARD,
    ui::{AppState, UiIface},
};

use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Layout},
    prelude::{Frame, Rect},
    style::{Color, Modifier},
    widgets::{List, ListState, Paragraph},
};

#[derive(PartialEq)]
enum FinishProcState {
    Confirm,
    Finishing,
    Finished,
}

pub struct FeatureFinish {
    name: String,
    entry: ListState,
    state: FinishProcState,
    messages: Arc<Mutex<Vec<String>>>,
    worker: Option<JoinHandle<()>>,
    rx: Option<mpsc::Receiver<String>>,
    tx: Option<mpsc::Sender<String>>,
}

impl FeatureFinish {
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
            entry: ListState::default().with_selected(Some(0)),
            state: FinishProcState::Confirm,
            messages: Arc::new(Mutex::new(Vec::new())),
            worker: None,
            rx: None,
            tx: None,
        }
    }
}

impl UiIface for FeatureFinish {
    fn render(&mut self, header: Rect, body: Rect, footer: Rect, frame: &mut Frame) {
        self.set_text("Finish feature".to_string(), header, frame);

        let mut messages = self.messages.lock().unwrap();
        let text = messages
            .iter()
            .skip(messages.len().saturating_sub(body.height as usize))
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        drop(messages);

        match self.state {
            FinishProcState::Confirm => {
                let layout_fl = Layout::vertical([Constraint::Length(1), Constraint::Length(2)]);
                let [question_area, answer_area] = body.layout(&layout_fl);

                let question_widget = Paragraph::new(format!(
                    "Are you sure you want to finish '{}' feature?",
                    self.name
                ));
                frame.render_widget(question_widget, question_area);

                let list = List::new(["No".to_string(), "Yes".to_string()])
                    .style(Color::White)
                    .highlight_style(Modifier::REVERSED)
                    .highlight_symbol(" ");

                frame.render_stateful_widget(list, answer_area, &mut self.entry);

                self.set_text(
                    "Up/Down: navigate | Enter: select | Esc: back".to_string(),
                    footer,
                    frame,
                );
            }
            FinishProcState::Finishing => {
                if let Some(rx) = &self.rx {
                    let mut messages = self.messages.lock().unwrap();
                    while let Ok(msg) = rx.try_recv() {
                        messages.push(msg.clone());
                    }
                    drop(messages);
                }

                let widget = Paragraph::new(text);
                frame.render_widget(widget, body);

                let finished = self
                    .worker
                    .as_ref()
                    .map(|t| t.is_finished())
                    .unwrap_or(false);
                if finished {
                    self.state = FinishProcState::Finished
                }
            }
            FinishProcState::Finished => {
                let widget = Paragraph::new(text);
                frame.render_widget(widget, body);
                self.set_text("Esc: back".to_string(), footer, frame);
            }
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppState> {
        match key {
            KeyCode::Esc => {
                if self.state != FinishProcState::Finishing {
                    return Some(AppState::FeatureList);
                }
            }

            KeyCode::Enter => {
                if self.state == FinishProcState::Confirm {
                    if let Some(selected) = self.entry.selected() {
                        if selected == 0 {
                            return Some(AppState::FeatureList);
                        } else {
                            let (tx, rx) = mpsc::channel();
                            let tx_err = tx.clone();

                            self.tx = Some(tx.clone());
                            self.rx = Some(rx);

                            let name = self.name.clone();
                            self.worker =
                                Some(thread::spawn(move || match feature_finish(&name, tx) {
                                    Err(e) => tx_err.send(format!("{}", e)).unwrap(),
                                    Ok(()) => (),
                                }));

                            self.state = FinishProcState::Finishing;
                        }
                    }
                }
            }
            KeyCode::Up => {
                if self.state == FinishProcState::Confirm {
                    self.entry.select_previous()
                }
            }
            KeyCode::Down => {
                if self.state == FinishProcState::Confirm {
                    self.entry.select_next()
                }
            }

            _ => {}
        }

        None
    }
}
