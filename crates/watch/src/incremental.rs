use crate::events::{WatchChange, WatchChangeKind};
use sbe_events::{EngineEvent, EventBus, FileEventKind};
use sbe_indexer::{Indexer, UpdateReport};
use std::path::PathBuf;

pub struct WatchIndexer {
    indexer: Indexer,
    bus: EventBus,
}

impl WatchIndexer {
    pub fn new(root: impl Into<PathBuf>, bus: EventBus) -> anyhow::Result<Self> {
        Ok(Self {
            indexer: Indexer::new(root)?,
            bus,
        })
    }

    pub fn apply(&mut self, changes: &[WatchChange]) -> anyhow::Result<UpdateReport> {
        let mut paths = Vec::new();
        for change in changes {
            paths.push(change.path.clone());
            if let WatchChangeKind::Renamed { from } = &change.kind {
                paths.push(from.clone());
            }
            self.bus.publish(EngineEvent::FileChanged {
                path: change.path.display().to_string(),
                kind: file_event_kind(&change.kind),
            });
        }

        let report = self.indexer.update_paths(paths)?;
        self.bus.publish(EngineEvent::IndexUpdated {
            changed_files: report.changed_files.clone(),
            symbols: report.added_symbols + report.modified_symbols,
        });
        self.bus.publish(EngineEvent::GraphUpdated {
            changed_files: report.changed_files.clone(),
            edges: report.edges_found,
        });
        self.bus.publish(EngineEvent::ImpactUpdated {
            symbols: report.affected_symbols.clone(),
        });
        Ok(report)
    }
}

fn file_event_kind(kind: &WatchChangeKind) -> FileEventKind {
    match kind {
        WatchChangeKind::Created => FileEventKind::Created,
        WatchChangeKind::Modified => FileEventKind::Modified,
        WatchChangeKind::Deleted => FileEventKind::Deleted,
        WatchChangeKind::Renamed { from } => FileEventKind::Renamed {
            from: from.display().to_string(),
        },
    }
}
