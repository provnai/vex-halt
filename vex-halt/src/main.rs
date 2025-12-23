//! VEX-HALT: Hallucination Assessment via Layered Testing
//!
//! A comprehensive benchmark for evaluating AI verification systems.
//! 
//! ## Overview
//!
//! VEX-HALT measures calibration rather than accuracy:
//! > "VEX doesn't make LLMs more accurate. VEX makes LLMs know when they're wrong."
//!
//! ## Test Categories
//!
//! - **CCT** (30%): Confidence Calibration Test
//! - **API** (20%): Adversarial Prompt Injection
//! - **FCT** (20%): Factual Consistency Test
//! - **HHT** (20%): Hallucination Honeypot Test
//! - **RT** (10%): Reproducibility Test

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::path::PathBuf;

mod config;
mod dataset;
mod evaluator;
mod llm_judge;
mod merkle;
mod provider;
mod report;
mod runner;
mod scoring;
mod tools;
mod types;
mod vex_integration;

use config::BenchmarkConfig;
use runner::BenchmarkRunner;

#[derive(Parser, Debug)]
#[command(name = "halt_benchmark")]
#[command(author = "VEX Project")]
#[command(version = "1.0.0")]
#[command(about = "VEX-HALT: Hallucination Assessment via Layered Testing")]
struct Args {
    /// Benchmark mode: baseline, vex, or compare
    #[arg(short, long, default_value = "compare")]
    mode: String,

    /// LLM provider: mock, mistral, deepseek, openai, claude, gemini, local
    #[arg(short, long, default_value = "mock")]
    provider: String,

    /// Path to dataset directory
    #[arg(short, long, default_value = "datasets/vex_halt")]
    dataset: PathBuf,

    /// Output format: console, json, markdown, html
    #[arg(short, long, default_value = "console")]
    output: String,

    /// Output file path (optional)
    #[arg(short = 'f', long)]
    output_file: Option<PathBuf>,

    /// Number of runs for statistical significance
    #[arg(short, long, default_value = "1")]
    runs: usize,

    /// Categories to run (comma-separated, e.g., "CCT,API,HHT")
    #[arg(short, long)]
    categories: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable VEX adversarial verification
    #[arg(long)]
    enable_vex: bool,

    #[arg(long, default_value = "3")]
    debate_rounds: usize,

    /// Run in lite mode (5 items per category) for debugging
    #[arg(long)]
    lite: bool,

    /// Validate dataset and configuration without running API calls
    #[arg(long)]
    dry_run: bool,
}

fn print_banner() {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║                                                                  ║".cyan());
    println!("{}", "║   ██╗   ██╗███████╗██╗  ██╗      ██╗  ██╗ █████╗ ██╗  ████████╗  ║".cyan());
    println!("{}", "║   ██║   ██║██╔════╝╚██╗██╔╝      ██║  ██║██╔══██╗██║  ╚══██╔══╝  ║".cyan());
    println!("{}", "║   ██║   ██║█████╗   ╚███╔╝ █████╗███████║███████║██║     ██║     ║".cyan());
    println!("{}", "║   ╚██╗ ██╔╝██╔══╝   ██╔██╗ ╚════╝██╔══██║██╔══██║██║     ██║     ║".cyan());
    println!("{}", "║    ╚████╔╝ ███████╗██╔╝ ██╗      ██║  ██║██║  ██║███████╗██║     ║".cyan());
    println!("{}", "║     ╚═══╝  ╚══════╝╚═╝  ╚═╝      ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝     ║".cyan());
    println!("{}", "║                                                                  ║".cyan());
    println!("{}", "║        Hallucination Assessment via Layered Testing             ║".cyan());
    println!("{}", "║                                                                  ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════════╝".cyan());
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = Args::parse();
    
    print_banner();

    // Parse categories
    let categories: Option<Vec<String>> = args.categories.map(|c| {
        c.split(',')
            .map(|s| s.trim().to_uppercase())
            .collect()
    });

    // Build configuration
    let config = BenchmarkConfig {
        mode: args.mode.parse()?,
        provider: args.provider.parse()?,
        dataset_path: args.dataset,
        output_format: args.output.parse()?,
        output_file: args.output_file,
        num_runs: args.runs,
        categories,
        verbose: args.verbose,
        enable_vex: args.enable_vex,
        debate_rounds: args.debate_rounds,
        lite_mode: args.lite,
        dry_run: args.dry_run,
    };

    println!("{} Configuration:", "▶".green());
    println!("  {} Mode: {}", "•".blue(), config.mode);
    println!("  {} Provider: {}", "•".blue(), config.provider);
    println!("  {} Dataset: {:?}", "•".blue(), config.dataset_path);
    println!("  {} Runs: {}", "•".blue(), config.num_runs);
    if config.lite_mode {
        println!("  {} Lite Mode: Enabled (5 items/category)", "•".blue());
    }
    if config.dry_run {
        println!("  {} Dry Run: Enabled", "•".blue());
    }
    if config.enable_vex {
        println!("  {} VEX Enabled: {} debate rounds", "•".blue(), config.debate_rounds);
    }
    println!();

    // Create and run benchmark
    let runner = BenchmarkRunner::new(config.clone()).await?;
    let results = runner.run().await?;

    // Generate report based on output format
    match config.output_format {
        types::OutputFormat::Console => {
            report::generate(&results)?;
        }
        types::OutputFormat::Json => {
            let json = report::generate_json(&results)?;
            if let Some(ref path) = config.output_file {
                std::fs::write(path, &json)?;
                println!("{} JSON report saved to: {:?}", "✓".green(), path);
            } else {
                println!("{}", json);
            }
        }
        types::OutputFormat::Markdown => {
            let md = report::generate_markdown(&results)?;
            if let Some(ref path) = config.output_file {
                std::fs::write(path, &md)?;
                println!("{} Markdown report saved to: {:?}", "✓".green(), path);
            } else {
                println!("{}", md);
            }
        }
        types::OutputFormat::Html => {
            let html = report::generate_html(&results)?;
            if let Some(ref path) = config.output_file {
                std::fs::write(path, &html)?;
                println!("{} HTML report saved to: {:?}", "✓".green(), path);
            } else {
                println!("{}", html);
            }
        }
    }

    println!();
    println!("{}", "✅ Benchmark complete!".green().bold());
    println!();

    Ok(())
}
