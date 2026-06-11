use sbe_common::{
    Edge as SnapshotEdge, FileEntry, IndexSnapshot, RelationType, Symbol, SymbolKind,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

pub type SymbolId = u64;
pub type FileId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolNode {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub file_id: FileId,
    pub start_line: u32,
    pub end_line: u32,
    pub hash: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeType {
    Calls,
    CalledBy,
    Imports,
    ImportedBy,
    Defines,
    DefinedIn,
    Extends,
    Implements,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: SymbolId,
    pub to: SymbolId,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRange {
    pub file_id: FileId,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextPack {
    pub root_symbol: SymbolId,
    pub dependencies: Vec<SymbolId>,
    pub callers: Vec<SymbolId>,
    pub file_ranges: Vec<FileRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactResult {
    pub affected_symbols: Vec<SymbolId>,
    pub depth: usize,
    pub affected_files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolVersion {
    pub symbol_id: SymbolId,
    pub hash: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphDiff {
    pub added_symbols: Vec<SymbolId>,
    pub modified_symbols: Vec<SymbolId>,
    pub removed_symbols: Vec<SymbolId>,
}

pub trait GraphTraversal {
    fn callers(&self, symbol_id: SymbolId) -> &[Edge];
    fn callees(&self, symbol_id: SymbolId) -> &[Edge];
}

pub trait ImpactAnalysis {
    fn impact(&self, symbol_id: SymbolId) -> ImpactResult;
    fn impact_with_max_depth(&self, symbol_id: SymbolId, max_depth: Option<usize>) -> ImpactResult;
}

#[derive(Debug, Clone, Default)]
pub struct SemanticGraph {
    pub nodes: HashMap<SymbolId, SymbolNode>,
    pub edges: HashMap<SymbolId, Vec<Edge>>,
    reverse_edges: HashMap<SymbolId, Vec<Edge>>,
}

impl SemanticGraph {
    pub fn build(edges: &[SnapshotEdge]) -> Self {
        let mut graph = Self::default();
        for edge in edges {
            graph.add_edge(edge.from, edge.to, edge_type_from_relation(edge.relation));
        }
        graph
    }

    pub fn from_snapshot(snapshot: &IndexSnapshot) -> Self {
        let mut graph = Self::build(&snapshot.edges);
        for symbol in &snapshot.symbols {
            graph.insert_symbol(symbol);
        }
        graph
    }

    pub fn insert_symbol(&mut self, symbol: &Symbol) {
        self.nodes.insert(symbol.id, SymbolNode::from(symbol));
    }

    pub fn add_edge(&mut self, from: SymbolId, to: SymbolId, edge_type: EdgeType) {
        let forward = Edge {
            from,
            to,
            edge_type,
        };
        let reverse = Edge {
            from: to,
            to: from,
            edge_type: reverse_edge_type(edge_type),
        };
        self.edges.entry(from).or_default().push(forward);
        self.reverse_edges.entry(to).or_default().push(reverse);
    }

    pub fn dependencies(&self, symbol_id: SymbolId) -> Vec<SymbolId> {
        self.callees(symbol_id).iter().map(|edge| edge.to).collect()
    }

    pub fn dependents(&self, symbol_id: SymbolId) -> Vec<SymbolId> {
        self.callers(symbol_id).iter().map(|edge| edge.to).collect()
    }

    pub fn context(&self, symbol_id: SymbolId) -> ContextPack {
        let mut dependencies = self.dependencies(symbol_id);
        dependencies.sort_unstable();
        dependencies.dedup();

        let mut callers = self.dependents(symbol_id);
        callers.sort_unstable();
        callers.dedup();

        let mut file_ranges = Vec::new();
        if let Some(root) = self.nodes.get(&symbol_id) {
            file_ranges.push(FileRange::from(root));
        }
        for id in dependencies.iter().chain(callers.iter()) {
            if let Some(node) = self.nodes.get(id) {
                file_ranges.push(FileRange::from(node));
            }
        }
        file_ranges.sort_by_key(|range| (range.file_id, range.start_line, range.end_line));
        file_ranges.dedup();

        ContextPack {
            root_symbol: symbol_id,
            dependencies,
            callers,
            file_ranges,
        }
    }

    pub fn versions(&self) -> Vec<SymbolVersion> {
        let timestamp = current_timestamp();
        let mut versions: Vec<SymbolVersion> = self
            .nodes
            .values()
            .map(|node| SymbolVersion {
                symbol_id: node.id,
                hash: node.hash,
                timestamp,
            })
            .collect();
        versions.sort_by_key(|version| version.symbol_id);
        versions
    }

    pub fn diff(old: &Self, new: &Self) -> GraphDiff {
        let mut diff = GraphDiff::default();

        for (id, new_node) in &new.nodes {
            match old.nodes.get(id) {
                None => diff.added_symbols.push(*id),
                Some(old_node) if old_node.hash != new_node.hash => {
                    diff.modified_symbols.push(*id);
                }
                Some(_) => {}
            }
        }

        for id in old.nodes.keys() {
            if !new.nodes.contains_key(id) {
                diff.removed_symbols.push(*id);
            }
        }

        diff.added_symbols.sort_unstable();
        diff.modified_symbols.sort_unstable();
        diff.removed_symbols.sort_unstable();
        diff
    }
}

impl GraphTraversal for SemanticGraph {
    fn callers(&self, symbol_id: SymbolId) -> &[Edge] {
        self.reverse_edges
            .get(&symbol_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn callees(&self, symbol_id: SymbolId) -> &[Edge] {
        self.edges.get(&symbol_id).map(Vec::as_slice).unwrap_or(&[])
    }
}

impl ImpactAnalysis for SemanticGraph {
    fn impact(&self, symbol_id: SymbolId) -> ImpactResult {
        self.impact_with_max_depth(symbol_id, None)
    }

    fn impact_with_max_depth(&self, symbol_id: SymbolId, max_depth: Option<usize>) -> ImpactResult {
        let mut visited = HashSet::from([symbol_id]);
        let mut queue = VecDeque::from([(symbol_id, 0usize)]);
        let mut affected_symbols = Vec::new();
        let mut affected_files = HashSet::new();
        let mut reached_depth = 0usize;

        while let Some((current, depth)) = queue.pop_front() {
            if max_depth.is_some_and(|limit| depth >= limit) {
                continue;
            }

            for edge in self.callers(current) {
                let dependent = edge.to;
                if !visited.insert(dependent) {
                    continue;
                }

                let next_depth = depth + 1;
                reached_depth = reached_depth.max(next_depth);
                affected_symbols.push(dependent);
                if let Some(node) = self.nodes.get(&dependent) {
                    affected_files.insert(node.file_id);
                }
                queue.push_back((dependent, next_depth));
            }
        }

        ImpactResult {
            affected_symbols,
            depth: reached_depth,
            affected_files: affected_files.len(),
        }
    }
}

impl From<&Symbol> for SymbolNode {
    fn from(symbol: &Symbol) -> Self {
        Self {
            id: symbol.id,
            name: symbol.name.clone(),
            kind: symbol.kind,
            file_id: symbol.file_id,
            start_line: symbol.range.start_line,
            end_line: symbol.range.end_line,
            hash: stable_hash(&symbol.content_hash),
        }
    }
}

impl From<&SymbolNode> for FileRange {
    fn from(node: &SymbolNode) -> Self {
        Self {
            file_id: node.file_id,
            start_line: node.start_line,
            end_line: node.end_line,
        }
    }
}

pub fn symbol_versions(symbols: &[Symbol]) -> Vec<SymbolVersion> {
    let timestamp = current_timestamp();
    symbols
        .iter()
        .map(|symbol| SymbolVersion {
            symbol_id: symbol.id,
            hash: stable_hash(&symbol.content_hash),
            timestamp,
        })
        .collect()
}

pub fn changed_file_ids(old: &IndexSnapshot, new_files: &[FileEntry]) -> HashSet<FileId> {
    let old_files: HashMap<&str, &FileEntry> = old
        .files
        .iter()
        .map(|file| (file.relative_path.as_str(), file))
        .collect();
    let new_paths: HashSet<&str> = new_files
        .iter()
        .map(|file| file.relative_path.as_str())
        .collect();
    let mut changed = HashSet::new();

    for file in new_files {
        match old_files.get(file.relative_path.as_str()) {
            Some(old_file) if old_file.hash == file.hash => {}
            Some(old_file) => {
                changed.insert(old_file.id);
                changed.insert(file.id);
            }
            None => {
                changed.insert(file.id);
            }
        }
    }

    for old_file in &old.files {
        if !new_paths.contains(old_file.relative_path.as_str()) {
            changed.insert(old_file.id);
        }
    }

    changed
}

fn edge_type_from_relation(relation: RelationType) -> EdgeType {
    match relation {
        RelationType::Imports => EdgeType::Imports,
        RelationType::Contains => EdgeType::Defines,
        RelationType::References => EdgeType::Calls,
    }
}

fn reverse_edge_type(edge_type: EdgeType) -> EdgeType {
    match edge_type {
        EdgeType::Calls => EdgeType::CalledBy,
        EdgeType::CalledBy => EdgeType::Calls,
        EdgeType::Imports => EdgeType::ImportedBy,
        EdgeType::ImportedBy => EdgeType::Imports,
        EdgeType::Defines => EdgeType::DefinedIn,
        EdgeType::DefinedIn => EdgeType::Defines,
        EdgeType::Extends => EdgeType::Extends,
        EdgeType::Implements => EdgeType::Implements,
    }
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{SourceRange, Visibility};

    fn edge(from: u64, to: u64) -> SnapshotEdge {
        SnapshotEdge {
            from,
            to,
            relation: RelationType::References,
            range: Some(SourceRange {
                start_line: 1,
                end_line: 1,
                start_col: 0,
                end_col: 1,
            }),
        }
    }

    fn symbol(id: u64, name: &str, file_id: u64, hash: &str) -> Symbol {
        Symbol {
            id,
            content_hash: hash.into(),
            name: name.into(),
            kind: SymbolKind::Function,
            file_id,
            range: SourceRange {
                start_line: id as u32,
                end_line: id as u32 + 1,
                start_col: 0,
                end_col: 1,
            },
            parent_symbol: None,
            visibility: Visibility::Public,
            signature: None,
            exported: true,
        }
    }

    #[test]
    fn caller_lookup_is_o1_indexed() {
        let graph = SemanticGraph::build(&[edge(1, 2), edge(3, 2)]);
        let mut callers: Vec<u64> = graph.callers(2).iter().map(|edge| edge.to).collect();
        callers.sort_unstable();

        assert_eq!(callers, vec![1, 3]);
    }

    #[test]
    fn callee_lookup_is_o1_indexed() {
        let graph = SemanticGraph::build(&[edge(1, 2), edge(1, 3)]);
        let mut callees: Vec<u64> = graph.callees(1).iter().map(|edge| edge.to).collect();
        callees.sort_unstable();

        assert_eq!(callees, vec![2, 3]);
    }

    #[test]
    fn traverses_forward_and_reverse_compatibility_methods() {
        let graph = SemanticGraph::build(&[edge(1, 2), edge(3, 2)]);

        assert_eq!(graph.dependencies(1), vec![2]);
        let mut dependents = graph.dependents(2);
        dependents.sort_unstable();
        assert_eq!(dependents, vec![1, 3]);
    }

    #[test]
    fn impact_uses_bfs_reverse_dependencies_with_cycle_protection() {
        let graph = SemanticGraph::build(&[edge(2, 1), edge(3, 2), edge(1, 3)]);

        let result = graph.impact(1);

        assert_eq!(result.affected_symbols.len(), 2);
        assert_eq!(result.depth, 2);
    }

    #[test]
    fn graph_diff_detects_added_modified_and_removed_symbols() {
        let old = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![],
            symbols: vec![symbol(1, "a", 1, "old"), symbol(2, "b", 1, "same")],
            imports: vec![],
            edges: vec![],
        };
        let new = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![],
            symbols: vec![symbol(1, "a", 1, "new"), symbol(3, "c", 2, "added")],
            imports: vec![],
            edges: vec![],
        };

        let diff = SemanticGraph::diff(
            &SemanticGraph::from_snapshot(&old),
            &SemanticGraph::from_snapshot(&new),
        );

        assert_eq!(diff.added_symbols, vec![3]);
        assert_eq!(diff.modified_symbols, vec![1]);
        assert_eq!(diff.removed_symbols, vec![2]);
    }

    #[test]
    fn context_pack_contains_dependencies_callers_and_ranges() {
        let snapshot = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![],
            symbols: vec![
                symbol(1, "root", 1, "a"),
                symbol(2, "dependency", 2, "b"),
                symbol(3, "caller", 3, "c"),
            ],
            imports: vec![],
            edges: vec![edge(1, 2), edge(3, 1)],
        };
        let graph = SemanticGraph::from_snapshot(&snapshot);

        let context = graph.context(1);

        assert_eq!(context.dependencies, vec![2]);
        assert_eq!(context.callers, vec![3]);
        assert_eq!(context.file_ranges.len(), 3);
    }
}
