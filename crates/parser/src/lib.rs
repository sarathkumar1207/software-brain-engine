use sbe_common::{
    Edge, FileEntry, ImportRecord, ParsedFile, RelationType, SourceRange, Symbol, SymbolKind,
    Visibility,
};
use tree_sitter::{Node, Parser};

pub trait LanguagePlugin {
    fn supports(&self, extension: &str) -> bool;
    fn parse_file(&mut self, file: &FileEntry, source: &str) -> anyhow::Result<ParsedFile>;
}

pub struct TypeScriptParser {
    inner: Parser,
    python: PythonPlugin,
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self {
            inner: Parser::new(),
            python: PythonPlugin,
        }
    }
}

impl TypeScriptParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_file(&mut self, file: &FileEntry, source: &str) -> anyhow::Result<ParsedFile> {
        if self.python.supports(&file.extension) {
            return self.python.parse_file(file, source);
        }

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

#[derive(Debug, Clone, Copy, Default)]
pub struct PythonPlugin;

impl LanguagePlugin for PythonPlugin {
    fn supports(&self, extension: &str) -> bool {
        extension == "py"
    }

    fn parse_file(&mut self, file: &FileEntry, source: &str) -> anyhow::Result<ParsedFile> {
        Ok(parse_python_file(file, source))
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
            let exported = is_exported(node);
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
        "variable_declarator" => Some(SymbolKind::Variable),
        "type_alias_declaration" => Some(SymbolKind::TypeAlias),
        _ => None,
    }
}

fn declaration_name(node: Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|name| node_text(name, source).map(str::to_string))
}

fn parse_import(node: Node, source: &str, file_id: u64) -> ImportRecord {
    let text = node_text(node, source).unwrap_or_default();
    let module = find_child_kind(node, "string")
        .and_then(|string| quoted(node_text(string, source).unwrap_or_default()))
        .or_else(|| {
            text.rsplit_once(" from ")
                .and_then(|(_, module)| quoted(module))
        })
        .or_else(|| text.strip_prefix("import ").and_then(quoted))
        .unwrap_or_default()
        .to_string();

    let mut names = import_names_from_ast(node, source);
    if names.is_empty() {
        names = text
            .split(" from ")
            .next()
            .map(import_names)
            .unwrap_or_default();
    }

    ImportRecord {
        file_id,
        module,
        names,
        range: range(node),
    }
}

fn import_names(text: &str) -> Vec<String> {
    text.strip_prefix("import")
        .unwrap_or(text)
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

fn import_names_from_ast(node: Node, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    collect_import_identifiers(node, source, &mut names);
    names.sort();
    names.dedup();
    names
}

fn collect_import_identifiers(node: Node, source: &str, names: &mut Vec<String>) {
    if matches!(
        node.kind(),
        "identifier" | "shorthand_property_identifier" | "property_identifier"
    ) {
        if let Some(text) = node_text(node, source) {
            names.push(text.to_string());
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "string" {
            collect_import_identifiers(child, source, names);
        }
    }
}

fn find_child_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_child_kind(child, kind) {
            return Some(found);
        }
    }
    None
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
            if from.id != to.id
                && !is_containment_pair(from, to)
                && contains_identifier(&body, &to.name)
            {
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

fn is_containment_pair(from: &Symbol, to: &Symbol) -> bool {
    to.parent_symbol == Some(from.id)
        || from.parent_symbol == Some(to.id)
        || (from.file_id == to.file_id
            && from.range.start_line <= to.range.start_line
            && from.range.end_line >= to.range.end_line
            && from.range != to.range)
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

fn slice_by_range(source: &str, range: &SourceRange) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let start = range.start_line.saturating_sub(1) as usize;
    let end = (range.end_line as usize).min(lines.len());
    if start >= end {
        None
    } else {
        Some(lines[start..end].join("\n"))
    }
}

fn is_exported(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "export_statement" {
            return true;
        }
        current = parent.parent();
    }
    false
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
    let mut bytes = Vec::with_capacity(16);
    bytes.extend_from_slice(&file_id.to_le_bytes());
    bytes.extend_from_slice(&index.to_le_bytes());
    let hash = blake3::hash(&bytes);
    let mut id = [0_u8; 8];
    id.copy_from_slice(&hash.as_bytes()[0..8]);
    u64::from_le_bytes(id)
}

fn node_text<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    node.utf8_text(source.as_bytes()).ok()
}

#[derive(Debug, Clone, Copy)]
struct OpenPythonSymbol {
    indent: usize,
    symbol_index: usize,
}

fn parse_python_file(file: &FileEntry, source: &str) -> ParsedFile {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();
    let mut edges = Vec::new();
    let mut open_symbols: Vec<OpenPythonSymbol> = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let line_no = idx as u32 + 1;
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(import) = parse_python_import(trimmed, file.id, line_no) {
            imports.push(import);
        }

        let indent = python_indent(line);
        if let Some((name, declaration_kind, signature)) = python_declaration(trimmed) {
            close_python_symbols(&mut symbols, &mut open_symbols, indent, line_no);
            let parent_symbol = open_symbols
                .last()
                .map(|open| symbols[open.symbol_index].id);
            let kind = if declaration_kind == SymbolKind::Function
                && parent_symbol
                    .and_then(|id| symbols.iter().find(|symbol| symbol.id == id))
                    .is_some_and(|symbol| symbol.kind == SymbolKind::Class)
            {
                SymbolKind::Method
            } else {
                declaration_kind
            };
            let id = symbol_id(file.id, symbols.len() as u64);
            let exported = !name.starts_with('_');
            let symbol = Symbol {
                id,
                content_hash: String::new(),
                name,
                kind,
                file_id: file.id,
                range: SourceRange {
                    start_line: line_no,
                    end_line: line_no,
                    start_col: indent as u32,
                    end_col: line.len() as u32,
                },
                parent_symbol,
                visibility: if exported {
                    Visibility::Public
                } else {
                    Visibility::Private
                },
                signature: Some(signature),
                exported,
            };
            if let Some(parent_id) = parent_symbol {
                edges.push(Edge {
                    from: parent_id,
                    to: id,
                    relation: RelationType::Contains,
                    range: Some(symbol.range.clone()),
                });
            }
            symbols.push(symbol);
            open_symbols.push(OpenPythonSymbol {
                indent,
                symbol_index: symbols.len() - 1,
            });
        } else if indent == 0 {
            if let Some((name, signature)) = python_variable(trimmed) {
                let id = symbol_id(file.id, symbols.len() as u64);
                let end_col = line.len() as u32;
                symbols.push(Symbol {
                    id,
                    content_hash: blake3::hash(line.as_bytes()).to_hex().to_string(),
                    name: name.to_string(),
                    kind: SymbolKind::Variable,
                    file_id: file.id,
                    range: SourceRange {
                        start_line: line_no,
                        end_line: line_no,
                        start_col: 0,
                        end_col,
                    },
                    parent_symbol: None,
                    visibility: if name.starts_with('_') {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    },
                    signature: Some(signature.to_string()),
                    exported: !name.starts_with('_'),
                });
            }
        }
    }

    close_python_symbols(&mut symbols, &mut open_symbols, 0, lines.len() as u32 + 1);
    for symbol in &mut symbols {
        if symbol.content_hash.is_empty() {
            symbol.content_hash = hash_source_range(source, &symbol.range);
        }
    }
    add_local_reference_edges(source, &symbols, &mut edges);

    ParsedFile {
        file: file.clone(),
        symbols,
        imports,
        edges,
    }
}

fn close_python_symbols(
    symbols: &mut [Symbol],
    open_symbols: &mut Vec<OpenPythonSymbol>,
    indent: usize,
    line_no: u32,
) {
    while open_symbols
        .last()
        .is_some_and(|open| open.indent >= indent)
    {
        let open = open_symbols.pop().expect("open symbol exists");
        symbols[open.symbol_index].range.end_line = line_no.saturating_sub(1);
    }
}

fn python_declaration(trimmed: &str) -> Option<(String, SymbolKind, String)> {
    let (kind, rest) = if let Some(rest) = trimmed.strip_prefix("async def ") {
        (SymbolKind::Function, rest)
    } else if let Some(rest) = trimmed.strip_prefix("def ") {
        (SymbolKind::Function, rest)
    } else if let Some(rest) = trimmed.strip_prefix("class ") {
        (SymbolKind::Class, rest)
    } else {
        return None;
    };

    let name_end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    let name = &rest[..name_end];
    if name.is_empty() {
        return None;
    }
    Some((
        name.to_string(),
        kind,
        trimmed.trim_end_matches(':').to_string(),
    ))
}

fn python_variable(trimmed: &str) -> Option<(&str, &str)> {
    if trimmed.starts_with("if ") || trimmed.starts_with("for ") || trimmed.starts_with("while ") {
        return None;
    }
    let (left, _) = trimmed.split_once('=')?;
    if left.contains("==") || left.contains('!') || left.contains('<') || left.contains('>') {
        return None;
    }
    let name = left.split(':').next().unwrap_or(left).trim();
    if is_python_identifier(name) {
        Some((name, trimmed))
    } else {
        None
    }
}

fn parse_python_import(trimmed: &str, file_id: u64, line_no: u32) -> Option<ImportRecord> {
    let (module, names) = if let Some(rest) = trimmed.strip_prefix("from ") {
        let (module, names) = rest.split_once(" import ")?;
        (module.trim().to_string(), python_import_names(names))
    } else if let Some(rest) = trimmed.strip_prefix("import ") {
        let names = python_import_names(rest);
        (names.first().cloned().unwrap_or_default(), names)
    } else {
        return None;
    };

    Some(ImportRecord {
        file_id,
        module,
        names,
        range: SourceRange {
            start_line: line_no,
            end_line: line_no,
            start_col: 0,
            end_col: trimmed.len() as u32,
        },
    })
}

fn python_import_names(names: &str) -> Vec<String> {
    names
        .trim_matches(|c| c == '(' || c == ')')
        .split(',')
        .filter_map(|part| {
            let cleaned = part.trim();
            if cleaned.is_empty() || cleaned == "*" {
                return None;
            }
            let alias_or_name = cleaned
                .rsplit_once(" as ")
                .map(|(_, alias)| alias)
                .unwrap_or(cleaned);
            let name = alias_or_name
                .rsplit_once('.')
                .map(|(_, name)| name)
                .unwrap_or(alias_or_name)
                .trim();
            if is_python_identifier(name) {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn python_indent(line: &str) -> usize {
    line.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

fn is_python_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn hash_source_range(source: &str, range: &SourceRange) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let start = range.start_line.saturating_sub(1) as usize;
    let end = (range.end_line as usize).min(lines.len());
    let text = if start >= end {
        String::new()
    } else {
        lines[start..end].join("\n")
    };
    blake3::hash(text.as_bytes()).to_hex().to_string()
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
        let mut parser = TypeScriptParser::new();
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
        let mut parser = TypeScriptParser::new();
        let parsed = parser.parse_file(&file("tsx"), source).unwrap();

        assert!(parsed.symbols.iter().any(|sym| sym.name == "View"));
    }

    #[test]
    fn default_parser_parses_files() {
        let source = r#"export function ready() { return true; }"#;
        let mut parser = TypeScriptParser::default();
        let parsed = parser.parse_file(&file("ts"), source).unwrap();

        assert!(parsed.symbols.iter().any(|sym| sym.name == "ready"));
    }

    #[test]
    fn export_detection_uses_ast_not_prefix_text() {
        let source = r#"
// export this later
function privateFn() {}
export function publicFn() {}
"#;
        let mut parser = TypeScriptParser::new();
        let parsed = parser.parse_file(&file("ts"), source).unwrap();

        let private_fn = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.name == "privateFn")
            .unwrap();
        let public_fn = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.name == "publicFn")
            .unwrap();

        assert!(!private_fn.exported);
        assert!(public_fn.exported);
    }

    #[test]
    fn extracts_variables_and_type_aliases() {
        let source = r#"
export const jwtConfig = { secret: "x" };
export type UserId = string;
"#;
        let mut parser = TypeScriptParser::new();
        let parsed = parser.parse_file(&file("ts"), source).unwrap();

        assert!(parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "jwtConfig" && symbol.kind == SymbolKind::Variable));
        assert!(parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "UserId" && symbol.kind == SymbolKind::TypeAlias));
    }

    #[test]
    fn import_name_parser_preserves_identifiers_containing_import() {
        let source = r#"import { importUser } from "./users";"#;
        let mut parser = TypeScriptParser::new();
        let parsed = parser.parse_file(&file("ts"), source).unwrap();

        assert_eq!(parsed.imports[0].names, vec!["importUser"]);
    }

    #[test]
    fn local_reference_edges_do_not_duplicate_contains_edges() {
        let source = r#"
export class Service {
  run() { return 1; }
}
"#;
        let mut parser = TypeScriptParser::new();
        let parsed = parser.parse_file(&file("ts"), source).unwrap();

        assert!(!parsed
            .edges
            .iter()
            .any(|edge| edge.relation == RelationType::References));
    }

    #[test]
    fn symbol_ids_do_not_overlap_at_legacy_multiplier_boundary() {
        assert_ne!(symbol_id(1, 10_000), symbol_id(2, 0));
    }

    #[test]
    fn unicode_source_ranges_still_slice_reference_bodies() {
        let source = r#"
export function helper() { return "\u0ba4\u0bae\u0bbf\u0bb4\u0bcd"; }
export function caller() {
  return helper();
}
"#;
        let mut parser = TypeScriptParser::new();
        let parsed = parser.parse_file(&file("ts"), source).unwrap();

        assert!(parsed
            .edges
            .iter()
            .any(|edge| edge.relation == RelationType::References));
    }

    #[test]
    fn python_plugin_extracts_classes_functions_methods_imports_and_references() {
        let source = r#"
from fastapi import FastAPI
import pydantic as pd

app = FastAPI()

class UserService:
    def create_user(self):
        return helper()

def helper():
    return "ok"
"#;
        let mut parser = TypeScriptParser::new();
        let parsed = parser.parse_file(&file("py"), source).unwrap();

        assert!(parsed
            .imports
            .iter()
            .any(|import| import.module == "fastapi"));
        assert!(parsed
            .symbols
            .iter()
            .any(|symbol| { symbol.name == "UserService" && symbol.kind == SymbolKind::Class }));
        assert!(parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "create_user" && symbol.kind == SymbolKind::Method));
        assert!(parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "helper" && symbol.kind == SymbolKind::Function));
        assert!(parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "app" && symbol.kind == SymbolKind::Variable));
        assert!(parsed
            .edges
            .iter()
            .any(|edge| edge.relation == RelationType::References));
    }
}
