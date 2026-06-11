use sbe_common::SymbolKind;
use sbe_graph::{FileId, SymbolId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPack {
    pub root_symbol: SymbolId,
    pub summary: String,
    pub symbols: Vec<ContextSymbol>,
    pub dependencies: Vec<DependencyPath>,
    pub callers: Vec<SymbolId>,
    pub code_ranges: Vec<CodeRange>,
    pub metrics: ContextMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextSymbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub importance_score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextMetrics {
    pub symbols_selected: usize,
    pub files_selected: usize,
    pub estimated_tokens: usize,
    pub context_reduction_percent: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyPath {
    pub nodes: Vec<SymbolId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CodeRange {
    pub file_id: FileId,
    pub start_line: u32,
    pub end_line: u32,
}
