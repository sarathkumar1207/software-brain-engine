use crate::budget::{apply_budget, Budget};
use crate::builder::ContextPackBuilder;
use crate::pack::{CodeRange, ContextMetrics, ContextPack, ContextSymbol, DependencyPath};
use crate::ranking::{RankedSymbol, RankingConfig, SymbolRanker};
use rustc_hash::{FxHashMap, FxHashSet};
use sbe_graph::{GraphTraversal, ImpactAnalysis, SemanticGraph, SymbolId};
use sbe_symbols::SymbolRegistry;
use smallvec::SmallVec;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ContextCompilerConfig {
    pub ranking: RankingConfig,
    pub max_dependency_depth: usize,
    pub max_caller_depth: usize,
    pub max_paths: usize,
}

impl Default for ContextCompilerConfig {
    fn default() -> Self {
        Self {
            ranking: RankingConfig::default(),
            max_dependency_depth: 4,
            max_caller_depth: 2,
            max_paths: 24,
        }
    }
}

pub trait ContextCompilation {
    fn compile(&self, root_symbol: SymbolId, budget: Option<Budget>) -> Option<ContextPack>;
}

#[derive(Debug, Clone)]
pub struct ContextCompiler<'a> {
    graph: &'a SemanticGraph,
    registry: &'a SymbolRegistry,
    config: ContextCompilerConfig,
}

impl<'a> ContextCompiler<'a> {
    pub fn new(graph: &'a SemanticGraph, registry: &'a SymbolRegistry) -> Self {
        Self {
            graph,
            registry,
            config: ContextCompilerConfig::default(),
        }
    }

    pub fn with_config(
        graph: &'a SemanticGraph,
        registry: &'a SymbolRegistry,
        config: ContextCompilerConfig,
    ) -> Self {
        Self {
            graph,
            registry,
            config,
        }
    }
}

impl ContextCompilation for ContextCompiler<'_> {
    fn compile(&self, root_symbol: SymbolId, budget: Option<Budget>) -> Option<ContextPack> {
        let root = self.registry.get(root_symbol)?;
        let direct_dependencies: SmallVec<[SymbolId; 8]> = self
            .graph
            .callees(root_symbol)
            .iter()
            .map(|edge| edge.to)
            .collect();
        let direct_callers: SmallVec<[SymbolId; 8]> = self
            .graph
            .callers(root_symbol)
            .iter()
            .map(|edge| edge.to)
            .collect();

        let (dependency_candidates, depths) =
            expand_dependencies(self.graph, root_symbol, self.config.max_dependency_depth);
        let caller_candidates =
            expand_callers(self.graph, root_symbol, self.config.max_caller_depth);
        let impact = self.graph.impact_with_max_depth(root_symbol, Some(6));
        let impact_scores = impact_score_map(self.graph, &impact.affected_symbols);

        let mut candidates: FxHashSet<SymbolId> = FxHashSet::default();
        candidates.insert(root_symbol);
        candidates.extend(direct_dependencies.iter().copied());
        candidates.extend(direct_callers.iter().copied());
        candidates.extend(dependency_candidates);
        candidates.extend(caller_candidates);
        candidates.extend(impact.affected_symbols.iter().copied());
        candidates.retain(|id| self.registry.get(*id).is_some());

        let mut candidate_vec: Vec<SymbolId> = candidates.into_iter().collect();
        candidate_vec.sort_unstable();
        let ranked = SymbolRanker::new(self.config.ranking).rank(
            self.graph,
            root_symbol,
            &candidate_vec,
            &depths,
            &impact_scores,
        );

        let mut required: FxHashSet<SymbolId> = FxHashSet::default();
        required.insert(root_symbol);
        required.extend(direct_dependencies.iter().copied());
        let selected_ids = apply_budget(
            &ranked,
            &required,
            |id| estimate_symbol_tokens(self.registry, id),
            budget,
        );
        let selected: FxHashSet<SymbolId> = selected_ids.iter().copied().collect();

        let symbols = context_symbols(self.registry, &ranked, &selected);
        let dependencies = dependency_paths(
            self.graph,
            root_symbol,
            &selected,
            self.config.max_dependency_depth,
            self.config.max_paths,
        );
        let mut callers: Vec<SymbolId> = direct_callers
            .iter()
            .copied()
            .filter(|id| selected.contains(id))
            .collect();
        callers.sort_unstable();
        callers.dedup();
        let code_ranges = code_ranges(self.registry, &selected);
        let metrics = metrics(self.registry.all().count(), &selected, &code_ranges);
        let summary = summary(
            root.name.as_str(),
            self.registry,
            &direct_dependencies,
            &callers,
            impact.affected_symbols.len(),
        );

        Some(
            ContextPackBuilder::new(root_symbol)
                .summary(summary)
                .symbols(symbols)
                .dependencies(dependencies)
                .callers(callers)
                .code_ranges(code_ranges)
                .metrics(metrics)
                .build(),
        )
    }
}

fn expand_dependencies(
    graph: &SemanticGraph,
    root: SymbolId,
    max_depth: usize,
) -> (Vec<SymbolId>, FxHashMap<SymbolId, usize>) {
    let mut visited = FxHashSet::default();
    let mut depths = FxHashMap::default();
    let mut output = Vec::new();
    let mut queue = VecDeque::from([(root, 0usize)]);
    visited.insert(root);
    depths.insert(root, 0);

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        let mut callees: Vec<SymbolId> =
            graph.callees(current).iter().map(|edge| edge.to).collect();
        callees.sort_unstable();
        for callee in callees {
            if !visited.insert(callee) {
                continue;
            }
            let next_depth = depth + 1;
            depths.insert(callee, next_depth);
            output.push(callee);
            queue.push_back((callee, next_depth));
        }
    }

    (output, depths)
}

fn expand_callers(graph: &SemanticGraph, root: SymbolId, max_depth: usize) -> Vec<SymbolId> {
    let mut visited = FxHashSet::default();
    let mut output = Vec::new();
    let mut queue = VecDeque::from([(root, 0usize)]);
    visited.insert(root);

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        let mut callers: Vec<SymbolId> =
            graph.callers(current).iter().map(|edge| edge.to).collect();
        callers.sort_unstable();
        for caller in callers {
            if !visited.insert(caller) {
                continue;
            }
            output.push(caller);
            queue.push_back((caller, depth + 1));
        }
    }

    output
}

fn impact_score_map(graph: &SemanticGraph, affected: &[SymbolId]) -> FxHashMap<SymbolId, usize> {
    let mut scores = FxHashMap::default();
    for id in affected {
        scores.insert(*id, graph.dependents(*id).len());
    }
    scores
}

fn dependency_paths(
    graph: &SemanticGraph,
    root: SymbolId,
    selected: &FxHashSet<SymbolId>,
    max_depth: usize,
    max_paths: usize,
) -> Vec<DependencyPath> {
    let mut paths = Vec::new();
    let mut queue: VecDeque<Arc<Vec<SymbolId>>> = VecDeque::new();
    queue.push_back(Arc::new(vec![root]));

    while let Some(path) = queue.pop_front() {
        if paths.len() >= max_paths {
            break;
        }
        let Some(current) = path.last().copied() else {
            continue;
        };
        if path.len() > 1 {
            paths.push(DependencyPath {
                nodes: path.as_ref().clone(),
            });
        }
        if path.len().saturating_sub(1) >= max_depth {
            continue;
        }

        let mut next: Vec<SymbolId> = graph
            .callees(current)
            .iter()
            .map(|edge| edge.to)
            .filter(|id| selected.contains(id))
            .filter(|id| !path.contains(id))
            .collect();
        next.sort_unstable();
        for id in next {
            let mut extended = path.as_ref().clone();
            extended.push(id);
            queue.push_back(Arc::new(extended));
        }
    }

    paths.sort_by(|a, b| a.nodes.cmp(&b.nodes));
    paths
}

fn context_symbols(
    registry: &SymbolRegistry,
    ranked: &[RankedSymbol],
    selected: &FxHashSet<SymbolId>,
) -> Vec<ContextSymbol> {
    ranked
        .iter()
        .filter(|rank| selected.contains(&rank.symbol_id))
        .filter_map(|rank| {
            let symbol = registry.get(rank.symbol_id)?;
            Some(ContextSymbol {
                id: symbol.id,
                name: symbol.name.clone(),
                kind: symbol.kind,
                importance_score: round_score(rank.score),
            })
        })
        .collect()
}

fn code_ranges(registry: &SymbolRegistry, selected: &FxHashSet<SymbolId>) -> Vec<CodeRange> {
    let mut ranges: Vec<CodeRange> = selected
        .iter()
        .filter_map(|id| {
            let symbol = registry.get(*id)?;
            Some(CodeRange {
                file_id: symbol.file_id,
                start_line: symbol.range.start_line,
                end_line: symbol.range.end_line,
            })
        })
        .collect();
    ranges.sort_by_key(|range| (range.file_id, range.start_line, range.end_line));
    ranges.dedup();
    ranges
}

fn metrics(
    repository_symbols: usize,
    selected: &FxHashSet<SymbolId>,
    code_ranges: &[CodeRange],
) -> ContextMetrics {
    let files_selected: FxHashSet<_> = code_ranges.iter().map(|range| range.file_id).collect();
    let estimated_tokens: usize = code_ranges
        .iter()
        .map(|range| {
            range
                .end_line
                .saturating_sub(range.start_line)
                .saturating_add(1) as usize
                * 20
        })
        .sum();
    let reduction = if repository_symbols == 0 {
        0.0
    } else {
        (1.0 - (selected.len() as f32 / repository_symbols as f32)) * 100.0
    };

    ContextMetrics {
        symbols_selected: selected.len(),
        files_selected: files_selected.len(),
        estimated_tokens,
        context_reduction_percent: round_score(reduction.max(0.0)),
    }
}

fn estimate_symbol_tokens(registry: &SymbolRegistry, symbol_id: SymbolId) -> usize {
    registry
        .get(symbol_id)
        .map(|symbol| {
            symbol
                .range
                .end_line
                .saturating_sub(symbol.range.start_line)
                .saturating_add(1) as usize
                * 20
        })
        .unwrap_or(20)
}

fn summary(
    root_name: &str,
    registry: &SymbolRegistry,
    dependencies: &[SymbolId],
    callers: &[SymbolId],
    impacted: usize,
) -> String {
    let mut lines = vec![format!("{root_name}:")];
    for id in dependencies.iter().take(8) {
        if let Some(symbol) = registry.get(*id) {
            lines.push(format!("* Calls {}", symbol.name));
        }
    }
    for id in callers.iter().take(8) {
        if let Some(symbol) = registry.get(*id) {
            lines.push(format!("* Used by {}", symbol.name));
        }
    }
    lines.push(format!("* Impacts {impacted} downstream symbols"));
    lines.join("\n")
}

fn round_score(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{
        Edge, IndexSnapshot, RelationType, SourceRange, Symbol, SymbolKind, Visibility,
    };

    fn symbol(id: u64, name: &str, file_id: u64) -> Symbol {
        Symbol {
            id,
            content_hash: format!("hash{id}"),
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

    fn edge(from: u64, to: u64) -> Edge {
        Edge {
            from,
            to,
            relation: RelationType::References,
            range: None,
        }
    }

    fn compiler_fixture() -> (SemanticGraph, SymbolRegistry) {
        let snapshot = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![],
            symbols: vec![
                symbol(1, "createUser", 1),
                symbol(2, "validateEmail", 1),
                symbol(3, "saveUser", 2),
                symbol(4, "publishEvent", 3),
                symbol(5, "AuthController", 4),
            ],
            imports: vec![],
            edges: vec![edge(1, 2), edge(1, 3), edge(3, 4), edge(5, 1)],
        };
        let graph = SemanticGraph::from_snapshot(&snapshot);
        let registry = SymbolRegistry::build(snapshot.symbols);
        (graph, registry)
    }

    #[test]
    fn extracts_ordered_dependency_paths() {
        let (graph, registry) = compiler_fixture();
        let pack = ContextCompiler::new(&graph, &registry)
            .compile(1, None)
            .unwrap();

        assert!(pack
            .dependencies
            .iter()
            .any(|path| path.nodes == vec![1, 2]));
        assert!(pack
            .dependencies
            .iter()
            .any(|path| path.nodes == vec![1, 3, 4]));
    }

    #[test]
    fn generates_deterministic_summary() {
        let (graph, registry) = compiler_fixture();
        let pack = ContextCompiler::new(&graph, &registry)
            .compile(1, None)
            .unwrap();

        assert!(pack.summary.contains("createUser:"));
        assert!(pack.summary.contains("* Calls validateEmail"));
        assert!(pack.summary.contains("* Used by AuthController"));
        assert!(pack.summary.contains("* Impacts"));
    }

    #[test]
    fn compiles_context_pack_with_metrics_and_ranges() {
        let (graph, registry) = compiler_fixture();
        let pack = ContextCompiler::new(&graph, &registry)
            .compile(1, None)
            .unwrap();

        assert_eq!(pack.root_symbol, 1);
        assert!(pack.metrics.symbols_selected >= 4);
        assert!(pack.metrics.context_reduction_percent >= 0.0);
        assert!(pack
            .code_ranges
            .iter()
            .all(|range| range.end_line >= range.start_line));
    }

    #[test]
    fn budget_keeps_root_and_direct_dependencies() {
        let (graph, registry) = compiler_fixture();
        let pack = ContextCompiler::new(&graph, &registry)
            .compile(1, Some(Budget::new(1)))
            .unwrap();
        let selected: FxHashSet<_> = pack.symbols.iter().map(|symbol| symbol.id).collect();

        assert!(selected.contains(&1));
        assert!(selected.contains(&2));
        assert!(selected.contains(&3));
    }
}
