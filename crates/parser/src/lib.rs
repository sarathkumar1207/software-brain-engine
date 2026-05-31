use sbe_common::{
    Edge, FileEntry, ImportRecord, ParsedFile, RelationType, SourceRange, Symbol, SymbolKind,
    Visibility,
};
use tree_sitter::{Node, Parser};

pub struct TypeScriptParser {
    inner: Parser,
}

impl TypeScriptParser {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            inner: Parser::new(),
        })
    }

    pub fn parse_file(&mut self, file: &FileEntry, source: &str) -> anyhow::Result<ParsedFile> {
        let language = if file.extension == "tsx" {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        };
        self.inner.set_language(&language)?;

        let Some(tree) = self.inner.parse(source, None) else {
            return Ok(ParsedFile {
                file: file.clone(),
                symbols: Vec::new(),
                imports: Vec::new(),
                edges: Vec::new(),
            });
        };

        let root = tree.root_node();
        let mut context = WalkContext::new(source, file.id);
        context.walk_declarations(root, None);
        add_local_reference_edges(source, &context.symbols, &mut context.edges);

        Ok(ParsedFile {
            file: file.clone(),
            symbols: context.symbols,
            imports: context.imports,
            edges: context.edges,
        })
    }
}

struct WalkContext<'a> {
    source: &'a str,
    file_id: u64,
    next: u64,
    symbols: Vec<Symbol>,
    imports: Vec<ImportRecord>,
    edges: Vec<Edge>,
}

impl<'a> WalkContext<'a> {
    fn new(source: &'a str, file_id: u64) -> Self {
        Self {
            source,
            file_id,
            next: 0,
            symbols: Vec::new(),
            imports: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn walk_declarations(&mut self, node: Node, parent: Option<u64>) {
        if node.kind() == "import_statement" {
            self.imports
                .push(parse_import(node, self.source, self.file_id));
        }

        let declaration = symbol_kind(node).and_then(|kind| {
            let name = declaration_name(node, self.source)?;
            let exported = is_exported(node, self.source);
            let id = symbol_id(self.file_id, self.next);
            self.next += 1;
            Some(Symbol {
                id,
                content_hash: node_hash(node, self.source),
                name,
                kind,
                file_id: self.file_id,
                range: range(node),
                parent_symbol: parent,
                visibility: if exported {
                    Visibility::Public
                } else {
                    Visibility::Private
                },
                signature: signature(node, self.source),
                exported,
            })
        });

        let current_parent = if let Some(symbol) = declaration {
            let id = symbol.id;
            if let Some(parent_id) = parent {
                self.edges.push(Edge {
                    from: parent_id,
                    to: id,
                    relation: RelationType::Contains,
                    range: Some(symbol.range.clone()),
                });
            }
            self.symbols.push(symbol);
            Some(id)
        } else {
            parent
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_declarations(child, current_parent);
        }
    }
}

fn symbol_kind(node: Node) -> Option<SymbolKind> {
    match node.kind() {
        "function_declaration" => Some(SymbolKind::Function),
        "class_declaration" => Some(SymbolKind::Class),
        "method_definition" | "method_signature" => Some(SymbolKind::Method),
        "interface_declaration" => Some(SymbolKind::Interface),
        "enum_declaration" => Some(SymbolKind::Enum),
        "internal_module" | "module" => Some(SymbolKind::Module),
        _ => None,
    }
}

fn declaration_name(node: Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|name| node_text(name, source).map(str::to_string))
}

fn parse_import(node: Node, source: &str, file_id: u64) -> ImportRecord {
    let text = node_text(node, source).unwrap_or_default();
    let module = text
        .rsplit_once(" from ")
        .and_then(|(_, module)| quoted(module))
        .or_else(|| text.strip_prefix("import ").and_then(quoted))
        .unwrap_or_default()
        .to_string();

    let names = text
        .split(" from ")
        .next()
        .map(import_names)
        .unwrap_or_default();

    ImportRecord {
        file_id,
        module,
        names,
        range: range(node),
    }
}

fn import_names(text: &str) -> Vec<String> {
    text.replace("import", "")
        .replace(['{', '}', '*'], " ")
        .replace(" as ", " ")
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|part| {
            !part.is_empty()
                && *part != "type"
                && *part != "from"
                && !part.starts_with('"')
                && !part.starts_with('\'')
        })
        .map(str::to_string)
        .collect()
}

fn quoted(text: &str) -> Option<&str> {
    let start = text.find(['"', '\''])?;
    let quote = text.as_bytes()[start] as char;
    let rest = &text[start + 1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn add_local_reference_edges(source: &str, symbols: &[Symbol], edges: &mut Vec<Edge>) {
    for from in symbols {
        let Some(body) = slice_by_range(source, &from.range) else {
            continue;
        };
        for to in symbols {
            if from.id != to.id && contains_identifier(body, &to.name) {
                edges.push(Edge {
                    from: from.id,
                    to: to.id,
                    relation: RelationType::References,
                    range: None,
                });
            }
        }
    }
}

fn contains_identifier(source: &str, needle: &str) -> bool {
    source.match_indices(needle).any(|(idx, _)| {
        is_left_boundary(source, idx) && is_right_boundary(source, idx + needle.len())
    })
}

fn is_left_boundary(source: &str, idx: usize) -> bool {
    source[..idx]
        .chars()
        .next_back()
        .map(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '$')
        .unwrap_or(true)
}

fn is_right_boundary(source: &str, idx: usize) -> bool {
    source[idx..]
        .chars()
        .next()
        .map(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '$')
        .unwrap_or(true)
}

fn slice_by_range<'a>(source: &'a str, range: &SourceRange) -> Option<&'a str> {
    let mut offset = 0_usize;
    let mut start = None;
    let mut end = None;
    for (line_idx, line) in source.lines().enumerate() {
        let line_no = line_idx as u32 + 1;
        if line_no == range.start_line {
            start = Some(offset);
        }
        if line_no == range.end_line {
            end = Some(offset + line.len());
            break;
        }
        offset += line.len() + 1;
    }
    start
        .zip(end)
        .and_then(|(start, end)| source.get(start..end))
}

fn is_exported(node: Node, source: &str) -> bool {
    let prefix_start = node.start_byte().saturating_sub(32);
    source
        .get(prefix_start..node.start_byte())
        .map(|prefix| prefix.contains("export"))
        .unwrap_or(false)
}

fn signature(node: Node, source: &str) -> Option<String> {
    node_text(node, source).and_then(|text| {
        let first = text.lines().next()?.trim();
        Some(first.trim_end_matches('{').trim().to_string())
    })
}

fn range(node: Node) -> SourceRange {
    SourceRange {
        start_line: node.start_position().row as u32 + 1,
        end_line: node.end_position().row as u32 + 1,
        start_col: node.start_position().column as u32,
        end_col: node.end_position().column as u32,
    }
}

fn node_hash(node: Node, source: &str) -> String {
    blake3::hash(node_text(node, source).unwrap_or_default().as_bytes())
        .to_hex()
        .to_string()
}

fn symbol_id(file_id: u64, index: u64) -> u64 {
    file_id.wrapping_mul(10_000).wrapping_add(index + 1)
}

fn node_text<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    node.utf8_text(source.as_bytes()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(extension: &str) -> FileEntry {
        FileEntry {
            id: 7,
            path: format!("src/app.{extension}"),
            relative_path: format!("src/app.{extension}"),
            hash: "hash".into(),
            extension: extension.into(),
        }
    }

    #[test]
    fn extracts_symbols_and_imports() {
        let source = r#"
import { helper } from "./helper";
export interface User { name: string }
export class Service {
  run() { helper(); }
}
function local() { return new Service(); }
"#;
        let mut parser = TypeScriptParser::new().unwrap();
        let parsed = parser.parse_file(&file("ts"), source).unwrap();

        assert!(parsed
            .imports
            .iter()
            .any(|import| import.module == "./helper"));
        assert!(parsed.symbols.iter().any(|sym| sym.name == "User"));
        assert!(parsed.symbols.iter().any(|sym| sym.name == "Service"));
        assert!(parsed.symbols.iter().any(|sym| sym.name == "run"));
        assert!(parsed.symbols.iter().any(|sym| sym.name == "local"));
    }

    #[test]
    fn parses_tsx_without_panicking() {
        let source = r#"export function View() { return <main>Hello</main>; }"#;
        let mut parser = TypeScriptParser::new().unwrap();
        let parsed = parser.parse_file(&file("tsx"), source).unwrap();

        assert!(parsed.symbols.iter().any(|sym| sym.name == "View"));
    }
}
