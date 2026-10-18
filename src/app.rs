//! Estado de la aplicación y manejo de teclas.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::{ListState, TableState};
use ratatui_textarea::{Input, TextArea};

use crate::history::History;
use crate::runner::{self, RunResult, Target};
use crate::table::{self, TableData};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Editor,
    History,
    Output,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    Table,
    Raw,
}

pub enum Status {
    Idle,
    Running(Instant),
    Done { success: bool, elapsed: Duration },
}

pub struct App<'a> {
    pub target: Target,
    pub editor: TextArea<'a>,
    pub history: History,
    pub history_state: ListState,
    pub focus: Focus,
    pub output: String,
    pub output_scroll: u16,
    /// Tabla construida desde el JSON del resultado; `None` si era un escalar.
    pub table: Option<TableData>,
    pub table_state: TableState,
    /// Primera columna visible (scroll horizontal con h/l).
    pub col_offset: usize,
    pub view: View,
    pub status: Status,
    /// `main` abre el editor externo cuando ve este flag (necesita la terminal).
    pub wants_external_editor: bool,
    pub quit: bool,
    pending: Option<Receiver<RunResult>>,
}

impl App<'_> {
    pub fn new(target: Target) -> Self {
        let history = History::load();
        let mut history_state = ListState::default();
        if !history.entries().is_empty() {
            history_state.select(Some(0));
        }
        Self {
            target,
            editor: TextArea::default(),
            history,
            history_state,
            focus: Focus::Editor,
            output: String::new(),
            output_scroll: 0,
            table: None,
            table_state: TableState::default(),
            col_offset: 0,
            view: View::Raw,
            status: Status::Idle,
            wants_external_editor: false,
            quit: false,
            pending: None,
        }
    }

    pub fn code(&self) -> String {
        self.editor.lines().join("\n")
    }

    pub fn set_code(&mut self, code: &str) {
        self.editor = TextArea::from(code.lines());
    }

    pub fn is_running(&self) -> bool {
        self.pending.is_some()
    }

    pub fn run(&mut self) {
        let code = self.code();
        if self.is_running() || code.trim().is_empty() {
            return;
        }
        self.history.push(&code);
        self.history_state.select(Some(0));
        self.pending = Some(runner::spawn(&self.target, &code));
        self.status = Status::Running(Instant::now());
    }

    /// Revisa si el hilo de ejecución ya respondió. Se llama en cada tick.
    pub fn poll_runner(&mut self) {
        let Some(rx) = &self.pending else { return };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => RunResult {
                output: "El hilo de ejecución terminó sin responder".into(),
                data: None,
                success: false,
                elapsed: Duration::ZERO,
            },
        };
        self.pending = None;
        self.output = result.output;
        self.output_scroll = 0;
        self.table = result.data.as_ref().and_then(table::from_json);
        self.view = if self.table.is_some() {
            View::Table
        } else {
            View::Raw
        };
        self.table_state = TableState::default().with_selected(Some(0));
        self.col_offset = 0;
        self.status = Status::Done {
            success: result.success,
            elapsed: result.elapsed,
        };
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Atajos globales, válidos en cualquier panel.
        match key.code {
            KeyCode::Char('c') if ctrl => return self.quit = true,
            KeyCode::Char('r') if ctrl => return self.run(),
            KeyCode::F(5) => return self.run(),
            KeyCode::Char('e') if ctrl => return self.wants_external_editor = true,
            KeyCode::Char('l') if ctrl => {
                self.output.clear();
                self.table = None;
                self.view = View::Raw;
                self.status = Status::Idle;
                return;
            }
            _ => {}
        }

        match self.focus {
            Focus::Editor => self.on_editor_key(key),
            Focus::History => self.on_history_key(key),
            Focus::Output => self.on_output_key(key),
        }
    }

    fn on_editor_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.focus = Focus::History;
            return;
        }
        self.editor.input(Input::from(Event::Key(key)));
    }

    fn on_history_key(&mut self, key: KeyEvent) {
        let len = self.history.entries().len();
        let selected = self.history_state.selected();
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('i') => self.focus = Focus::Editor,
            KeyCode::Tab => self.focus = Focus::Output,
            KeyCode::Char('j') | KeyCode::Down if len > 0 => {
                let next = selected.map_or(0, |i| (i + 1).min(len - 1));
                self.history_state.select(Some(next));
            }
            KeyCode::Char('k') | KeyCode::Up if len > 0 => {
                let prev = selected.map_or(0, |i| i.saturating_sub(1));
                self.history_state.select(Some(prev));
            }
            KeyCode::Enter => {
                if let Some(code) = selected
                    .and_then(|i| self.history.entries().get(i))
                    .cloned()
                {
                    self.set_code(&code);
                    self.focus = Focus::Editor;
                }
            }
            KeyCode::Char('d') => {
                if let Some(i) = selected {
                    self.history.remove(i);
                    let len = self.history.entries().len();
                    self.history_state.select((len > 0).then(|| i.min(len - 1)));
                }
            }
            _ => {}
        }
    }

    fn on_output_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => return self.quit = true,
            KeyCode::Char('i') => return self.focus = Focus::Editor,
            KeyCode::Tab => return self.focus = Focus::History,
            KeyCode::Char('t') if self.table.is_some() => {
                self.view = match self.view {
                    View::Table => View::Raw,
                    View::Raw => View::Table,
                };
                return;
            }
            _ => {}
        }
        let dims = self.table.as_ref().map(|t| (t.rows.len(), t.headers.len()));
        match (self.view, dims) {
            (View::Table, Some((rows, cols))) => self.on_table_key(key, rows, cols),
            _ => self.on_raw_key(key),
        }
    }

    fn on_table_key(&mut self, key: KeyEvent, rows: usize, cols: usize) {
        let last_row = rows.saturating_sub(1);
        let row = self.table_state.selected().unwrap_or(0);
        let select = |state: &mut TableState, r: usize| state.select(Some(r.min(last_row)));
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => select(&mut self.table_state, row + 1),
            KeyCode::Char('k') | KeyCode::Up => {
                select(&mut self.table_state, row.saturating_sub(1))
            }
            KeyCode::Char('d') | KeyCode::PageDown => select(&mut self.table_state, row + 10),
            KeyCode::Char('u') | KeyCode::PageUp => {
                select(&mut self.table_state, row.saturating_sub(10))
            }
            KeyCode::Char('g') => select(&mut self.table_state, 0),
            KeyCode::Char('G') => select(&mut self.table_state, last_row),
            KeyCode::Char('l') | KeyCode::Right => {
                self.col_offset = (self.col_offset + 1).min(cols.saturating_sub(1))
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.col_offset = self.col_offset.saturating_sub(1)
            }
            KeyCode::Char('0') => self.col_offset = 0,
            KeyCode::Char('$') => self.col_offset = cols.saturating_sub(1),
            _ => {}
        }
    }

    fn on_raw_key(&mut self, key: KeyEvent) {
        let max = self.output.lines().count().saturating_sub(1) as u16;
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.output_scroll = (self.output_scroll + 1).min(max)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.output_scroll = self.output_scroll.saturating_sub(1)
            }
            KeyCode::Char('d') | KeyCode::PageDown => {
                self.output_scroll = (self.output_scroll + 10).min(max)
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                self.output_scroll = self.output_scroll.saturating_sub(10)
            }
            KeyCode::Char('g') => self.output_scroll = 0,
            KeyCode::Char('G') => self.output_scroll = max,
            _ => {}
        }
    }
}
