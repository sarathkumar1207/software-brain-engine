use crate::events::{WatchChange, WatchChangeKind};
use crate::incremental::WatchIndexer;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sbe_events::EventBus;
use sbe_scanner::is_ignored_path;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub fn run_watch(root: impl Into<PathBuf>) -> anyhow::Result<()> {
    let root = root.into();
    let bus = EventBus::new();
    let mut indexer = WatchIndexer::new(&root, bus)?;
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&root, RecursiveMode::Recursive)?;

    println!("[WATCH]");
    println!("Watching {}", root.display());

    for event in rx {
        let event = match event {
            Ok(event) => event,
            Err(error) => {
                eprintln!("watch error: {error}");
                continue;
            }
        };
        let changes = event_to_changes(&root, event);
        if changes.is_empty() {
            continue;
        }
        match indexer.apply(&changes) {
            Ok(report) => print_report(&report),
            Err(error) => eprintln!("update failed: {error}"),
        }
    }
    Ok(())
}

fn event_to_changes(root: &Path, event: notify::Event) -> Vec<WatchChange> {
    let kind = match event.kind {
        EventKind::Create(_) => WatchChangeKind::Created,
        EventKind::Modify(notify::event::ModifyKind::Name(_)) if event.paths.len() >= 2 => {
            return vec![WatchChange {
                path: normalize(root, &event.paths[1]),
                kind: WatchChangeKind::Renamed {
                    from: normalize(root, &event.paths[0]),
                },
            }];
        }
        EventKind::Modify(_) => WatchChangeKind::Modified,
        EventKind::Remove(_) => WatchChangeKind::Deleted,
        _ => return Vec::new(),
    };

    event
        .paths
        .into_iter()
        .filter(|path| !is_ignored_path(path))
        .filter(|path| path.is_file() || matches!(kind, WatchChangeKind::Deleted))
        .map(|path| WatchChange {
            path: normalize(root, &path),
            kind: kind.clone(),
        })
        .collect()
}

fn normalize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.strip_prefix(root).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn print_report(report: &sbe_indexer::UpdateReport) {
    println!();
    println!("[WATCH]");
    println!();
    println!("Modified:");
    for file in &report.changed_files {
        println!("{file}");
    }
    println!();
    println!("Updated:");
    println!(
        "- {} symbols",
        report.added_symbols + report.modified_symbols + report.removed_symbols
    );
    println!("- graph refreshed");
    println!();
    println!("Duration:");
    println!("{}ms", report.elapsed_ms);
}
