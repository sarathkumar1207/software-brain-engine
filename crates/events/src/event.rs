use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileEventKind {
    Created,
    Modified,
    Deleted,
    Renamed { from: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineEvent {
    FileChanged {
        path: String,
        kind: FileEventKind,
    },
    FileDeleted {
        path: String,
    },
    FileCreated {
        path: String,
    },
    GraphUpdated {
        changed_files: Vec<String>,
        edges: usize,
    },
    IndexUpdated {
        changed_files: Vec<String>,
        symbols: usize,
    },
    ImpactUpdated {
        symbols: Vec<u64>,
    },
}
