use crate::path::TracePath;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceJson {
    pub root: String,
    pub path: Vec<String>,
}

impl From<TracePath> for TraceJson {
    fn from(path: TracePath) -> Self {
        Self {
            root: path.root,
            path: path.path,
        }
    }
}

pub fn format_text(path: &TracePath) -> String {
    let mut lines = Vec::new();
    lines.push(path.root.clone());
    for (index, node) in path.path.iter().enumerate() {
        let indent = "    ".repeat(index);
        lines.push(format!("{indent} └─ {node}"));
    }
    lines.join("\n")
}
