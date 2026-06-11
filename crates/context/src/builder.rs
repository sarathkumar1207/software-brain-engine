use crate::pack::{CodeRange, ContextMetrics, ContextPack, ContextSymbol, DependencyPath};
use sbe_graph::SymbolId;

#[derive(Debug, Clone, Default)]
pub struct ContextPackBuilder {
    root_symbol: Option<SymbolId>,
    summary: Option<String>,
    symbols: Vec<ContextSymbol>,
    dependencies: Vec<DependencyPath>,
    callers: Vec<SymbolId>,
    code_ranges: Vec<CodeRange>,
    metrics: Option<ContextMetrics>,
}

impl ContextPackBuilder {
    pub fn new(root_symbol: SymbolId) -> Self {
        Self {
            root_symbol: Some(root_symbol),
            ..Self::default()
        }
    }

    pub fn summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }

    pub fn symbols(mut self, symbols: Vec<ContextSymbol>) -> Self {
        self.symbols = symbols;
        self
    }

    pub fn dependencies(mut self, dependencies: Vec<DependencyPath>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn callers(mut self, callers: Vec<SymbolId>) -> Self {
        self.callers = callers;
        self
    }

    pub fn code_ranges(mut self, code_ranges: Vec<CodeRange>) -> Self {
        self.code_ranges = code_ranges;
        self
    }

    pub fn metrics(mut self, metrics: ContextMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn build(self) -> ContextPack {
        ContextPack {
            root_symbol: self.root_symbol.expect("root symbol is required"),
            summary: self.summary.unwrap_or_default(),
            symbols: self.symbols,
            dependencies: self.dependencies,
            callers: self.callers,
            code_ranges: self.code_ranges,
            metrics: self.metrics.unwrap_or(ContextMetrics {
                symbols_selected: 0,
                files_selected: 0,
                estimated_tokens: 0,
                context_reduction_percent: 0.0,
            }),
        }
    }
}
