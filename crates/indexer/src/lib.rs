use rayon::prelude::*;
use sbe_common::{Edge, FileEntry, IndexSnapshot, ParsedFile, RelationType, Symbol};
use sbe_graph::{GraphDiff, SemanticGraph};
use sbe_parser::TypeScriptParser;
use sbe_scanner::Scanner;
use sbe_storage::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexReport {
    pub files_scanned: usize,
    pub symbols_found: usize,
    pub imports_found: usize,
    pub edges_found: usize,
    pub storage_path: String,
    pub index_size_bytes: u64,
    pub bytes_read: u64,
    pub elapsed_ms: u128,
    pub skipped_dirs: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub path: String,
    pub initialized: bool,
    pub has_index: bool,
    pub storage_version: Option<u32>,
    pub indexed_files: usize,
    pub stale_files: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateReport {
    pub changed_files: Vec<String>,
    pub added_symbols: usize,
    pub modified_symbols: usize,
    pub removed_symbols: usize,
    pub affected_symbols: Vec<u64>,
    pub affected_files: usize,
    pub edges_found: usize,
    pub storage_path: String,
    pub index_size_bytes: u64,
    pub elapsed_ms: u128,
    pub warnings: Vec<String>,
}

pub struct Indexer {
    root: PathBuf,
    store: Store,
    scanner: Scanner,
}

impl Indexer {
    pub fn new(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let root = root.into();
        Ok(Self {
            store: Store::open_or_create(&root)?,
            scanner: Scanner::new(&root),
            root,
        })
    }

    pub fn init(root: impl AsRef<Path>) -> anyhow::Result<Store> {
        Store::open_or_create(root)
    }

    pub fn run(&mut self) -> anyhow::Result<IndexReport> {
        let started = std::time::Instant::now();
        let scan = self.scanner.scan_with_report()?;
        let mut warnings = scan.warnings.clone();

        let parsed_results: Vec<anyhow::Result<ParsedFile>> = scan
            .files
            .par_iter()
            .map(|file| {
                let source = std::fs::read_to_string(&file.path)
                    .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", file.path))?;
                let mut parser = TypeScriptParser::new();
                parser
                    .parse_file(file, &source)
                    .map_err(|error| anyhow::anyhow!("failed to parse {}: {error}", file.path))
            })
            .collect();

        let mut parsed_files = Vec::new();
        for result in parsed_results {
            match result {
                Ok(parsed) => parsed_files.push(parsed),
                Err(error) => warnings.push(error.to_string()),
            }
        }

        let (snapshot, import_warnings) = build_snapshot(&self.root, scan.files, parsed_files);
        warnings.extend(import_warnings);
        let report = IndexReport {
            files_scanned: snapshot.files.len(),
            symbols_found: snapshot.symbols.len(),
            imports_found: snapshot.imports.len(),
            edges_found: snapshot.edges.len(),
            storage_path: self.store.root().display().to_string(),
            index_size_bytes: 0,
            bytes_read: scan.bytes_read,
            elapsed_ms: started.elapsed().as_millis(),
            skipped_dirs: scan.skipped_dirs,
            warnings,
        };
        self.store.write_snapshot(&snapshot)?;
        Ok(IndexReport {
            index_size_bytes: self.store.index_size().unwrap_or_default(),
            ..report
        })
    }

    pub fn update(&mut self) -> anyhow::Result<UpdateReport> {
        let started = std::time::Instant::now();
        let old_snapshot = self.store.read_snapshot_or_empty(&self.root)?;
        let scan = self.scanner.scan_with_report()?;
        let mut warnings = scan.warnings.clone();
        let changed_paths = changed_paths(&old_snapshot, &scan.files);
        let changed_path_set: std::collections::HashSet<&str> =
            changed_paths.iter().map(String::as_str).collect();

        if changed_paths.is_empty() {
            return Ok(UpdateReport {
                changed_files: Vec::new(),
                added_symbols: 0,
                modified_symbols: 0,
                removed_symbols: 0,
                affected_symbols: Vec::new(),
                affected_files: 0,
                edges_found: old_snapshot.edges.len(),
                storage_path: self.store.root().display().to_string(),
                index_size_bytes: self.store.index_size().unwrap_or_default(),
                elapsed_ms: started.elapsed().as_millis(),
                warnings,
            });
        }

        let files_to_parse: Vec<FileEntry> = scan
            .files
            .iter()
            .filter(|file| changed_path_set.contains(file.relative_path.as_str()))
            .cloned()
            .collect();
        let parsed_results: Vec<anyhow::Result<ParsedFile>> = files_to_parse
            .par_iter()
            .map(|file| {
                let source = std::fs::read_to_string(&file.path)
                    .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", file.path))?;
                let mut parser = TypeScriptParser::new();
                parser
                    .parse_file(file, &source)
                    .map_err(|error| anyhow::anyhow!("failed to parse {}: {error}", file.path))
            })
            .collect();

        let mut parsed_files = Vec::new();
        for result in parsed_results {
            match result {
                Ok(parsed) => parsed_files.push(parsed),
                Err(error) => warnings.push(error.to_string()),
            }
        }

        let old_changed_file_ids: std::collections::HashSet<u64> = old_snapshot
            .files
            .iter()
            .filter(|file| changed_path_set.contains(file.relative_path.as_str()))
            .map(|file| file.id)
            .collect();
        let changed_symbol_ids: std::collections::HashSet<u64> = old_snapshot
            .symbols
            .iter()
            .filter(|symbol| old_changed_file_ids.contains(&symbol.file_id))
            .map(|symbol| symbol.id)
            .collect();

        let mut snapshot = IndexSnapshot::empty(self.root.to_string_lossy().to_string());
        snapshot.files = scan.files;
        snapshot.symbols = old_snapshot
            .symbols
            .iter()
            .filter(|symbol| !old_changed_file_ids.contains(&symbol.file_id))
            .cloned()
            .collect();
        snapshot.imports = old_snapshot
            .imports
            .iter()
            .filter(|import| !old_changed_file_ids.contains(&import.file_id))
            .cloned()
            .collect();
        snapshot.edges = old_snapshot
            .edges
            .iter()
            .filter(|edge| edge.relation != RelationType::Imports)
            .filter(|edge| {
                !changed_symbol_ids.contains(&edge.from) && !changed_symbol_ids.contains(&edge.to)
            })
            .cloned()
            .collect();

        for parsed in parsed_files {
            snapshot.symbols.extend(parsed.symbols);
            snapshot.imports.extend(parsed.imports);
            snapshot.edges.extend(parsed.edges);
        }

        let (import_edges, import_warnings) = resolve_import_edges(&snapshot);
        warnings.extend(import_warnings);
        snapshot.edges.extend(import_edges);
        dedupe_edges(&mut snapshot.edges);

        let diff = SemanticGraph::diff(
            &SemanticGraph::from_snapshot(&old_snapshot),
            &SemanticGraph::from_snapshot(&snapshot),
        );
        let affected_symbols = affected_symbols(&diff);

        self.store.write_snapshot(&snapshot)?;

        Ok(UpdateReport {
            changed_files: changed_paths,
            added_symbols: diff.added_symbols.len(),
            modified_symbols: diff.modified_symbols.len(),
            removed_symbols: diff.removed_symbols.len(),
            affected_files: affected_file_count(&snapshot, &affected_symbols),
            affected_symbols,
            edges_found: snapshot.edges.len(),
            storage_path: self.store.root().display().to_string(),
            index_size_bytes: self.store.index_size().unwrap_or_default(),
            elapsed_ms: started.elapsed().as_millis(),
            warnings,
        })
    }

    pub fn update_paths(
        &mut self,
        changed_paths: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> anyhow::Result<UpdateReport> {
        let started = std::time::Instant::now();
        let old_snapshot = self.store.read_snapshot_or_empty(&self.root)?;
        let mut warnings = Vec::new();
        let mut changed = Vec::new();
        let mut current_by_path: HashMap<String, FileEntry> = old_snapshot
            .files
            .iter()
            .cloned()
            .map(|file| (file.relative_path.clone(), file))
            .collect();
        let mut files_to_parse = Vec::new();

        for path in changed_paths {
            let absolute_path = normalize_absolute_path(&self.root, path.as_ref());
            let relative_path = relative_path(&self.root, &absolute_path);
            changed.push(relative_path.clone());

            if absolute_path.exists() {
                match self.scanner.file_entry(&absolute_path) {
                    Ok(Some(file)) => {
                        current_by_path.insert(file.relative_path.clone(), file.clone());
                        files_to_parse.push(file);
                    }
                    Ok(None) => {
                        current_by_path.remove(&relative_path);
                    }
                    Err(error) => {
                        warnings.push(format!(
                            "failed to read {}: {error}",
                            absolute_path.display()
                        ));
                    }
                }
            } else {
                current_by_path.remove(&relative_path);
            }
        }

        changed.sort();
        changed.dedup();

        if changed.is_empty() {
            return empty_update_report(&self.store, started, warnings);
        }

        let changed_path_set: std::collections::HashSet<&str> =
            changed.iter().map(String::as_str).collect();
        let old_changed_file_ids: std::collections::HashSet<u64> = old_snapshot
            .files
            .iter()
            .filter(|file| changed_path_set.contains(file.relative_path.as_str()))
            .map(|file| file.id)
            .collect();
        let changed_symbol_ids: std::collections::HashSet<u64> =
            symbols_in_files(&old_snapshot.symbols, &old_changed_file_ids);

        let parsed_results: Vec<anyhow::Result<ParsedFile>> =
            files_to_parse.par_iter().map(parse_file).collect();
        let mut parsed_files = Vec::new();
        for result in parsed_results {
            match result {
                Ok(parsed) => parsed_files.push(parsed),
                Err(error) => warnings.push(error.to_string()),
            }
        }

        let mut snapshot = IndexSnapshot::empty(self.root.to_string_lossy().to_string());
        snapshot.files = current_by_path.into_values().collect();
        snapshot
            .files
            .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        snapshot.symbols = old_snapshot
            .symbols
            .iter()
            .filter(|symbol| !old_changed_file_ids.contains(&symbol.file_id))
            .cloned()
            .collect();
        snapshot.imports = old_snapshot
            .imports
            .iter()
            .filter(|import| !old_changed_file_ids.contains(&import.file_id))
            .cloned()
            .collect();
        snapshot.edges = old_snapshot
            .edges
            .iter()
            .filter(|edge| edge.relation != RelationType::Imports)
            .filter(|edge| {
                !changed_symbol_ids.contains(&edge.from) && !changed_symbol_ids.contains(&edge.to)
            })
            .cloned()
            .collect();

        for parsed in parsed_files {
            snapshot.symbols.extend(parsed.symbols);
            snapshot.imports.extend(parsed.imports);
            snapshot.edges.extend(parsed.edges);
        }

        let (import_edges, import_warnings) = resolve_import_edges(&snapshot);
        warnings.extend(import_warnings);
        snapshot.edges.extend(import_edges);
        dedupe_edges(&mut snapshot.edges);

        let diff = SemanticGraph::diff(
            &SemanticGraph::from_snapshot(&old_snapshot),
            &SemanticGraph::from_snapshot(&snapshot),
        );
        let affected_symbols = affected_symbols(&diff);
        self.store.write_snapshot(&snapshot)?;

        Ok(UpdateReport {
            changed_files: changed,
            added_symbols: diff.added_symbols.len(),
            modified_symbols: diff.modified_symbols.len(),
            removed_symbols: diff.removed_symbols.len(),
            affected_files: affected_file_count(&snapshot, &affected_symbols),
            affected_symbols,
            edges_found: snapshot.edges.len(),
            storage_path: self.store.root().display().to_string(),
            index_size_bytes: self.store.index_size().unwrap_or_default(),
            elapsed_ms: started.elapsed().as_millis(),
            warnings,
        })
    }

    pub fn doctor(root: impl Into<PathBuf>) -> anyhow::Result<DoctorReport> {
        let root = root.into();
        let scanner = Scanner::new(&root);
        let scan = scanner.scan_with_report()?;
        let mut warnings = scan.warnings;

        let store = Store::open_existing(&root).ok();
        let snapshot = if let Some(store) = &store {
            match store.read_snapshot() {
                Ok(snapshot) => Some(snapshot),
                Err(error) => {
                    warnings.push(error.to_string());
                    None
                }
            }
        } else {
            None
        };

        let stale_files = snapshot
            .as_ref()
            .map(|snapshot| stale_files(snapshot, &scan.files))
            .unwrap_or_default();

        Ok(DoctorReport {
            path: root.display().to_string(),
            initialized: store.is_some(),
            has_index: store.as_ref().map(Store::has_index).unwrap_or(false),
            storage_version: snapshot.as_ref().map(|snapshot| snapshot.storage_version),
            indexed_files: snapshot
                .as_ref()
                .map(|snapshot| snapshot.files.len())
                .unwrap_or(0),
            stale_files,
            warnings,
        })
    }
}

fn parse_file(file: &FileEntry) -> anyhow::Result<ParsedFile> {
    let source = std::fs::read_to_string(&file.path)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", file.path))?;
    let mut parser = TypeScriptParser::new();
    parser
        .parse_file(file, &source)
        .map_err(|error| anyhow::anyhow!("failed to parse {}: {error}", file.path))
}

fn empty_update_report(
    store: &Store,
    started: std::time::Instant,
    warnings: Vec<String>,
) -> anyhow::Result<UpdateReport> {
    Ok(UpdateReport {
        changed_files: Vec::new(),
        added_symbols: 0,
        modified_symbols: 0,
        removed_symbols: 0,
        affected_symbols: Vec::new(),
        affected_files: 0,
        edges_found: 0,
        storage_path: store.root().display().to_string(),
        index_size_bytes: store.index_size().unwrap_or_default(),
        elapsed_ms: started.elapsed().as_millis(),
        warnings,
    })
}

fn symbols_in_files(
    symbols: &[Symbol],
    file_ids: &std::collections::HashSet<u64>,
) -> std::collections::HashSet<u64> {
    symbols
        .iter()
        .filter(|symbol| file_ids.contains(&symbol.file_id))
        .map(|symbol| symbol.id)
        .collect()
}

fn normalize_absolute_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn changed_paths(old_snapshot: &IndexSnapshot, current_files: &[FileEntry]) -> Vec<String> {
    let old_files: HashMap<&str, &str> = old_snapshot
        .files
        .iter()
        .map(|file| (file.relative_path.as_str(), file.hash.as_str()))
        .collect();
    let current_paths: std::collections::HashSet<&str> = current_files
        .iter()
        .map(|file| file.relative_path.as_str())
        .collect();
    let mut changed = Vec::new();

    for file in current_files {
        match old_files.get(file.relative_path.as_str()) {
            Some(hash) if *hash == file.hash => {}
            _ => changed.push(file.relative_path.clone()),
        }
    }

    for file in &old_snapshot.files {
        if !current_paths.contains(file.relative_path.as_str()) {
            changed.push(file.relative_path.clone());
        }
    }

    changed.sort();
    changed.dedup();
    changed
}

fn affected_symbols(diff: &GraphDiff) -> Vec<u64> {
    let mut affected = Vec::with_capacity(
        diff.added_symbols.len() + diff.modified_symbols.len() + diff.removed_symbols.len(),
    );
    affected.extend(diff.added_symbols.iter().copied());
    affected.extend(diff.modified_symbols.iter().copied());
    affected.extend(diff.removed_symbols.iter().copied());
    affected.sort_unstable();
    affected.dedup();
    affected
}

fn affected_file_count(snapshot: &IndexSnapshot, affected_symbols: &[u64]) -> usize {
    let affected: std::collections::HashSet<u64> = affected_symbols.iter().copied().collect();
    snapshot
        .symbols
        .iter()
        .filter(|symbol| affected.contains(&symbol.id))
        .map(|symbol| symbol.file_id)
        .collect::<std::collections::HashSet<_>>()
        .len()
}

fn stale_files(snapshot: &IndexSnapshot, current_files: &[FileEntry]) -> Vec<String> {
    let indexed: HashMap<&str, &str> = snapshot
        .files
        .iter()
        .map(|file| (file.relative_path.as_str(), file.hash.as_str()))
        .collect();
    let mut stale = Vec::new();
    for file in current_files {
        match indexed.get(file.relative_path.as_str()) {
            Some(hash) if *hash == file.hash => {}
            _ => stale.push(file.relative_path.clone()),
        }
    }
    stale.sort();
    stale
}

fn build_snapshot(
    root: &Path,
    files: Vec<FileEntry>,
    parsed_files: Vec<ParsedFile>,
) -> (IndexSnapshot, Vec<String>) {
    let mut snapshot = IndexSnapshot::empty(root.to_string_lossy().to_string());
    snapshot.files = files;

    for parsed in parsed_files {
        snapshot.symbols.extend(parsed.symbols);
        snapshot.imports.extend(parsed.imports);
        snapshot.edges.extend(parsed.edges);
    }

    let (import_edges, warnings) = resolve_import_edges(&snapshot);
    snapshot.edges.extend(import_edges);
    dedupe_edges(&mut snapshot.edges);
    (snapshot, warnings)
}

fn resolve_import_edges(snapshot: &IndexSnapshot) -> (Vec<Edge>, Vec<String>) {
    let file_by_id: HashMap<u64, &FileEntry> =
        snapshot.files.iter().map(|file| (file.id, file)).collect();
    let mut symbols_by_name: HashMap<&str, Vec<u64>> = HashMap::new();
    for symbol in &snapshot.symbols {
        symbols_by_name
            .entry(symbol.name.as_str())
            .or_default()
            .push(symbol.id);
    }

    let mut edges = Vec::new();
    let mut warnings = Vec::new();
    for import in &snapshot.imports {
        let imported_file = file_by_id
            .get(&import.file_id)
            .and_then(|file| resolve_relative_module(file, &import.module, &snapshot.files));

        for name in &import.names {
            let candidates = symbols_by_name.get(name.as_str()).into_iter().flatten();
            for candidate_id in candidates {
                let Some(candidate) = snapshot
                    .symbols
                    .iter()
                    .find(|symbol| symbol.id == *candidate_id)
                else {
                    continue;
                };
                let module_matches = imported_file
                    .map(|file| file.id == candidate.file_id)
                    .unwrap_or(true);
                if module_matches {
                    let source = file_by_id
                        .get(&import.file_id)
                        .map(|file| snapshot_file_path(snapshot, file))
                        .and_then(|path| match std::fs::read_to_string(&path) {
                            Ok(source) => Some(source),
                            Err(error) => {
                                warnings.push(format!(
                                    "failed to read import source {}: {error}",
                                    path.display()
                                ));
                                None
                            }
                        })
                        .unwrap_or_default();

                    for local_symbol in snapshot
                        .symbols
                        .iter()
                        .filter(|symbol| symbol.file_id == import.file_id)
                        .filter(|symbol| {
                            source
                                .lines()
                                .skip(symbol.range.start_line.saturating_sub(1) as usize)
                                .take(
                                    symbol
                                        .range
                                        .end_line
                                        .saturating_sub(symbol.range.start_line)
                                        .saturating_add(1)
                                        as usize,
                                )
                                .any(|line| contains_identifier(line, name))
                        })
                    {
                        edges.push(Edge {
                            from: local_symbol.id,
                            to: candidate.id,
                            relation: RelationType::Imports,
                            range: Some(import.range.clone()),
                        });
                    }
                }
            }
        }
    }

    (edges, warnings)
}

fn snapshot_file_path(snapshot: &IndexSnapshot, file: &FileEntry) -> PathBuf {
    PathBuf::from(&snapshot.root).join(&file.relative_path)
}

fn contains_identifier(source: &str, needle: &str) -> bool {
    source.match_indices(needle).any(|(idx, _)| {
        is_left_boundary(source, idx) && is_right_boundary(source, idx + needle.len())
    })
}

fn is_left_boundary(source: &str, idx: usize) -> bool {
    source[..idx]
        .chars()
        .next_back()
        .map(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '$')
        .unwrap_or(true)
}

fn is_right_boundary(source: &str, idx: usize) -> bool {
    source[idx..]
        .chars()
        .next()
        .map(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '$')
        .unwrap_or(true)
}

fn resolve_relative_module<'a>(
    source_file: &FileEntry,
    module: &str,
    files: &'a [FileEntry],
) -> Option<&'a FileEntry> {
    if !module.starts_with('.') {
        return None;
    }

    let source_dir = Path::new(&source_file.relative_path).parent()?;
    let base = normalize_path(source_dir.join(module));
    let candidates = [
        base.clone(),
        format!("{base}.ts"),
        format!("{base}.tsx"),
        format!("{base}/index.ts"),
        format!("{base}/index.tsx"),
    ];

    files.iter().find(|file| {
        candidates
            .iter()
            .any(|candidate| candidate == &file.relative_path)
    })
}

fn normalize_path(path: PathBuf) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(part) => {
                parts.push(part.to_string_lossy().to_string());
            }
            _ => {}
        }
    }
    parts.join("/")
}

fn dedupe_edges(edges: &mut Vec<Edge>) {
    let mut seen = std::collections::HashSet::new();
    edges.retain(|edge| seen.insert((edge.from, edge.to, edge.relation)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{ImportRecord, SourceRange, Symbol, SymbolKind, Visibility};

    #[test]
    fn indexes_sample_project_end_to_end() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/helper.ts"),
            "export function helper() { return 1; }",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("src/app.ts"),
            "import { helper } from './helper'; export function app() { return helper(); }",
        )
        .unwrap();

        let mut indexer = Indexer::new(temp.path()).unwrap();
        let report = indexer.run().unwrap();

        assert_eq!(report.files_scanned, 2);
        assert!(report.symbols_found >= 2);
        assert!(temp.path().join(".sbe/index.bin").exists());
        assert!(report.index_size_bytes > 0);
    }

    #[test]
    fn doctor_detects_stale_files() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("a.ts"), "export function a() {}").unwrap();
        Indexer::new(temp.path()).unwrap().run().unwrap();
        std::fs::write(
            temp.path().join("a.ts"),
            "export function a() { return 1; }",
        )
        .unwrap();

        let report = Indexer::doctor(temp.path()).unwrap();

        assert!(report.has_index);
        assert_eq!(report.stale_files, vec!["a.ts"]);
    }

    #[test]
    fn doctor_does_not_create_storage_for_unindexed_repo() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("a.ts"), "export function a() {}").unwrap();

        let report = Indexer::doctor(temp.path()).unwrap();

        assert!(!report.initialized);
        assert!(!report.has_index);
        assert!(!temp.path().join(".sbe").exists());
    }

    #[test]
    fn import_edges_read_from_snapshot_root_relative_path() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/app.ts"),
            "import { helper } from './helper'; export function app() { return helper(); }",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("src/helper.ts"),
            "export function helper() { return 1; }",
        )
        .unwrap();

        let range = SourceRange {
            start_line: 1,
            end_line: 1,
            start_col: 0,
            end_col: 1,
        };
        let snapshot = IndexSnapshot {
            storage_version: sbe_common::STORAGE_VERSION,
            root: temp.path().to_string_lossy().to_string(),
            files: vec![
                FileEntry {
                    id: 1,
                    path: "old/moved/src/app.ts".into(),
                    relative_path: "src/app.ts".into(),
                    hash: "hash".into(),
                    extension: "ts".into(),
                },
                FileEntry {
                    id: 2,
                    path: "old/moved/src/helper.ts".into(),
                    relative_path: "src/helper.ts".into(),
                    hash: "hash".into(),
                    extension: "ts".into(),
                },
            ],
            symbols: vec![
                Symbol {
                    id: 10,
                    content_hash: "app".into(),
                    name: "app".into(),
                    kind: SymbolKind::Function,
                    file_id: 1,
                    range: range.clone(),
                    parent_symbol: None,
                    visibility: Visibility::Public,
                    signature: None,
                    exported: true,
                },
                Symbol {
                    id: 20,
                    content_hash: "helper".into(),
                    name: "helper".into(),
                    kind: SymbolKind::Function,
                    file_id: 2,
                    range: range.clone(),
                    parent_symbol: None,
                    visibility: Visibility::Public,
                    signature: None,
                    exported: true,
                },
            ],
            imports: vec![ImportRecord {
                file_id: 1,
                module: "./helper".into(),
                names: vec!["helper".into()],
                range,
            }],
            edges: vec![],
        };

        let (edges, warnings) = resolve_import_edges(&snapshot);

        assert!(warnings.is_empty());
        assert!(edges.iter().any(|edge| edge.from == 10 && edge.to == 20));
    }

    #[test]
    fn update_parses_only_changed_files_and_reports_symbol_diff() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/a.ts"),
            "export function createUser() { return 1; }",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("src/b.ts"),
            "export function unchanged() { return 2; }",
        )
        .unwrap();

        let mut indexer = Indexer::new(temp.path()).unwrap();
        indexer.run().unwrap();
        std::fs::write(
            temp.path().join("src/a.ts"),
            "export function createUser() { return 42; }\nexport function deleteUser() { return 0; }",
        )
        .unwrap();

        let report = indexer.update().unwrap();
        let snapshot = sbe_storage::Store::open_existing(temp.path())
            .unwrap()
            .read_snapshot()
            .unwrap();

        assert_eq!(report.changed_files, vec!["src/a.ts"]);
        assert_eq!(report.modified_symbols, 1);
        assert_eq!(report.added_symbols, 1);
        assert_eq!(report.removed_symbols, 0);
        assert!(snapshot
            .symbols
            .iter()
            .any(|symbol| symbol.name == "unchanged"));
        assert!(snapshot
            .symbols
            .iter()
            .any(|symbol| symbol.name == "deleteUser"));
    }

    #[test]
    fn indexes_python_project_end_to_end() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("app")).unwrap();
        std::fs::write(
            temp.path().join("app/main.py"),
            "from fastapi import FastAPI\n\napp = FastAPI()\n\ndef create_user():\n    return helper()\n\ndef helper():\n    return {'ok': True}\n",
        )
        .unwrap();

        let mut indexer = Indexer::new(temp.path()).unwrap();
        let report = indexer.run().unwrap();
        let snapshot = sbe_storage::Store::open_existing(temp.path())
            .unwrap()
            .read_snapshot()
            .unwrap();

        assert_eq!(report.files_scanned, 1);
        assert!(snapshot
            .symbols
            .iter()
            .any(|symbol| symbol.name == "create_user"));
        assert!(snapshot
            .imports
            .iter()
            .any(|import| import.module == "fastapi"));
        assert!(snapshot.edges.iter().any(|edge| edge.to
            == snapshot
                .symbols
                .iter()
                .find(|symbol| symbol.name == "helper")
                .unwrap()
                .id));
    }
}
