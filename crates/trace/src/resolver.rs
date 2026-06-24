use crate::path::TracePath;
use sbe_common::{IndexSnapshot, Symbol};
use sbe_graph::SemanticGraph;
use sbe_storage::Store;
use sbe_symbols::SymbolRegistry;
use std::path::PathBuf;

pub struct TraceResolver {
    registry: SymbolRegistry,
    graph: SemanticGraph,
}

impl TraceResolver {
    pub fn from_repo(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let snapshot = Store::open_existing(root.into())?.read_snapshot()?;
        Ok(Self::from_snapshot(snapshot))
    }

    pub fn from_snapshot(snapshot: IndexSnapshot) -> Self {
        Self {
            registry: SymbolRegistry::build(snapshot.symbols.clone()),
            graph: SemanticGraph::from_snapshot(&snapshot),
        }
    }

    pub fn trace(&self, query: &str) -> Vec<TracePath> {
        self.find_symbols(query)
            .into_iter()
            .map(|symbol| self.trace_symbol(symbol))
            .collect()
    }

    fn trace_symbol(&self, target: &Symbol) -> TracePath {
        let mut chain = vec![target.id];
        let mut current = target.id;
        let mut seen = std::collections::HashSet::from([current]);

        while let Some(next) = self
            .graph
            .dependents(current)
            .into_iter()
            .filter(|id| seen.insert(*id))
            .min_by_key(|id| self.symbol_name(*id).unwrap_or(""))
        {
            chain.push(next);
            current = next;
        }

        chain.reverse();
        let mut names: Vec<String> = chain
            .iter()
            .filter_map(|id| self.symbol_name(*id).map(str::to_string))
            .collect();

        if names.len() == 1 {
            let mut downstream = self
                .graph
                .dependencies(target.id)
                .into_iter()
                .filter_map(|id| self.symbol_name(id).map(str::to_string))
                .collect::<Vec<_>>();
            downstream.sort();
            downstream.dedup();
            names.extend(downstream.into_iter().take(4));
        }

        let root = names
            .first()
            .cloned()
            .unwrap_or_else(|| target.name.clone());
        let path = names.into_iter().skip(1).collect();
        TracePath { root, path }
    }

    fn find_symbols(&self, query: &str) -> Vec<&Symbol> {
        let exact = self.registry.find_by_name(query);
        if !exact.is_empty() {
            return exact;
        }
        let fallback = query.rsplit('.').next().unwrap_or(query);
        self.registry.find_by_name(fallback)
    }

    fn symbol_name(&self, id: u64) -> Option<&str> {
        self.registry.get(id).map(|symbol| symbol.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{Edge, FileEntry, RelationType, SourceRange, SymbolKind, Visibility};

    fn symbol(id: u64, name: &str) -> Symbol {
        Symbol {
            id,
            content_hash: name.into(),
            name: name.into(),
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
    fn traces_upstream_dependents_to_root() {
        let snapshot = IndexSnapshot {
            storage_version: 1,
            root: ".".into(),
            files: vec![FileEntry {
                id: 1,
                path: "src/app.ts".into(),
                relative_path: "src/app.ts".into(),
                hash: "hash".into(),
                extension: "ts".into(),
            }],
            symbols: vec![
                symbol(1, "Route"),
                symbol(2, "AuthController"),
                symbol(3, "login"),
            ],
            imports: vec![],
            edges: vec![
                Edge {
                    from: 1,
                    to: 2,
                    relation: RelationType::References,
                    range: None,
                },
                Edge {
                    from: 2,
                    to: 3,
                    relation: RelationType::References,
                    range: None,
                },
            ],
        };

        let traces = TraceResolver::from_snapshot(snapshot).trace("AuthService.login");

        assert_eq!(traces[0].root, "Route");
        assert_eq!(traces[0].path, vec!["AuthController", "login"]);
    }
}
