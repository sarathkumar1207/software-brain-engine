use crate::diff::{changed_files, changed_files_since};
use crate::status::GitChangedFile;
use sbe_common::{IndexSnapshot, Symbol};
use sbe_graph::SemanticGraph;
use sbe_impact::ImpactAnalyzer;
use sbe_storage::Store;
use sbe_symbols::SymbolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitImpactReport {
    pub changed_files: Vec<GitChangedFile>,
    pub changed_symbols: Vec<Symbol>,
    pub impacted_symbols: Vec<Symbol>,
}

pub fn analyze_git_diff(
    repo_root: impl Into<PathBuf>,
    base: Option<&str>,
) -> anyhow::Result<GitImpactReport> {
    let repo_root = repo_root.into();
    let changed_files = match base {
        Some(base) => changed_files_since(&repo_root, base)?,
        None => changed_files(&repo_root)?,
    };
    let store = Store::open_existing(&repo_root)?;
    let snapshot = store.read_snapshot()?;
    Ok(analyze_snapshot(snapshot, changed_files))
}

fn analyze_snapshot(
    snapshot: IndexSnapshot,
    changed_files: Vec<GitChangedFile>,
) -> GitImpactReport {
    let changed_path_set: HashSet<&str> = changed_files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    let changed_file_ids: HashSet<u64> = snapshot
        .files
        .iter()
        .filter(|file| changed_path_set.contains(file.relative_path.as_str()))
        .map(|file| file.id)
        .collect();
    let changed_symbols: Vec<Symbol> = snapshot
        .symbols
        .iter()
        .filter(|symbol| changed_file_ids.contains(&symbol.file_id))
        .cloned()
        .collect();

    let registry = SymbolRegistry::build(snapshot.symbols.clone());
    let graph = SemanticGraph::from_snapshot(&snapshot);
    let analyzer = ImpactAnalyzer::new(&graph, &registry);
    let mut impacted_by_id = HashMap::new();
    for symbol in &changed_symbols {
        for impacted in analyzer.analyze(symbol.id).affected {
            impacted_by_id.insert(impacted.id, impacted);
        }
    }
    let mut impacted_symbols: Vec<Symbol> = impacted_by_id.into_values().collect();
    impacted_symbols.sort_by_key(|symbol| (symbol.file_id, symbol.range.start_line, symbol.id));

    GitImpactReport {
        changed_files,
        changed_symbols,
        impacted_symbols,
    }
}
