//! Historial de consultas persistido en `~/.local/share/tinker-tui/history.json`.

use std::fs;
use std::path::PathBuf;

const MAX_ENTRIES: usize = 200;

pub struct History {
    entries: Vec<String>,
    path: Option<PathBuf>,
}

impl History {
    pub fn load() -> Self {
        let path = dirs::data_dir().map(|d| d.join("tinker-tui").join("history.json"));
        let entries = path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { entries, path }
    }

    /// Lo más reciente primero.
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn push(&mut self, code: &str) {
        let code = code.trim().to_string();
        if code.is_empty() {
            return;
        }
        self.entries.retain(|e| e != &code);
        self.entries.insert(0, code);
        self.entries.truncate(MAX_ENTRIES);
        self.save();
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.save();
        }
    }

    fn save(&self) {
        let Some(path) = &self.path else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.entries) {
            let _ = fs::write(path, json);
        }
    }
}
