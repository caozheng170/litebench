use crate::types::BenchResult;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    /// Current phase id: hardware | cpu | memory | disk | scoring | done
    pub phase: String,
    /// Human-readable label for the current phase.
    pub label: String,
    /// 0.0 .. 1.0
    pub progress: f64,
    pub done: bool,
}

impl Default for Progress {
    fn default() -> Self {
        Progress {
            phase: "starting".to_string(),
            label: "准备中…".to_string(),
            progress: 0.0,
            done: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct AppState {
    pub progress: Progress,
    pub result: Option<BenchResult>,
    /// True while a benchmark thread is active.
    pub running: bool,
}

impl AppState {
    pub fn set_phase(&mut self, phase: &str, label: &str, progress: f64) {
        self.progress.phase = phase.to_string();
        self.progress.label = label.to_string();
        self.progress.progress = progress;
    }
}
