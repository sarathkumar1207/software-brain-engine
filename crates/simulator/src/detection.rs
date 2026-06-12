use crate::model::{FlowId, TestId};
use rustc_hash::FxHashSet;
use sbe_common::{FileEntry, Symbol, SymbolKind};
use sbe_graph::{GraphTraversal, SemanticGraph, SymbolId};
use sbe_symbols::SymbolRegistry;
use std::collections::HashMap;

pub trait FlowDetection {
    fn affected_flows(
        &self,
        reverse_reachable: &FxHashSet<SymbolId>,
        target: SymbolId,
    ) -> Vec<FlowId>;
}

pub trait TestSelection {
    fn recommended_tests(&self, affected: &FxHashSet<SymbolId>) -> Vec<TestId>;
}

pub struct GraphDetector<'a> {
    graph: &'a SemanticGraph,
    registry: &'a SymbolRegistry,
    files: &'a HashMap<u64, FileEntry>,
}

impl<'a> GraphDetector<'a> {
    pub fn new(
        graph: &'a SemanticGraph,
        registry: &'a SymbolRegistry,
        files: &'a HashMap<u64, FileEntry>,
    ) -> Self {
        Self {
            graph,
            registry,
            files,
        }
    }

    fn is_test(&self, symbol: &Symbol) -> bool {
        let name = normalized(&symbol.name);
        let path = self
            .files
            .get(&symbol.file_id)
            .map(|file| file.relative_path.to_ascii_lowercase())
            .unwrap_or_default();
        name.starts_with("test")
            || name.ends_with("test")
            || name.ends_with("spec")
            || path.contains("/tests/")
            || path.contains("/test/")
            || path.contains("__tests__")
            || path.contains(".test.")
            || path.contains(".spec.")
            || path.starts_with("tests/")
            || path.starts_with("test/")
    }
}

impl FlowDetection for GraphDetector<'_> {
    fn affected_flows(
        &self,
        reverse_reachable: &FxHashSet<SymbolId>,
        target: SymbolId,
    ) -> Vec<FlowId> {
        let mut flows: Vec<FlowId> = reverse_reachable
            .iter()
            .copied()
            .chain(std::iter::once(target))
            .filter(|id| {
                let Some(symbol) = self.registry.get(*id) else {
                    return false;
                };
                if self.is_test(symbol) {
                    return false;
                }
                let name = normalized(&symbol.name);
                let named_flow = FLOW_KEYWORDS.iter().any(|term| name.contains(term));
                let graph_entry = symbol.exported
                    && self.graph.callers(*id).is_empty()
                    && matches!(
                        symbol.kind,
                        SymbolKind::Function
                            | SymbolKind::Method
                            | SymbolKind::Class
                            | SymbolKind::Module
                    );
                named_flow || graph_entry
            })
            .collect();
        flows.sort_unstable();
        flows.dedup();
        flows
    }
}

impl TestSelection for GraphDetector<'_> {
    fn recommended_tests(&self, affected: &FxHashSet<SymbolId>) -> Vec<TestId> {
        let mut tests = FxHashSet::default();
        for id in affected {
            if self
                .registry
                .get(*id)
                .is_some_and(|symbol| self.is_test(symbol))
            {
                tests.insert(*id);
            }
            for edge in self.graph.callers(*id) {
                if self
                    .registry
                    .get(edge.to)
                    .is_some_and(|symbol| self.is_test(symbol))
                {
                    tests.insert(edge.to);
                }
            }
        }
        let mut tests: Vec<TestId> = tests.into_iter().collect();
        tests.sort_unstable();
        tests
    }
}

const FLOW_KEYWORDS: &[&str] = &[
    "auth", "checkout", "create", "delete", "login", "logout", "payment", "publish", "register",
    "route", "signup", "sync", "welcome", "workflow",
];

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
