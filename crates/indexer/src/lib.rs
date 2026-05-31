use sbe_common::{Edge, FileEntry, IndexSnapshot, ParsedFile, RelationType};
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

pub struct Indexer {
    root: PathBuf,
    store: Store,
    scanner: Scanner,
    parser: TypeScriptParser,
}

impl Indexer {
    pub fn new(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let root = root.into();
        Ok(Self {
            store: Store::open(&root)?,
            scanner: Scanner::new(&root),
            parser: TypeScriptParser::new()?,
            root,
        })
    }

    pub fn init(root: impl AsRef<Path>) -> anyhow::Result<Store> {
        Store::open(root)
    }

    pub fn run(&mut self) -> anyhow::Result<IndexReport> {
        let started = std::time::Instant::now();
        let scan = self.scanner.scan_with_report()?;
        let mut parsed_files = Vec::new();
        let mut warnings = scan.warnings.clone();

        for file in &scan.files {
            let source = match std::fs::read_to_string(&file.path) {
                Ok(source) => source,
                Err(error) => {
                    warnings.push(format!("failed to read {}: {error}", file.path));
                    continue;
                }
            };
            match self.parser.parse_file(file, &source) {
                Ok(parsed) => parsed_files.push(parsed),
                Err(error) => warnings.push(format!("failed to parse {}: {error}", file.path)),
            }
        }

        let snapshot = build_snapshot(&self.root, scan.files, parsed_files);
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

    pub fn doctor(root: impl Into<PathBuf>) -> anyhow::Result<DoctorReport> {
        let root = root.into();
        let store = Store::open(&root)?;
        let scanner = Scanner::new(&root);
        let scan = scanner.scan_with_report()?;
        let mut warnings = scan.warnings;

        let snapshot = if store.has_index() {
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
            initialized: store.root().exists(),
            has_index: store.has_index(),
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
) -> IndexSnapshot {
    let mut snapshot = IndexSnapshot::empty(root.to_string_lossy().to_string());
    snapshot.files = files;

    for parsed in parsed_files {
        snapshot.symbols.extend(parsed.symbols);
        snapshot.imports.extend(parsed.imports);
        snapshot.edges.extend(parsed.edges);
    }

    snapshot.edges.extend(resolve_import_edges(&snapshot));
    dedupe_edges(&mut snapshot.edges);
    snapshot
}

fn resolve_import_edges(snapshot: &IndexSnapshot) -> Vec<Edge> {
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
                        .and_then(|file| std::fs::read_to_string(&file.path).ok())
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

    edges
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
}
