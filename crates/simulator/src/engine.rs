use crate::detection::{FlowDetection, GraphDetector, TestSelection};
use crate::model::{RiskLevel, SimulationReport, SimulationRequest};
use crate::risk::{RiskInput, RiskScorer, RiskScoring};
use rustc_hash::FxHashSet;
use sbe_common::FileEntry;
use sbe_context::{Budget, ContextCompilation, ContextCompiler};
use sbe_graph::{GraphTraversal, SemanticGraph, SymbolId};
use sbe_symbols::SymbolRegistry;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, PartialEq, Eq)]
pub enum SimulationError {
    SymbolNotFound(SymbolId),
    ContextCompilationFailed(SymbolId),
}

impl std::fmt::Display for SimulationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SymbolNotFound(id) => {
                write!(formatter, "symbol {id} does not exist in the graph")
            }
            Self::ContextCompilationFailed(id) => {
                write!(formatter, "context compilation failed for symbol {id}")
            }
        }
    }
}

impl std::error::Error for SimulationError {}

pub trait ChangeSimulation {
    fn simulate(&self, request: SimulationRequest) -> Result<SimulationReport, SimulationError>;
}

pub struct ChangeSimulator<'a> {
    graph: &'a SemanticGraph,
    registry: &'a SymbolRegistry,
    files: &'a HashMap<u64, FileEntry>,
    risk_scorer: RiskScorer,
    context_budget: Option<Budget>,
}

impl<'a> ChangeSimulator<'a> {
    pub fn new(
        graph: &'a SemanticGraph,
        registry: &'a SymbolRegistry,
        files: &'a HashMap<u64, FileEntry>,
    ) -> Self {
        Self {
            graph,
            registry,
            files,
            risk_scorer: RiskScorer::default(),
            context_budget: Some(Budget::new(8_000)),
        }
    }

    pub fn with_risk_scorer(mut self, risk_scorer: RiskScorer) -> Self {
        self.risk_scorer = risk_scorer;
        self
    }

    pub fn with_context_budget(mut self, budget: Option<Budget>) -> Self {
        self.context_budget = budget;
        self
    }
}

impl ChangeSimulation for ChangeSimulator<'_> {
    fn simulate(&self, request: SimulationRequest) -> Result<SimulationReport, SimulationError> {
        if self.registry.get(request.target_symbol).is_none() {
            return Err(SimulationError::SymbolNotFound(request.target_symbol));
        }

        let forward = traverse(
            self.graph,
            request.target_symbol,
            request.max_depth,
            Direction::Forward,
        );
        let reverse = traverse(
            self.graph,
            request.target_symbol,
            request.max_depth,
            Direction::Reverse,
        );
        let mut affected = forward.visited;
        affected.extend(reverse.visited.iter().copied());
        affected.insert(request.target_symbol);

        let detector = GraphDetector::new(self.graph, self.registry, self.files);
        let affected_flows = detector.affected_flows(&reverse.visited, request.target_symbol);
        let recommended_tests = detector.recommended_tests(&affected);
        let reached_depth = forward.reached_depth.max(reverse.reached_depth);
        let cross_module_edges = cross_module_edges(self.graph, &affected);
        let (risk_score, risk_breakdown) = self.risk_scorer.score(RiskInput {
            operation: request.operation,
            caller_count: self.graph.callers(request.target_symbol).len(),
            dependency_count: self.graph.callees(request.target_symbol).len(),
            cross_module_edges,
            reached_depth,
            max_depth: request.max_depth,
            critical_flows: affected_flows.len(),
        });
        let context_pack = ContextCompiler::new(self.graph, self.registry)
            .compile(request.target_symbol, self.context_budget)
            .ok_or(SimulationError::ContextCompilationFailed(
                request.target_symbol,
            ))?;

        let mut affected_symbols: Vec<SymbolId> = affected.into_iter().collect();
        affected_symbols.sort_unstable();
        let mut affected_files: Vec<u64> = affected_symbols
            .iter()
            .filter_map(|id| self.registry.get(*id).map(|symbol| symbol.file_id))
            .collect();
        affected_files.sort_unstable();
        affected_files.dedup();

        Ok(SimulationReport {
            operation: request.operation,
            target_symbol: request.target_symbol,
            risk_score,
            risk_level: RiskLevel::from_score(risk_score),
            risk_breakdown,
            traversal_depth: reached_depth,
            affected_symbols,
            affected_files,
            affected_flows,
            recommended_tests,
            context_pack,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Forward,
    Reverse,
}

struct Traversal {
    visited: FxHashSet<SymbolId>,
    reached_depth: usize,
}

fn traverse(
    graph: &SemanticGraph,
    root: SymbolId,
    max_depth: usize,
    direction: Direction,
) -> Traversal {
    let mut visited = FxHashSet::default();
    let mut queue = VecDeque::from([(root, 0usize)]);
    let mut reached_depth = 0;
    visited.insert(root);

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        let edges = match direction {
            Direction::Forward => graph.callees(current),
            Direction::Reverse => graph.callers(current),
        };
        for edge in edges {
            if !visited.insert(edge.to) {
                continue;
            }
            let next_depth = depth + 1;
            reached_depth = reached_depth.max(next_depth);
            queue.push_back((edge.to, next_depth));
        }
    }

    visited.remove(&root);
    Traversal {
        visited,
        reached_depth,
    }
}

fn cross_module_edges(graph: &SemanticGraph, affected: &FxHashSet<SymbolId>) -> usize {
    let mut count = 0;
    for id in affected {
        let Some(source) = graph.nodes.get(id) else {
            continue;
        };
        for edge in graph.callees(*id) {
            if affected.contains(&edge.to)
                && graph
                    .nodes
                    .get(&edge.to)
                    .is_some_and(|target| target.file_id != source.file_id)
            {
                count += 1;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SimulationOperation;
    use sbe_common::{
        Edge, IndexSnapshot, RelationType, SourceRange, Symbol, SymbolKind, Visibility,
    };

    fn symbol(id: u64, name: &str, file_id: u64) -> Symbol {
        Symbol {
            id,
            content_hash: format!("hash-{id}"),
            name: name.into(),
            kind: SymbolKind::Function,
            file_id,
            range: SourceRange {
                start_line: 1,
                end_line: 4,
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

    fn fixture() -> (SemanticGraph, SymbolRegistry, HashMap<u64, FileEntry>) {
        let files = vec![
            FileEntry {
                id: 1,
                path: "src/auth.ts".into(),
                relative_path: "src/auth.ts".into(),
                hash: "1".into(),
                extension: "ts".into(),
            },
            FileEntry {
                id: 2,
                path: "src/user.ts".into(),
                relative_path: "src/user.ts".into(),
                hash: "2".into(),
                extension: "ts".into(),
            },
            FileEntry {
                id: 3,
                path: "tests/auth.test.ts".into(),
                relative_path: "tests/auth.test.ts".into(),
                hash: "3".into(),
                extension: "ts".into(),
            },
            FileEntry {
                id: 4,
                path: "src/events.py".into(),
                relative_path: "src/events.py".into(),
                hash: "4".into(),
                extension: "py".into(),
            },
        ];
        let symbols = vec![
            symbol(1, "createUser", 2),
            symbol(2, "validateEmail", 2),
            symbol(3, "saveUser", 2),
            symbol(4, "Signup", 1),
            symbol(5, "Login", 1),
            symbol(6, "test_create_user", 3),
            symbol(7, "publishWelcomeEvent", 4),
        ];
        let snapshot = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: files.clone(),
            symbols: symbols.clone(),
            imports: vec![],
            edges: vec![
                edge(1, 2),
                edge(1, 3),
                edge(3, 7),
                edge(4, 1),
                edge(5, 1),
                edge(6, 1),
                edge(7, 1),
            ],
        };
        (
            SemanticGraph::from_snapshot(&snapshot),
            SymbolRegistry::build(symbols),
            files.into_iter().map(|file| (file.id, file)).collect(),
        )
    }

    #[test]
    fn modify_simulation_traverses_both_directions_and_handles_cycles() {
        let (graph, registry, files) = fixture();
        let report = ChangeSimulator::new(&graph, &registry, &files)
            .simulate(SimulationRequest {
                operation: SimulationOperation::Modify,
                target_symbol: 1,
                max_depth: 4,
            })
            .unwrap();

        assert_eq!(report.affected_symbols, vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(report.affected_files, vec![1, 2, 3, 4]);
        assert!(report.affected_flows.contains(&4));
        assert!(report.affected_flows.contains(&5));
        assert_eq!(report.recommended_tests, vec![6]);
        assert_eq!(report.context_pack.root_symbol, 1);
        assert!(report.traversal_depth <= 4);
    }

    #[test]
    fn max_depth_limits_blast_radius() {
        let (graph, registry, files) = fixture();
        let report = ChangeSimulator::new(&graph, &registry, &files)
            .simulate(SimulationRequest {
                operation: SimulationOperation::Replace,
                target_symbol: 2,
                max_depth: 1,
            })
            .unwrap();

        assert!(report.affected_symbols.contains(&1));
        assert!(!report.affected_symbols.contains(&4));
        assert!(!report.affected_symbols.contains(&5));
        assert_eq!(report.traversal_depth, 1);
    }

    #[test]
    fn delete_has_higher_risk_than_modify() {
        let (graph, registry, files) = fixture();
        let simulator = ChangeSimulator::new(&graph, &registry, &files);
        let modify = simulator
            .simulate(SimulationRequest {
                operation: SimulationOperation::Modify,
                target_symbol: 1,
                max_depth: 4,
            })
            .unwrap();
        let delete = simulator
            .simulate(SimulationRequest {
                operation: SimulationOperation::Delete,
                target_symbol: 1,
                max_depth: 4,
            })
            .unwrap();

        assert!(delete.risk_score > modify.risk_score);
        assert_eq!(delete.risk_breakdown.operation_score, 10.0);
    }

    #[test]
    fn missing_symbol_returns_typed_error() {
        let (graph, registry, files) = fixture();
        let error = ChangeSimulator::new(&graph, &registry, &files)
            .simulate(SimulationRequest {
                operation: SimulationOperation::Modify,
                target_symbol: 999,
                max_depth: 4,
            })
            .unwrap_err();

        assert_eq!(error, SimulationError::SymbolNotFound(999));
    }
}
