use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchChangeKind {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchChange {
    pub path: PathBuf,
    pub kind: WatchChangeKind,
}
