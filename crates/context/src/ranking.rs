use rustc_hash::FxHashMap;
use sbe_graph::{GraphTraversal, SemanticGraph, SymbolId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RankingConfig {
    pub direct_dependency_weight: f32,
    pub caller_count_weight: f32,
    pub impact_weight: f32,
    pub depth_weight: f32,
    pub module_crossing_weight: f32,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            direct_dependency_weight: 30.0,
            caller_count_weight: 8.0,
            impact_weight: 0.35,
            depth_weight: 12.0,
            module_crossing_weight: 10.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedSymbol {
    pub symbol_id: SymbolId,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct SymbolRanker {
    config: RankingConfig,
}

impl SymbolRanker {
    pub fn new(config: RankingConfig) -> Self {
        Self { config }
    }

    pub fn rank(
        &self,
        graph: &SemanticGraph,
        root_symbol: SymbolId,
        candidates: &[SymbolId],
        depths: &FxHashMap<SymbolId, usize>,
        impact_scores: &FxHashMap<SymbolId, usize>,
    ) -> Vec<RankedSymbol> {
        let root_file = graph.nodes.get(&root_symbol).map(|node| node.file_id);
        let mut ranked = Vec::with_capacity(candidates.len());

        for symbol_id in candidates {
            let direct_dependency = graph
                .callees(root_symbol)
                .iter()
                .any(|edge| edge.to == *symbol_id);
            let caller_count = graph.callers(*symbol_id).len() as f32;
            let impact = impact_scores.get(symbol_id).copied().unwrap_or_default() as f32;
            let depth = depths.get(symbol_id).copied().unwrap_or(usize::MAX);
            let depth_factor = if depth == usize::MAX {
                0.0
            } else {
                1.0 / (depth.max(1) as f32)
            };
            let module_crossing = match (root_file, graph.nodes.get(symbol_id)) {
                (Some(root_file), Some(node)) if root_file != node.file_id => 1.0,
                _ => 0.0,
            };

            let mut score = 0.0;
            if *symbol_id == root_symbol {
                score = 100.0;
            } else {
                if direct_dependency {
                    score += self.config.direct_dependency_weight;
                }
                score += caller_count * self.config.caller_count_weight;
                score += impact * self.config.impact_weight;
                score += depth_factor * self.config.depth_weight;
                score += module_crossing * self.config.module_crossing_weight;
            }

            ranked.push(RankedSymbol {
                symbol_id: *symbol_id,
                score: score.clamp(0.0, 100.0),
            });
        }

        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.symbol_id.cmp(&b.symbol_id))
        });
        ranked
    }
}

impl Default for SymbolRanker {
    fn default() -> Self {
        Self::new(RankingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{
        Edge, IndexSnapshot, RelationType, SourceRange, Symbol, SymbolKind, Visibility,
    };

    fn symbol(id: u64, file_id: u64) -> Symbol {
        Symbol {
            id,
            content_hash: format!("hash{id}"),
            name: format!("s{id}"),
            kind: SymbolKind::Function,
            file_id,
            range: SourceRange {
                start_line: 1,
                end_line: 2,
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
    fn ranking_prefers_direct_dependencies_and_root() {
        let snapshot = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![],
            symbols: vec![symbol(1, 1), symbol(2, 1), symbol(3, 2)],
            imports: vec![],
            edges: vec![edge(1, 2), edge(3, 2)],
        };
        let graph = SemanticGraph::from_snapshot(&snapshot);
        let mut depths = FxHashMap::default();
        depths.insert(1, 0);
        depths.insert(2, 1);
        depths.insert(3, 2);
        let impact = FxHashMap::default();

        let ranked = SymbolRanker::default().rank(&graph, 1, &[3, 2, 1], &depths, &impact);

        assert_eq!(ranked[0].symbol_id, 1);
        assert!(
            ranked
                .iter()
                .find(|rank| rank.symbol_id == 2)
                .unwrap()
                .score
                > 0.0
        );
        assert!(ranked
            .iter()
            .all(|rank| (0.0..=100.0).contains(&rank.score)));
    }
}
