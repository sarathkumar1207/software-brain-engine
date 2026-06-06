use clap::{Parser, Subcommand};
use sbe_indexer::{IndexReport, Indexer};
use sbe_query::{BenchmarkReport, QueryEngine};
use sbe_storage::Store;
use serde::Serialize;
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
    /// Scan and index a TypeScript/TSX repository.
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
                for report in reports {
                    println!("origin: {}", report.origin);
                    for affected in report.affected {
                        println!("affected: {} {:?}", affected.name, affected.kind);
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
