use sbe_common::{Symbol, SymbolKind};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolRegistry {
    by_id: HashMap<u64, Symbol>,
    by_name: HashMap<String, Vec<u64>>,
    by_file: HashMap<u64, Vec<u64>>,
    by_kind: HashMap<SymbolKind, Vec<u64>>,
}

impl SymbolRegistry {
    pub fn build(symbols: Vec<Symbol>) -> Self {
        let mut by_id = HashMap::new();
        let mut by_name: HashMap<String, Vec<u64>> = HashMap::new();
        let mut by_file: HashMap<u64, Vec<u64>> = HashMap::new();
        let mut by_kind: HashMap<SymbolKind, Vec<u64>> = HashMap::new();

        for symbol in symbols {
            by_name
                .entry(symbol.name.clone())
                .or_default()
                .push(symbol.id);
            by_file.entry(symbol.file_id).or_default().push(symbol.id);
            by_kind.entry(symbol.kind).or_default().push(symbol.id);
            by_id.insert(symbol.id, symbol);
        }

        Self {
            by_id,
            by_name,
            by_file,
            by_kind,
        }
    }

    pub fn get(&self, id: u64) -> Option<&Symbol> {
        self.by_id.get(&id)
    }

    pub fn find_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.by_name
            .get(name)
            .into_iter()
            .flatten()
            .filter_map(|id| self.by_id.get(id))
            .collect()
    }

    pub fn by_file(&self, file_id: u64) -> Vec<&Symbol> {
        self.by_file
            .get(&file_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.by_id.get(id))
            .collect()
    }

    pub fn by_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.by_kind
            .get(&kind)
            .into_iter()
            .flatten()
            .filter_map(|id| self.by_id.get(id))
            .collect()
    }

    pub fn all(&self) -> impl Iterator<Item = &Symbol> {
        self.by_id.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{SourceRange, Visibility};

    fn symbol(id: u64, name: &str, file_id: u64) -> Symbol {
        Symbol {
            id,
            content_hash: "hash".into(),
            name: name.into(),
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

    #[test]
    fn indexes_by_name_and_file() {
        let registry = SymbolRegistry::build(vec![symbol(1, "run", 10), symbol(2, "run", 11)]);

        assert_eq!(registry.find_by_name("run").len(), 2);
        assert_eq!(registry.by_file(10).len(), 1);
    }
}
