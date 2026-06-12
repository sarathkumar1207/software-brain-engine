use clap::{Args, Parser, Subcommand};
use sbe_context::ContextPack;
use sbe_impact::ImpactReport;
use sbe_indexer::{IndexReport, Indexer};
use sbe_query::{BenchmarkReport, QueryEngine};
use sbe_simulator::{RiskLevel, SimulationOperation, SimulationReport};
use sbe_storage::Store;
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "sbe", version, about = "Software Brain Engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create the .sbe directory and manifest.
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Scan and index a TypeScript, TSX, or Python repository.
    Scan {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Inspect symbols by name.
    Inspect {
        name: String,
        #[arg(long)]
        json: bool,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Show dependencies and dependents for symbols by name.
    Graph {
        name: String,
        #[arg(long)]
        json: bool,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Show transitive dependents affected by changing a symbol.
    Impact {
        name: String,
        #[arg(long)]
        json: bool,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Compile a minimal AI-ready context pack for a symbol.
    Context {
        name: String,
        #[arg(long)]
        budget: Option<usize>,
        #[arg(long)]
        json: bool,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Predict graph impact, risk, affected flows, and tests before editing code.
    Simulate {
        #[command(subcommand)]
        operation: SimulationCommands,
    },
    /// Incrementally update the existing index from changed files.
    Update {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Analyze a planned change for AI-focused impact and token savings.
    AnalyzeChange {
        query: String,
        #[arg(long)]
        json: bool,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Benchmark token savings and query speed for a planned change.
    Benchmark {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Run scan plus benchmark and write a validation report under .sbe/reports.
    Validate {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value = "jwt to passport")]
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Check project index health and stale files.
    Doctor {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Export the binary index to readable JSON for debugging.
    ExportJson {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Print build and storage metadata.
    Version {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum SimulationCommands {
    /// Predict the impact of modifying a symbol in place.
    Modify(SimulationArgs),
    /// Predict the impact of deleting a symbol.
    Delete(SimulationArgs),
    /// Predict the impact of replacing a symbol or implementation.
    Replace(SimulationArgs),
}

#[derive(Debug, Args)]
struct SimulationArgs {
    name: String,
    #[arg(long, default_value_t = 6)]
    max_depth: usize,
    #[arg(long)]
    json: bool,
    /// Persist the JSON report under .sbe/reports for later comparison.
    #[arg(long)]
    record: bool,
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[derive(Debug, Serialize)]
struct ValidationReport {
    path: String,
    scan: IndexReport,
    benchmark: BenchmarkReport,
    report_path: String,
}

#[derive(Debug, Serialize)]
struct VersionReport {
    name: &'static str,
    version: &'static str,
    storage_version: u32,
    target: &'static str,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let store = Indexer::init(&path)?;
            println!("initialized {}", store.root().display());
        }
        Commands::Scan { path, json } => {
            let mut indexer = Indexer::new(path)?;
            let report = indexer.run()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("SBE scan complete");
                println!("  indexed files : {}", report.files_scanned);
                println!(
                    "  graph         : {} symbols, {} imports, {} edges",
                    report.symbols_found, report.imports_found, report.edges_found
                );
                println!("  read          : {} bytes", report.bytes_read);
                println!("  index size    : {} bytes", report.index_size_bytes);
                println!("  elapsed       : {} ms", report.elapsed_ms);
                println!("  storage       : {}", report.storage_path);
                println!("  skipped dirs  : {}", report.skipped_dirs.len());
                if !report.warnings.is_empty() {
                    println!("  warnings      : {}", report.warnings.len());
                    for warning in report.warnings.iter().take(5) {
                        println!("    - {warning}");
                    }
                }
            }
        }
        Commands::Inspect { name, json, path } => {
            let packets = query_engine(path)?.inspect(&name);
            if json {
                println!("{}", serde_json::to_string_pretty(&packets)?);
            } else if packets.is_empty() {
                println!("no symbols found for {name}");
            } else {
                for packet in packets {
                    println!(
                        "{} {:?} {}:{}-{}",
                        packet.symbol.name,
                        packet.symbol.kind,
                        packet.file_path,
                        packet.source_lines.0,
                        packet.source_lines.1
                    );
                    println!(
                        "dependencies: {}  dependents: {}  tokens: ~{}",
                        packet.direct_dependencies.len(),
                        packet.dependents.len(),
                        packet.estimated_tokens
                    );
                }
            }
        }
        Commands::Graph { name, json, path } => {
            let reports = query_engine(path)?.graph(&name);
            if json {
                println!("{}", serde_json::to_string_pretty(&reports)?);
            } else if reports.is_empty() {
                println!("no symbols found for {name}");
            } else {
                for report in reports {
                    println!("{} {:?}", report.symbol.name, report.symbol.kind);
                    println!("dependencies:");
                    for dependency in report.dependencies {
                        println!("  -> {} {:?}", dependency.name, dependency.kind);
                    }
                    println!("dependents:");
                    for dependent in report.dependents {
                        println!("  <- {} {:?}", dependent.name, dependent.kind);
                    }
                }
            }
        }
        Commands::Impact { name, json, path } => {
            let reports = query_engine(path)?.impact(&name);
            if json {
                println!("{}", serde_json::to_string_pretty(&reports)?);
            } else if reports.is_empty() {
                println!("no symbols found for {name}");
            } else {
                print_impact_summary(&reports);
            }
        }
        Commands::Context {
            name,
            budget,
            json,
            path,
        } => {
            let packs = query_engine(path)?.context(&name, budget);
            if json {
                println!("{}", serde_json::to_string_pretty(&packs)?);
            } else if packs.is_empty() {
                println!("no symbols found for {name}");
            } else {
                for (idx, pack) in packs.iter().enumerate() {
                    if idx > 0 {
                        println!();
                        println!("---");
                        println!();
                    }
                    print_context_pack(pack);
                }
            }
        }
        Commands::Simulate { operation } => {
            let (operation, args) = match operation {
                SimulationCommands::Modify(args) => (SimulationOperation::Modify, args),
                SimulationCommands::Delete(args) => (SimulationOperation::Delete, args),
                SimulationCommands::Replace(args) => (SimulationOperation::Replace, args),
            };
            let path = args.path;
            let engine = query_engine(path.clone())?;
            let reports = engine.simulate(&args.name, operation, args.max_depth);
            let recorded_path = if args.record && !reports.is_empty() {
                let store = Store::open_existing(&path)?;
                let operation_name = match operation {
                    SimulationOperation::Modify => "modify",
                    SimulationOperation::Delete => "delete",
                    SimulationOperation::Replace => "replace",
                };
                let symbol_name: String = args
                    .name
                    .chars()
                    .map(|character| {
                        if character.is_ascii_alphanumeric() || character == '-' {
                            character
                        } else {
                            '_'
                        }
                    })
                    .collect();
                Some(store.write_report_json(
                    &format!("simulation-{operation_name}-{symbol_name}-latest.json"),
                    &reports,
                )?)
            } else {
                None
            };
            if args.json {
                println!("{}", serde_json::to_string_pretty(&reports)?);
            } else if reports.is_empty() {
                println!("no symbols found for {}", args.name);
            } else {
                for (index, report) in reports.iter().enumerate() {
                    if index > 0 {
                        println!();
                        println!("---");
                        println!();
                    }
                    print_simulation_report(&engine, report);
                }
                if let Some(path) = recorded_path {
                    println!();
                    println!("Recorded: {}", path.display());
                }
            }
        }
        Commands::Update { path, json } => {
            let mut indexer = Indexer::new(path)?;
            let report = indexer.update()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("SBE update complete");
                println!("  changed files : {}", report.changed_files.len());
                println!(
                    "  symbols       : {} added, {} modified, {} removed",
                    report.added_symbols, report.modified_symbols, report.removed_symbols
                );
                println!(
                    "  affected      : {} symbols",
                    report.affected_symbols.len()
                );
                println!("  affected files: {}", report.affected_files);
                println!("  graph         : {} edges", report.edges_found);
                println!("  elapsed       : {} ms", report.elapsed_ms);
                println!("  storage       : {}", report.storage_path);
                if !report.warnings.is_empty() {
                    println!("  warnings      : {}", report.warnings.len());
                    for warning in report.warnings.iter().take(5) {
                        println!("    - {warning}");
                    }
                }
            }
        }
        Commands::AnalyzeChange { query, json, path } => {
            let report = query_engine(path)?.analyze_change(&query);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("change: {}", report.query);
                println!(
                    "impact: {}% of indexed symbols ({} matched, {} affected)",
                    report.impact_percentage,
                    report.matched_symbols.len(),
                    report.affected_symbols.len()
                );
                println!(
                    "tokens: without SBE ~{}, with SBE ~{}, saved ~{} ({}%)",
                    report.token_estimate.without_sbe_tokens,
                    report.token_estimate.with_sbe_tokens,
                    report.token_estimate.saved_tokens,
                    report.token_estimate.reduction_percentage
                );
                println!("layers:");
                for layer in &report.impacted_layers {
                    println!(
                        "  {:?}: {} files, {} symbols",
                        layer.layer, layer.files, layer.symbols
                    );
                }
                println!("files:");
                for file in &report.impacted_files {
                    println!(
                        "  {:?}: {} [{}]",
                        file.layer,
                        file.path,
                        file.symbols.join(", ")
                    );
                }
                println!("llm: {}", report.llm_summary);
            }
        }
        Commands::Benchmark { path, query, json } => {
            let report = query_engine(path)?.benchmark(&query);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_benchmark(&report);
            }
        }
        Commands::Validate { path, query, json } => {
            let mut indexer = Indexer::new(&path)?;
            let scan = indexer.run()?;
            let benchmark = query_engine(path.clone())?.benchmark(&query);
            let mut report = ValidationReport {
                path: path.display().to_string(),
                scan,
                benchmark,
                report_path: String::new(),
            };
            let store = Store::open_or_create(&path)?;
            let report_path = store.report_path("validation-latest.json");
            report.report_path = report_path.display().to_string();
            store.write_report_json("validation-latest.json", &report)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("SBE validation complete");
                println!("  project       : {}", report.path);
                println!("  report        : {}", report.report_path);
                println!("  scan warnings : {}", report.scan.warnings.len());
                print_benchmark(&report.benchmark);
            }
        }
        Commands::Doctor { path, json } => {
            let report = Indexer::doctor(path)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("SBE doctor");
                println!("  path          : {}", report.path);
                println!("  initialized   : {}", yes_no(report.initialized));
                println!("  has index     : {}", yes_no(report.has_index));
                println!(
                    "  storage       : {}",
                    report
                        .storage_version
                        .map(|version| version.to_string())
                        .unwrap_or_else(|| "unknown".into())
                );
                println!("  indexed files : {}", report.indexed_files);
                println!("  stale files   : {}", report.stale_files.len());
                for file in report.stale_files.iter().take(5) {
                    println!("    - {file}");
                }
                if !report.warnings.is_empty() {
                    println!("  warnings      : {}", report.warnings.len());
                    for warning in report.warnings.iter().take(5) {
                        println!("    - {warning}");
                    }
                }
            }
        }
        Commands::ExportJson { path, json } => {
            let store = Store::open_existing(path)?;
            let export_path = store.export_snapshot_json()?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "export_path": export_path.display().to_string()
                    }))?
                );
            } else {
                println!("exported debug JSON to {}", export_path.display());
            }
        }
        Commands::Version { json } => {
            let report = VersionReport {
                name: "Software Brain Engine",
                version: env!("CARGO_PKG_VERSION"),
                storage_version: sbe_common::STORAGE_VERSION,
                target: std::env::consts::OS,
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{} {}", report.name, report.version);
                println!("storage version {}", report.storage_version);
                println!("target {}", report.target);
            }
        }
    }

    Ok(())
}

fn print_context_pack(pack: &ContextPack) {
    let symbol_name = |id| {
        pack.symbols
            .iter()
            .find(|symbol| symbol.id == id)
            .map(|symbol| symbol.name.as_str())
            .unwrap_or("<unknown>")
    };

    println!("Root Symbol: {}", symbol_name(pack.root_symbol));
    println!();
    println!("Dependencies:");
    let mut dependency_names: Vec<&str> = pack
        .dependencies
        .iter()
        .filter_map(|path| path.nodes.get(1).copied())
        .map(symbol_name)
        .collect();
    dependency_names.sort_unstable();
    dependency_names.dedup();
    if dependency_names.is_empty() {
        println!("* none");
    } else {
        for name in dependency_names {
            println!("* {name}");
        }
    }
    println!();
    println!("Callers:");
    if pack.callers.is_empty() {
        println!("* none");
    } else {
        for caller in &pack.callers {
            println!("* {}", symbol_name(*caller));
        }
    }
    println!();
    println!("Context Reduction:");
    println!("{:.1}%", pack.metrics.context_reduction_percent);
}

fn print_simulation_report(engine: &QueryEngine, report: &SimulationReport) {
    let name = |id| engine.symbol_name(id).unwrap_or("<unknown>");
    println!(
        "Simulation: {:?} {}",
        report.operation,
        name(report.target_symbol)
    );
    println!(
        "Risk: {} ({:.1}/100)",
        risk_label(report.risk_level),
        report.risk_score
    );
    println!();
    println!("Affected Symbols: {}", report.affected_symbols.len());
    println!("Affected Files: {}", report.affected_files.len());
    println!("Depth: {}", report.traversal_depth);
    println!();
    println!("Affected Flows:");
    if report.affected_flows.is_empty() {
        println!("* none detected");
    } else {
        for id in &report.affected_flows {
            println!("* {}", name(*id));
        }
    }
    println!();
    println!("Recommended Tests:");
    if report.recommended_tests.is_empty() {
        println!("* none detected");
    } else {
        for id in &report.recommended_tests {
            println!("* {}", name(*id));
        }
    }
    println!();
    println!(
        "Context: {} symbols, {} files, ~{} tokens",
        report.context_pack.metrics.symbols_selected,
        report.context_pack.metrics.files_selected,
        report.context_pack.metrics.estimated_tokens
    );
}

fn risk_label(level: RiskLevel) -> &'static str {
    match level {
        RiskLevel::Low => "LOW",
        RiskLevel::Medium => "MEDIUM",
        RiskLevel::High => "HIGH",
        RiskLevel::Critical => "CRITICAL",
    }
}

fn print_impact_summary(reports: &[ImpactReport]) {
    let affected_symbols: HashSet<u64> = reports
        .iter()
        .flat_map(|report| report.affected_symbols.iter().copied())
        .collect();
    let affected_files: HashSet<u64> = reports
        .iter()
        .flat_map(|report| report.affected.iter().map(|symbol| symbol.file_id))
        .collect();
    let depth = reports
        .iter()
        .map(|report| report.depth)
        .max()
        .unwrap_or_default();

    println!("Affected Symbols: {}", affected_symbols.len());
    println!("Affected Files: {}", affected_files.len());
    println!("Depth: {}", depth);
}

fn print_benchmark(report: &BenchmarkReport) {
    println!("SBE benchmark");
    println!("  query         : {}", report.query);
    println!("  query time    : {} ms", report.query_time_ms);
    println!(
        "  indexed       : {} files, {} symbols",
        report.indexed_files, report.indexed_symbols
    );
    println!(
        "  impacted      : {} files, {} symbols",
        report.impacted_files, report.impacted_symbols
    );
    println!(
        "  tokens        : full ~{}, sbe ~{}, saved ~{} ({}%)",
        report.token_estimate.without_sbe_tokens,
        report.token_estimate.with_sbe_tokens,
        report.token_estimate.saved_tokens,
        report.token_estimate.reduction_percentage
    );
    println!("  layers        :");
    for layer in &report.impacted_layers {
        println!(
            "    {:?}: {} files, {} symbols",
            layer.layer, layer.files, layer.symbols
        );
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn query_engine(path: PathBuf) -> anyhow::Result<QueryEngine> {
    let store = Store::open_existing(path)?;
    if !store.has_index() {
        anyhow::bail!(
            "no SBE index found at {}. Run `sbe scan <path>` or `sbe validate <path>` first.",
            store.index_path().display()
        );
    }
    Ok(QueryEngine::from_snapshot(store.read_snapshot()?))
}
