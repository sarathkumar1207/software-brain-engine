use sbe_common::Symbol;
use sbe_graph::{ImpactAnalysis, ImpactResult, SemanticGraph, SymbolId};
use sbe_symbols::SymbolRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactReport {
    pub origin: SymbolId,
    pub affected: Vec<Symbol>,
    pub affected_symbols: Vec<SymbolId>,
    pub depth: usize,
    pub affected_files: usize,
}

pub struct ImpactAnalyzer<'a> {
    graph: &'a SemanticGraph,
    registry: &'a SymbolRegistry,
}

impl<'a> ImpactAnalyzer<'a> {
    pub fn new(graph: &'a SemanticGraph, registry: &'a SymbolRegistry) -> Self {
        Self { graph, registry }
    }

    pub fn analyze(&self, symbol_id: SymbolId) -> ImpactReport {
        self.analyze_with_max_depth(symbol_id, None)
    }

    pub fn analyze_with_max_depth(
        &self,
        symbol_id: SymbolId,
        max_depth: Option<usize>,
    ) -> ImpactReport {
        let result = self.graph.impact_with_max_depth(symbol_id, max_depth);
        self.report(symbol_id, result)
    }

    fn report(&self, origin: SymbolId, result: ImpactResult) -> ImpactReport {
        let affected = result
            .affected_symbols
            .iter()
            .filter_map(|id| self.registry.get(*id).cloned())
            .collect();

        ImpactReport {
            origin,
            affected,
            affected_symbols: result.affected_symbols,
            depth: result.depth,
            affected_files: result.affected_files,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{Edge, RelationType, SourceRange, SymbolKind, Visibility};

    fn symbol(id: u64, file_id: u64) -> Symbol {
        Symbol {
            id,
            content_hash: format!("hash{id}"),
            name: format!("s{id}"),
            kind: SymbolKind::Function,
            file_id,
            range: SourceRange {
                start_line: 1,
                end_line: 1,
                start_col: 0,
                end_col: 1,
            },
            parent_symbol: None,
            visibility: Visibility::Public,
            signature: None,
            exported: true,
        }
    }

    fn edge(from: u64, to: u64) -> Edge {
        Edge {
            from,
            to,
            relation: RelationType::References,
            range: None,
        }
    }

    #[test]
    fn returns_transitive_dependents_through_cycles() {
        let registry = SymbolRegistry::build(vec![symbol(1, 1), symbol(2, 1), symbol(3, 2)]);
        let graph = SemanticGraph::from_snapshot(&sbe_common::IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![],
            symbols: vec![symbol(1, 1), symbol(2, 1), symbol(3, 2)],
            imports: vec![],
            edges: vec![edge(2, 1), edge(3, 2), edge(1, 3)],
        });

        let report = ImpactAnalyzer::new(&graph, &registry).analyze(1);

        assert_eq!(report.affected.len(), 2);
        assert_eq!(report.affected_symbols.len(), 2);
        assert_eq!(report.depth, 2);
        assert_eq!(report.affected_files, 2);
    }

    #[test]
    fn respects_max_depth() {
        let registry = SymbolRegistry::build(vec![symbol(1, 1), symbol(2, 1), symbol(3, 2)]);
        let graph = SemanticGraph::from_snapshot(&sbe_common::IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![],
            symbols: vec![symbol(1, 1), symbol(2, 1), symbol(3, 2)],
            imports: vec![],
            edges: vec![edge(2, 1), edge(3, 2)],
        });

        let report = ImpactAnalyzer::new(&graph, &registry).analyze_with_max_depth(1, Some(1));

        assert_eq!(report.affected_symbols, vec![2]);
        assert_eq!(report.depth, 1);
    }
}
