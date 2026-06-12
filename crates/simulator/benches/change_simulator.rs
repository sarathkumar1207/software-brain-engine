use sbe_common::{
    Edge, FileEntry, IndexSnapshot, RelationType, SourceRange, Symbol, SymbolKind, Visibility,
};
use sbe_graph::SemanticGraph;
use sbe_simulator::{ChangeSimulation, ChangeSimulator, SimulationOperation, SimulationRequest};
use sbe_symbols::SymbolRegistry;
use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

fn main() {
    const SYMBOLS: u64 = 10_000;
    const RUNS: u32 = 50;
    let mut symbols = Vec::with_capacity(SYMBOLS as usize);
    let mut edges = Vec::with_capacity((SYMBOLS * 2) as usize);
    let mut files = Vec::with_capacity(200);

    for file_id in 1..=200 {
        files.push(FileEntry {
            id: file_id,
            path: format!("src/module-{file_id}.ts"),
            relative_path: format!("src/module-{file_id}.ts"),
            hash: file_id.to_string(),
            extension: "ts".into(),
        });
    }
    for id in 1..=SYMBOLS {
        symbols.push(Symbol {
            id,
            content_hash: id.to_string(),
            name: if id % 997 == 0 {
                format!("CheckoutFlow{id}")
            } else {
                format!("symbol{id}")
            },
            kind: SymbolKind::Function,
            file_id: (id % 200) + 1,
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
        });
        if id < SYMBOLS {
            edges.push(edge(id, id + 1));
        }
        if id + 101 <= SYMBOLS {
            edges.push(edge(id, id + 101));
        }
    }

    let snapshot = IndexSnapshot {
        storage_version: 1,
        root: ".".into(),
        files: files.clone(),
        symbols: symbols.clone(),
        imports: vec![],
        edges,
    };
    let graph = SemanticGraph::from_snapshot(&snapshot);
    let registry = SymbolRegistry::build(symbols);
    let files: HashMap<_, _> = files.into_iter().map(|file| (file.id, file)).collect();
    let simulator = ChangeSimulator::new(&graph, &registry, &files);
    let request = SimulationRequest {
        operation: SimulationOperation::Modify,
        target_symbol: SYMBOLS / 2,
        max_depth: 6,
    };

    let started = Instant::now();
    for _ in 0..RUNS {
        black_box(
            simulator
                .simulate(request)
                .expect("simulation must succeed"),
        );
    }
    let elapsed = started.elapsed();
    let average = elapsed / RUNS;
    println!(
        "change simulator: {SYMBOLS} symbols, {RUNS} runs, average {:?}",
        average
    );
    assert!(
        average.as_millis() < 100,
        "average simulation exceeded 100ms: {average:?}"
    );
}

fn edge(from: u64, to: u64) -> Edge {
    Edge {
        from,
        to,
        relation: RelationType::References,
        range: None,
    }
}
