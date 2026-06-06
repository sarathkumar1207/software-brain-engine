use serde::{Deserialize, Serialize};

pub const STORAGE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRange {
    pub start_line: u32,
    pub end_line: u32,
    pub start_col: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: u64,
    pub path: String,
    pub relative_path: String,
    pub hash: String,
    pub extension: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub id: u64,
    pub content_hash: String,
    pub name: String,
    pub kind: SymbolKind,
    pub file_id: u64,
    pub range: SourceRange,
    pub parent_symbol: Option<u64>,
    pub visibility: Visibility,
    pub signature: Option<String>,
    pub exported: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Interface,
    Enum,
    Module,
    Import,
    Variable,
    TypeAlias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub from: u64,
    pub to: u64,
    pub relation: RelationType,
    pub range: Option<SourceRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    Imports,
    References,
    Contains,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRecord {
    pub file_id: u64,
    pub module: String,
    pub names: Vec<String>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedFile {
    pub file: FileEntry,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<ImportRecord>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSnapshot {
    pub storage_version: u32,
    pub root: String,
    pub files: Vec<FileEntry>,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<ImportRecord>,
    pub edges: Vec<Edge>,
}

impl IndexSnapshot {
    pub fn empty(root: impl Into<String>) -> Self {
        Self {
            storage_version: STORAGE_VERSION,
            root: root.into(),
            files: Vec::new(),
            symbols: Vec::new(),
            imports: Vec::new(),
            edges: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPacket {
    pub symbol: Symbol,
    pub file_path: String,
    pub source_lines: (u32, u32),
    pub direct_dependencies: Vec<Symbol>,
    pub dependents: Vec<Symbol>,
    pub estimated_tokens: u32,
}
