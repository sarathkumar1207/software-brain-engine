use sbe_common::{ContextPacket, FileEntry, IndexSnapshot, Symbol};
use sbe_context::{Budget, ContextCompilation, ContextCompiler, ContextPack};
use sbe_graph::SemanticGraph;
use sbe_impact::{ImpactAnalyzer, ImpactReport};
use sbe_symbols::SymbolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphReport {
    pub symbol: Symbol,
    pub dependencies: Vec<Symbol>,
    pub dependents: Vec<Symbol>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeAnalysisReport {
    pub query: String,
    pub matched_symbols: Vec<Symbol>,
    pub affected_symbols: Vec<Symbol>,
    pub impacted_files: Vec<ImpactedFile>,
    pub impacted_layers: Vec<LayerImpact>,
    pub impact_percentage: u32,
    pub token_estimate: TokenEstimate,
    pub llm_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactedFile {
    pub path: String,
    pub layer: CodeLayer,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerImpact {
    pub layer: CodeLayer,
    pub files: usize,
    pub symbols: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum CodeLayer {
    Auth,
    Middleware,
    Controller,
    Service,
    Dto,
    Database,
    Route,
    Config,
    Test,
    Ui,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenEstimate {
    pub without_sbe_tokens: u32,
    pub with_sbe_tokens: u32,
    pub saved_tokens: u32,
    pub reduction_percentage: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub query: String,
    pub query_time_ms: u128,
    pub indexed_files: usize,
    pub indexed_symbols: usize,
    pub impacted_files: usize,
    pub impacted_symbols: usize,
    pub impacted_layers: Vec<LayerImpact>,
    pub token_estimate: TokenEstimate,
}

pub struct QueryEngine {
    registry: SymbolRegistry,
    graph: SemanticGraph,
    root: PathBuf,
    files: HashMap<u64, FileEntry>,
    symbols: Vec<Symbol>,
}

impl QueryEngine {
    pub fn from_snapshot(snapshot: IndexSnapshot) -> Self {
        let root = PathBuf::from(&snapshot.root);
        let graph = SemanticGraph::from_snapshot(&snapshot);
        let files = snapshot
            .files
            .into_iter()
            .map(|file| (file.id, file))
            .collect();
        let symbols = snapshot.symbols;
        let registry = SymbolRegistry::build(symbols.clone());

        Self {
            registry,
            graph,
            root,
            files,
            symbols,
        }
    }

    pub fn inspect(&self, name: &str) -> Vec<ContextPacket> {
        self.registry
            .find_by_name(name)
            .into_iter()
            .map(|symbol| self.context_packet(symbol))
            .collect()
    }

    pub fn graph(&self, name: &str) -> Vec<GraphReport> {
        self.registry
            .find_by_name(name)
            .into_iter()
            .map(|symbol| GraphReport {
                symbol: symbol.clone(),
                dependencies: self.symbols_for_ids(self.graph.dependencies(symbol.id)),
                dependents: self.symbols_for_ids(self.graph.dependents(symbol.id)),
            })
            .collect()
    }

    pub fn impact(&self, name: &str) -> Vec<ImpactReport> {
        let analyzer = ImpactAnalyzer::new(&self.graph, &self.registry);
        self.registry
            .find_by_name(name)
            .into_iter()
            .map(|symbol| analyzer.analyze(symbol.id))
            .collect()
    }

    pub fn context(&self, name: &str, budget: Option<usize>) -> Vec<ContextPack> {
        let compiler = ContextCompiler::new(&self.graph, &self.registry);
        let budget = budget.map(Budget::new);
        self.registry
            .find_by_name(name)
            .into_iter()
            .filter_map(|symbol| compiler.compile(symbol.id, budget))
            .collect()
    }

    pub fn analyze_change(&self, query: &str) -> ChangeAnalysisReport {
        let analysis = self.analyze_change_parts(query);
        ChangeAnalysisReport {
            query: query.to_string(),
            matched_symbols: analysis.matched_symbols,
            affected_symbols: analysis.affected_symbols,
            impacted_files: analysis.impacted_files,
            impacted_layers: analysis.impacted_layers.clone(),
            impact_percentage: analysis.impact_percentage,
            token_estimate: analysis.token_estimate.clone(),
            llm_summary: llm_summary(query, &analysis.impacted_layers, &analysis.token_estimate),
        }
    }

    pub fn benchmark(&self, query: &str) -> BenchmarkReport {
        let started = std::time::Instant::now();
        let analysis = self.analyze_change_parts(query);
        BenchmarkReport {
            query: query.to_string(),
            query_time_ms: started.elapsed().as_millis(),
            indexed_files: self.files.len(),
            indexed_symbols: self.symbols.len(),
            impacted_files: analysis.impacted_files.len(),
            impacted_symbols: analysis.affected_symbols.len(),
            impacted_layers: analysis.impacted_layers,
            token_estimate: analysis.token_estimate,
        }
    }

    fn analyze_change_parts(&self, query: &str) -> ChangeAnalysisParts {
        let terms = tokenize_query(query);
        let matched_symbols = self.match_symbols(&terms);
        let analyzer = ImpactAnalyzer::new(&self.graph, &self.registry);
        let mut affected_by_id = HashMap::new();

        for symbol in &matched_symbols {
            affected_by_id.insert(symbol.id, symbol.clone());
            for affected in analyzer.analyze(symbol.id).affected {
                affected_by_id.insert(affected.id, affected);
            }
            for dependency in self.symbols_for_ids(self.graph.dependencies(symbol.id)) {
                affected_by_id.insert(dependency.id, dependency);
            }
        }

        let mut affected_symbols: Vec<Symbol> = affected_by_id.into_values().collect();
        affected_symbols.sort_by_key(|symbol| (symbol.file_id, symbol.range.start_line, symbol.id));

        let impacted_files = self.impacted_files(&affected_symbols);
        let impacted_layers = layer_impacts(&impacted_files);
        let impact_percentage = percentage(affected_symbols.len(), self.symbols.len());
        let token_estimate = self.token_estimate(&affected_symbols);

        ChangeAnalysisParts {
            matched_symbols,
            affected_symbols,
            impacted_files,
            impacted_layers,
            impact_percentage,
            token_estimate,
        }
    }

    fn context_packet(&self, symbol: &Symbol) -> ContextPacket {
        let direct_dependencies = self.symbols_for_ids(self.graph.dependencies(symbol.id));
        let analyzer = ImpactAnalyzer::new(&self.graph, &self.registry);
        let dependents = analyzer.analyze(symbol.id).affected;
        let file_path = self
            .files
            .get(&symbol.file_id)
            .map(|file| self.file_path(file).display().to_string())
            .unwrap_or_default();
        let line_count = symbol
            .range
            .end_line
            .saturating_sub(symbol.range.start_line)
            .saturating_add(1);

        ContextPacket {
            symbol: symbol.clone(),
            file_path,
            source_lines: (symbol.range.start_line, symbol.range.end_line),
            direct_dependencies,
            dependents,
            estimated_tokens: (line_count * 80) / 4,
        }
    }

    fn symbols_for_ids(&self, ids: Vec<u64>) -> Vec<Symbol> {
        ids.into_iter()
            .filter_map(|id| self.registry.get(id).cloned())
            .collect()
    }

    fn match_symbols(&self, terms: &[String]) -> Vec<Symbol> {
        let mut matched: Vec<Symbol> = self
            .symbols
            .iter()
            .filter(|symbol| {
                let file = self
                    .files
                    .get(&symbol.file_id)
                    .map(|file| file.relative_path.as_str())
                    .unwrap_or_default();
                let haystack = format!("{} {} {:?}", symbol.name, file, symbol.kind).to_lowercase();
                terms.iter().any(|term| haystack.contains(term))
            })
            .cloned()
            .collect();

        matched.sort_by_key(|symbol| (symbol.file_id, symbol.range.start_line, symbol.id));
        matched
    }

    fn impacted_files(&self, symbols: &[Symbol]) -> Vec<ImpactedFile> {
        let mut by_file: HashMap<u64, Vec<String>> = HashMap::new();
        for symbol in symbols {
            by_file
                .entry(symbol.file_id)
                .or_default()
                .push(symbol.name.clone());
        }

        let mut files: Vec<ImpactedFile> = by_file
            .into_iter()
            .filter_map(|(file_id, mut symbols)| {
                symbols.sort();
                symbols.dedup();
                let file = self.files.get(&file_id)?;
                Some(ImpactedFile {
                    path: file.relative_path.clone(),
                    layer: classify_layer(&file.relative_path),
                    symbols,
                })
            })
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        files
    }

    fn token_estimate(&self, focused_symbols: &[Symbol]) -> TokenEstimate {
        let without_sbe_tokens: u32 = self
            .files
            .values()
            .map(|file| estimate_file_tokens(self.file_path(file)))
            .sum();

        let focused_ranges = merged_ranges_by_file(focused_symbols);
        let symbol_tokens: u32 = focused_ranges
            .iter()
            .map(|(file_id, ranges)| {
                self.files
                    .get(file_id)
                    .map(|file| estimate_ranges_tokens(self.file_path(file), ranges))
                    .unwrap_or_default()
            })
            .sum();
        let focused_file_count = focused_ranges.len();
        let metadata_tokens = (focused_file_count as u32 * 40) + 120;
        let with_sbe_tokens = symbol_tokens.saturating_add(metadata_tokens);
        let saved_tokens = without_sbe_tokens.saturating_sub(with_sbe_tokens);
        let reduction_percentage = percentage(saved_tokens as usize, without_sbe_tokens as usize);

        TokenEstimate {
            without_sbe_tokens,
            with_sbe_tokens,
            saved_tokens,
            reduction_percentage,
        }
    }

    fn file_path(&self, file: &FileEntry) -> PathBuf {
        self.root.join(&file.relative_path)
    }
}

fn merged_ranges_by_file(symbols: &[Symbol]) -> HashMap<u64, Vec<(u32, u32)>> {
    let mut by_file: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
    for symbol in symbols {
        by_file
            .entry(symbol.file_id)
            .or_default()
            .push((symbol.range.start_line, symbol.range.end_line));
    }

    for ranges in by_file.values_mut() {
        ranges.sort_unstable();
        let mut merged: Vec<(u32, u32)> = Vec::new();
        for (start, end) in ranges.drain(..) {
            match merged.last_mut() {
                Some((_, merged_end)) if start <= merged_end.saturating_add(1) => {
                    *merged_end = (*merged_end).max(end);
                }
                _ => merged.push((start, end)),
            }
        }
        *ranges = merged;
    }

    by_file
}

struct ChangeAnalysisParts {
    matched_symbols: Vec<Symbol>,
    affected_symbols: Vec<Symbol>,
    impacted_files: Vec<ImpactedFile>,
    impacted_layers: Vec<LayerImpact>,
    impact_percentage: u32,
    token_estimate: TokenEstimate,
}

fn tokenize_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|term| term.len() > 1)
        .map(str::to_string)
        .collect()
}

fn classify_layer(path: &str) -> CodeLayer {
    let path = path.to_lowercase().replace('\\', "/");
    if path.contains("middleware") || path.contains("guard") {
        CodeLayer::Middleware
    } else if path.contains("controller") || path.contains("handler") {
        CodeLayer::Controller
    } else if path.contains("dto") || path.contains("schema") || path.contains("request") {
        CodeLayer::Dto
    } else if path.contains("repository")
        || path.contains("entity")
        || path.contains("model")
        || path.contains("migration")
        || path.contains("prisma")
        || path.contains("database")
        || path.contains("/db/")
    {
        CodeLayer::Database
    } else if path.contains("route") || path.contains("router") {
        CodeLayer::Route
    } else if path.contains("config") || path.contains("env") {
        CodeLayer::Config
    } else if path.contains("test") || path.contains("spec") {
        CodeLayer::Test
    } else if path.ends_with(".tsx") || path.contains("component") || path.contains("page") {
        CodeLayer::Ui
    } else if path.contains("/auth/") || path.contains("jwt") || path.contains("passport") {
        CodeLayer::Auth
    } else if path.contains("service") {
        CodeLayer::Service
    } else {
        CodeLayer::Unknown
    }
}

fn layer_impacts(files: &[ImpactedFile]) -> Vec<LayerImpact> {
    let mut by_layer: HashMap<CodeLayer, (usize, usize)> = HashMap::new();
    for file in files {
        let entry = by_layer.entry(file.layer).or_default();
        entry.0 += 1;
        entry.1 += file.symbols.len();
    }

    let mut impacts: Vec<LayerImpact> = by_layer
        .into_iter()
        .map(|(layer, (files, symbols))| LayerImpact {
            layer,
            files,
            symbols,
        })
        .collect();
    impacts.sort_by_key(|impact| impact.layer);
    impacts
}

fn estimate_file_tokens(path: impl AsRef<std::path::Path>) -> u32 {
    std::fs::read_to_string(path)
        .map(|source| (source.chars().count() as u32).saturating_add(3) / 4)
        .unwrap_or_default()
}

fn estimate_ranges_tokens(path: impl AsRef<std::path::Path>, ranges: &[(u32, u32)]) -> u32 {
    let Ok(source) = std::fs::read_to_string(path) else {
        return 0;
    };
    let lines: Vec<&str> = source.lines().collect();
    let chars: usize = ranges
        .iter()
        .map(|(start, end)| {
            let start = start.saturating_sub(1) as usize;
            let end = (*end as usize).min(lines.len());
            if start >= end {
                0
            } else {
                lines[start..end].iter().map(|line| line.len() + 1).sum()
            }
        })
        .sum();
    (chars as u32).saturating_add(3) / 4
}

fn percentage(part: usize, total: usize) -> u32 {
    if total == 0 {
        0
    } else {
        ((part as f64 / total as f64) * 100.0).round() as u32
    }
}

fn llm_summary(query: &str, layers: &[LayerImpact], tokens: &TokenEstimate) -> String {
    let layer_names: Vec<String> = layers
        .iter()
        .map(|impact| format!("{:?}", impact.layer))
        .collect();
    format!(
        "For change '{query}', send the matched symbols, impacted layers [{}], and dependency/impact lists instead of the full codebase. Estimated token reduction: {}%.",
        layer_names.join(", "),
        tokens.reduction_percentage
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{SourceRange, SymbolKind, Visibility};

    fn symbol(id: u64, name: &str, file_id: u64) -> Symbol {
        Symbol {
            id,
            content_hash: "hash".into(),
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

    #[test]
    fn classifies_auth_change_layers() {
        assert_eq!(classify_layer("src/auth/jwt.strategy.ts"), CodeLayer::Auth);
        assert_eq!(classify_layer("src/auth/auth.service.ts"), CodeLayer::Auth);
        assert_eq!(
            classify_layer("src/middleware/auth.middleware.ts"),
            CodeLayer::Middleware
        );
        assert_eq!(
            classify_layer("src/user/user.controller.ts"),
            CodeLayer::Controller
        );
        assert_eq!(classify_layer("src/dto/login.dto.ts"), CodeLayer::Dto);
        assert_eq!(classify_layer("src/db/user.entity.ts"), CodeLayer::Database);
    }

    #[test]
    fn analyzes_change_from_query_terms() {
        let snapshot = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![FileEntry {
                id: 1,
                path: "missing/auth.service.ts".into(),
                relative_path: "src/auth/auth.service.ts".into(),
                hash: "hash".into(),
                extension: "ts".into(),
            }],
            symbols: vec![symbol(10, "JwtAuthService", 1), symbol(11, "login", 1)],
            imports: vec![],
            edges: vec![],
        };
        let report = QueryEngine::from_snapshot(snapshot).analyze_change("jwt to passport");

        assert_eq!(report.matched_symbols.len(), 1);
        assert!(report
            .impacted_layers
            .iter()
            .any(|impact| impact.layer == CodeLayer::Auth));
        assert!(report.llm_summary.contains("token reduction"));
    }

    #[test]
    fn token_estimate_uses_snapshot_root_not_absolute_file_path() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("src/auth.ts"),
            "export function JwtAuthService() { return true; }\n",
        )
        .unwrap();
        let snapshot = IndexSnapshot {
            storage_version: 1,
            root: temp.path().to_string_lossy().to_string(),
            files: vec![FileEntry {
                id: 1,
                path: "moved/old/path/src/auth.ts".into(),
                relative_path: "src/auth.ts".into(),
                hash: "hash".into(),
                extension: "ts".into(),
            }],
            symbols: vec![symbol(10, "JwtAuthService", 1)],
            imports: vec![],
            edges: vec![],
        };

        let report = QueryEngine::from_snapshot(snapshot).benchmark("jwt");

        assert!(report.token_estimate.without_sbe_tokens > 0);
    }

    #[test]
    fn compiles_context_pack_by_symbol_name() {
        let snapshot = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![FileEntry {
                id: 1,
                path: "missing/users.ts".into(),
                relative_path: "src/users.ts".into(),
                hash: "hash".into(),
                extension: "ts".into(),
            }],
            symbols: vec![symbol(10, "createUser", 1), symbol(11, "saveUser", 1)],
            imports: vec![],
            edges: vec![sbe_common::Edge {
                from: 10,
                to: 11,
                relation: sbe_common::RelationType::References,
                range: None,
            }],
        };

        let packs = QueryEngine::from_snapshot(snapshot).context("createUser", Some(100));

        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].root_symbol, 10);
        assert!(packs[0]
            .symbols
            .iter()
            .any(|symbol| symbol.name == "saveUser"));
        assert!(packs[0].summary.contains("Calls saveUser"));
    }
}
