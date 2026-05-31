use sbe_common::Symbol;
use sbe_graph::SemanticGraph;
use sbe_symbols::SymbolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactReport {
    pub origin: u64,
    pub affected: Vec<Symbol>,
}

pub struct ImpactAnalyzer<'a> {
    graph: &'a SemanticGraph,
    registry: &'a SymbolRegistry,
}

impl<'a> ImpactAnalyzer<'a> {
    pub fn new(graph: &'a SemanticGraph, registry: &'a SymbolRegistry) -> Self {
        Self { graph, registry }
    }

    pub fn analyze(&self, symbol_id: u64) -> ImpactReport {
        let mut visited = HashSet::new();
        let mut discovered = HashSet::from([symbol_id]);
        let mut queue = vec![symbol_id];
        let mut affected = Vec::new();

        while let Some(current) = queue.pop() {
            if !visited.insert(current) {
                continue;
            }

            for dependent in self.graph.dependents(current) {
                if visited.contains(&dependent) || !discovered.insert(dependent) {
                    continue;
                }
                if let Some(symbol) = self.registry.get(dependent) {
                    affected.push(symbol.clone());
                    queue.push(dependent);
                }
            }
        }

        ImpactReport {
            origin: symbol_id,
            affected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{Edge, RelationType, SourceRange, SymbolKind, Visibility};

    fn symbol(id: u64) -> Symbol {
        Symbol {
            id,
            content_hash: "hash".into(),
            name: format!("s{id}"),
            kind: SymbolKind::Function,
            file_id: 1,
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

    #[test]
    fn returns_transitive_dependents_through_cycles() {
        let registry = SymbolRegistry::build(vec![symbol(1), symbol(2), symbol(3)]);
        let graph = SemanticGraph::build(&[
            Edge {
                from: 2,
                to: 1,
                relation: RelationType::References,
                range: None,
            },
            Edge {
                from: 3,
                to: 2,
                relation: RelationType::References,
                range: None,
            },
            Edge {
                from: 1,
                to: 3,
                relation: RelationType::References,
                range: None,
            },
        ]);

        let report = ImpactAnalyzer::new(&graph, &registry).analyze(1);

        assert_eq!(report.affected.len(), 2);
    }
}
