//! Report generation for VEX-HALT benchmark results

use anyhow::Result;
use colored::*;

use crate::types::{BenchmarkResults, TestCategory};

/// Generate and display the benchmark report
pub fn generate(results: &BenchmarkResults) -> Result<()> {
    match results.mode.as_str() {
        "compare" => generate_comparison_report(results),
        _ => generate_single_report(results),
    }
}

/// Generate a comparison report (baseline vs VEX)
fn generate_comparison_report(results: &BenchmarkResults) -> Result<()> {
    let baseline = results.baseline_score.unwrap_or(0.0);
    let vex = results.vex_score.unwrap_or(0.0);
    let improvement = results.improvement.unwrap_or(0.0);

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║                   VEX-HALT BENCHMARK RESULTS                     ║".cyan().bold());
    println!("{}", "╠══════════════════════════════════════════════════════════════════╣".cyan());
    println!("{}  Provider: {:<20}  Date: {}   {}",
        "║".cyan(),
        results.provider.bright_white(),
        results.timestamp.format("%Y-%m-%d %H:%M").to_string().dimmed(),
        "║".cyan()
    );
    println!("{}", "╠══════════════════════════════════════════════════════════════════╣".cyan());
    
    // Header row
    println!("{}  {:<24} {:>10} {:>10} {:>8} {:>6}  {}",
        "║".cyan(),
        "CATEGORY".bright_white().bold(),
        "BASELINE".yellow(),
        "VEX".green(),
        "Δ".bright_cyan(),
        "GRADE".white(),
        "║".cyan()
    );
    println!("{}", "╟──────────────────────────────────────────────────────────────────╢".cyan());

    // Category rows
    for cat in TestCategory::all() {
        if let Some(cat_result) = results.categories.get(&cat) {
            let _cat_grade = BenchmarkResults::score_to_grade(cat_result.score);
            
            // Get per-category baseline score if available
            let cat_baseline = results.baseline_categories
                .as_ref()
                .and_then(|bc| bc.get(&cat))
                .map(|br| br.score)
                .unwrap_or(0.0);
            
            let cat_vex = cat_result.score;
            let delta = cat_vex - cat_baseline;
            let grade = score_to_letter_grade(cat_vex);
            
            let delta_str = if cat_baseline > 0.0 {
                if delta >= 0.0 {
                    format!("+{:.1}", delta).green()
                } else {
                    format!("{:.1}", delta).red()
                }
            } else {
                "N/A".dimmed()
            };

            println!("{}  {:<24} {:>10.1} {:>10.1} {:>8} {:>6}  {}",
                "║".cyan(),
                cat.name(),
                cat_baseline,
                cat_vex,
                delta_str,
                grade_color(&grade),
                "║".cyan()
            );
        }
    }

    println!("{}", "╟──────────────────────────────────────────────────────────────────╢".cyan());
    
    // Final score row
    let final_delta = if improvement >= 0.0 {
        format!("+{:.1}", improvement).green().bold()
    } else {
        format!("{:.1}", improvement).red().bold()
    };

    println!("{}  {:<24} {:>10.1} {:>10.1} {:>8} {:>6}  {}",
        "║".cyan(),
        "FINAL SCORE".bright_white().bold(),
        baseline,
        vex,
        final_delta,
        grade_color(&results.grade).bold(),
        "║".cyan()
    );

    println!("{}", "╠══════════════════════════════════════════════════════════════════╣".cyan());
    
    // Performance section
    println!("{}  {}                                                  {}",
        "║".cyan(),
        "PERFORMANCE".bright_white().bold(),
        "║".cyan()
    );
    println!("{}  Throughput: {} qps  │  Latency p50: {} ms  │  Merkle: {} ms   {}",
        "║".cyan(),
        format!("{:.0}", results.performance.throughput_qps).bright_green(),
        format!("{:.0}", results.performance.latency_p50_ms).bright_yellow(),
        format!("{:.1}", results.performance.merkle_overhead_ms).bright_cyan(),
        "║".cyan()
    );
    
    if let Some(compression) = results.performance.memory_compression_ratio {
        println!("{}  Memory Compression: {}:1  │  Audit Export: {} ms              {}",
            "║".cyan(),
            format!("{:.1}", compression).bright_magenta(),
            format!("{:.0}", results.performance.audit_export_time_ms.unwrap_or(0.0)).dimmed(),
            "║".cyan()
        );
    }

    println!("{}", "╠══════════════════════════════════════════════════════════════════╣".cyan());
    
    // Merkle root
    let short_root = if results.merkle_root.len() > 16 {
        format!("{}...{}", &results.merkle_root[..8], &results.merkle_root[results.merkle_root.len()-8..])
    } else {
        results.merkle_root.clone()
    };
    
    println!("{}  MERKLE ROOT: {}                               {}",
        "║".cyan(),
        short_root.bright_cyan(),
        "║".cyan()
    );
    println!("{}  This benchmark run is cryptographically verified.                {}",
        "║".cyan(),
        "║".cyan()
    );

    // Calculate total tokens
    let total_prompt: usize = results.categories.values()
        .flat_map(|c| c.test_results.iter())
        .filter_map(|r| r.token_usage.as_ref())
        .map(|t| t.prompt_tokens)
        .sum();
    
    let total_completion: usize = results.categories.values()
        .flat_map(|c| c.test_results.iter())
        .filter_map(|r| r.token_usage.as_ref())
        .map(|t| t.completion_tokens)
        .sum();
        
    let total_tokens = total_prompt + total_completion;
    
    // Estimate cost (Using roughly Mistral Large pricing: $2/M in, $6/M out)
    let cost = (total_prompt as f64 / 1_000_000.0 * 2.0) + (total_completion as f64 / 1_000_000.0 * 6.0);

    // Calculate Flip Rate
    let mut total_flips = 0;
    let mut total_regressions = 0;
    let mut total_baseline_failures = 0;
    
    // Build baseline map for O(1) lookup
    let mut baseline_map: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    if let Some(ref baseline_cats) = results.baseline_categories {
         for cat_res in baseline_cats.values() {
             for test in &cat_res.test_results {
                 baseline_map.insert(test.test_id.clone(), test.passed);
                 if !test.passed {
                     total_baseline_failures += 1;
                 }
             }
         }
    }
    
    if !baseline_map.is_empty() {
        for cat_res in results.categories.values() {
             for test in &cat_res.test_results {
                 if let Some(&baseline_passed) = baseline_map.get(&test.test_id) {
                     if !baseline_passed && test.passed {
                         total_flips += 1;
                     } else if baseline_passed && !test.passed {
                         total_regressions += 1;
                     }
                 }
             }
        }
        
        let flip_rate = if total_baseline_failures > 0 {
             (total_flips as f64 / total_baseline_failures as f64) * 100.0
        } else {
             0.0
        };
        
        println!("{}", "╠══════════════════════════════════════════════════════════════════╣".cyan());
        println!("{}  VEX IMPACT: {} Flips (Incorrect→Correct) | {} Regressions   {}", 
            "║".cyan(),
            format!("+{}", total_flips).bright_green(),
            format!("-{}", total_regressions).bright_red(),
            "║".cyan()
        );
        println!("{}  Flip Rate: {:.1}% of baseline failures corrected             {}",
            "║".cyan(),
            flip_rate,
            "║".cyan()
        );
    }

    // Calculate Flip Rate
    let mut total_flips = 0;
    let mut total_regressions = 0;
    let mut total_baseline_failures = 0;
    
    // Build baseline map for O(1) lookup
    let mut baseline_map: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    if let Some(ref baseline_cats) = results.baseline_categories {
         for cat_res in baseline_cats.values() {
             for test in &cat_res.test_results {
                 baseline_map.insert(test.test_id.clone(), test.passed);
                 if !test.passed {
                     total_baseline_failures += 1;
                 }
             }
         }
    }
    
    if !baseline_map.is_empty() {
        for cat_res in results.categories.values() {
             for test in &cat_res.test_results {
                 if let Some(&baseline_passed) = baseline_map.get(&test.test_id) {
                     if !baseline_passed && test.passed {
                         total_flips += 1;
                     } else if baseline_passed && !test.passed {
                         total_regressions += 1;
                     }
                 }
             }
        }
        
        let flip_rate = if total_baseline_failures > 0 {
             (total_flips as f64 / total_baseline_failures as f64) * 100.0
        } else {
             0.0
        };
        
        println!("{}", "╠══════════════════════════════════════════════════════════════════╣".cyan());
        println!("{}  VEX IMPACT: {} Flips (Incorrect→Correct) | {} Regressions   {}", 
            "║".cyan(),
            format!("+{}", total_flips).bright_green(),
            format!("-{}", total_regressions).bright_red(),
            "║".cyan()
        );
        println!("{}  Flip Rate: {:.1}% of baseline failures corrected             {}",
            "║".cyan(),
            flip_rate,
            "║".cyan()
        );
    }

    println!("{}", "╠══════════════════════════════════════════════════════════════════╣".cyan());
    println!("{}  ESTIMATED COST: ${:.4}                                    {}",
        "║".cyan(),
        cost.to_string().bright_yellow(),
        "║".cyan()
    );
    println!("{}  Tokens: {} ({} in / {} out)                {}",
        "║".cyan(),
        total_tokens.to_string().bright_white(),
        total_prompt.to_string().dimmed(),
        total_completion.to_string().dimmed(),
        "║".cyan()
    );

    // Calculate Average Semantic Entropy
    let total_entropy: f64 = results.categories.values()
        .flat_map(|c| c.test_results.iter())
        .filter_map(|r| r.metadata.get("semantic_entropy").and_then(|v| v.as_f64()))
        .sum();
    let entropy_count = results.categories.values()
        .flat_map(|c| c.test_results.iter())
        .filter(|r| r.metadata.contains_key("semantic_entropy"))
        .count();
        
    if entropy_count > 0 {
        let avg_entropy = total_entropy / entropy_count as f64;
        println!("{}  Avg Semantic Entropy: {:.4} (Uncertainty)                  {}",
            "║".cyan(),
            avg_entropy.to_string().bright_magenta(),
            "║".cyan()
        );
    }

    println!("{}", "╚══════════════════════════════════════════════════════════════════╝".cyan().bold());

    // Interpretation
    println!();
    println!("{} {}", "INTERPRETATION:".bright_white().bold(), BenchmarkResults::grade_interpretation(&results.grade));
    println!();

    // Key insight
    if improvement > 0.0 {
        println!(
            "{} Raw {} scores {:.1} on VEX-HALT; {} + {} scores {:.1}. That's a {} improvement!",
            "►".green(),
            results.provider.bright_yellow(),
            baseline,
            results.provider.bright_yellow(),
            "VEX".bright_green().bold(),
            vex,
            format!("+{:.1}%", improvement).bright_green().bold()
        );
    }

    Ok(())
}

/// Generate a single-mode report
fn generate_single_report(results: &BenchmarkResults) -> Result<()> {
    println!();
    println!("{}", "═".repeat(60).cyan());
    println!("{} VEX-HALT BENCHMARK - {} MODE",
        "▶".cyan(),
        results.mode.to_uppercase().bright_white().bold()
    );
    println!("{}", "═".repeat(60).cyan());
    println!();

    println!("{} Results by Category:", "▶".yellow());
    println!();

    for cat in TestCategory::all() {
        if let Some(cat_result) = results.categories.get(&cat) {
            let grade = score_to_letter_grade(cat_result.score);
            let bar = create_score_bar(cat_result.score);
            
            println!(
                "  {} {:<25} │ {} │ {:>5.1} │ {}",
                grade_icon(&grade),
                cat.name(),
                bar,
                cat_result.score,
                grade_color(&grade)
            );
        }
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!(
        "  {} {:<25} │       {:>5.1} │ {}",
        "★".bright_yellow(),
        "FINAL SCORE".bright_white().bold(),
        results.final_score,
        grade_color(&results.grade).bold()
    );
    println!("{}", "═".repeat(60).cyan());

    // Calculate total tokens
    let total_prompt: usize = results.categories.values()
        .flat_map(|c| c.test_results.iter())
        .filter_map(|r| r.token_usage.as_ref())
        .map(|t| t.prompt_tokens)
        .sum();
    
    let total_completion: usize = results.categories.values()
        .flat_map(|c| c.test_results.iter())
        .filter_map(|r| r.token_usage.as_ref())
        .map(|t| t.completion_tokens)
        .sum();
    
    // Estimate cost (Using roughly Mistral Large pricing: $2/M in, $6/M out)
    let cost = (total_prompt as f64 / 1_000_000.0 * 2.0) + (total_completion as f64 / 1_000_000.0 * 6.0);

    println!("  {} Est. Cost: ${:.4} ({} tokens)",
        "$" .bright_yellow(),
        cost,
        (total_prompt + total_completion).to_string().dimmed()
    );
    println!();

    // Display Merkle root for cryptographic verification
    let short_root = if results.merkle_root.len() > 16 {
        format!("{}...{}", &results.merkle_root[..8], &results.merkle_root[results.merkle_root.len()-8..])
    } else {
        results.merkle_root.clone()
    };

    println!("  {} Merkle Root: {} (cryptographically verified)",
        "🔐".bright_cyan(),
        short_root.bright_cyan()
    );
    println!();

    Ok(())
}

/// Create a visual score bar
fn create_score_bar(score: f64) -> String {
    let filled = (score / 5.0).round() as usize;
    let empty = 20 - filled;
    
    let bar: String = "█".repeat(filled);
    let empty_bar: String = "░".repeat(empty);
    
    if score >= 80.0 {
        format!("{}{}", bar.green(), empty_bar.dimmed())
    } else if score >= 60.0 {
        format!("{}{}", bar.yellow(), empty_bar.dimmed())
    } else {
        format!("{}{}", bar.red(), empty_bar.dimmed())
    }
}

/// Convert score to letter grade
fn score_to_letter_grade(score: f64) -> String {
    match score {
        s if s >= 90.0 => "A+".to_string(),
        s if s >= 80.0 => "A".to_string(),
        s if s >= 70.0 => "B".to_string(),
        s if s >= 60.0 => "C".to_string(),
        s if s >= 50.0 => "D".to_string(),
        _ => "F".to_string(),
    }
}

/// Color a grade string
fn grade_color(grade: &str) -> colored::ColoredString {
    match grade {
        "A+" => grade.bright_green().bold(),
        "A" => grade.green(),
        "B" => grade.yellow(),
        "C" => grade.bright_yellow(),
        "D" => grade.red(),
        _ => grade.bright_red(),
    }
}

/// Get icon for grade
fn grade_icon(grade: &str) -> colored::ColoredString {
    match grade {
        "A+" | "A" => "✓".green(),
        "B" => "●".yellow(),
        "C" => "○".yellow(),
        _ => "✗".red(),
    }
}

/// Generate JSON report
pub fn generate_json(results: &BenchmarkResults) -> Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}

/// Generate Markdown report
pub fn generate_markdown(results: &BenchmarkResults) -> Result<String> {
    let mut md = String::new();
    
    md.push_str("# VEX-HALT Benchmark Results\n\n");
    md.push_str(&format!("**Date:** {}  \n", results.timestamp.format("%Y-%m-%d %H:%M UTC")));
    md.push_str(&format!("**Provider:** {}  \n", results.provider));
    md.push_str(&format!("**Mode:** {}  \n\n", results.mode));

    md.push_str("## Results by Category\n\n");
    md.push_str("| Category | Score | Grade | Passed/Total |\n");
    md.push_str("|----------|-------|-------|-------------|\n");

    for cat in TestCategory::all() {
        if let Some(r) = results.categories.get(&cat) {
            md.push_str(&format!(
                "| {} | {:.1} | {} | {}/{} |\n",
                cat.name(),
                r.score,
                score_to_letter_grade(r.score),
                r.passed,
                r.total_tests
            ));
        }
    }

    md.push_str(&format!("\n**Final Score:** {:.1} ({})\n\n", results.final_score, results.grade));

    if let Some(improvement) = results.improvement {
        md.push_str(&format!("**Improvement over baseline:** {:.1}%\n\n", improvement));
    }

    md.push_str("## Performance Metrics\n\n");
    md.push_str(&format!("- **Throughput:** {:.0} qps\n", results.performance.throughput_qps));
    md.push_str(&format!("- **Latency (p50):** {:.0} ms\n", results.performance.latency_p50_ms));
    md.push_str(&format!("- **Merkle Overhead:** {:.1} ms\n", results.performance.merkle_overhead_ms));

    md.push_str(&format!("\n**Merkle Root:** `{}`\n", results.merkle_root));

    Ok(md)
}

/// Generate beautiful HTML report
pub fn generate_html(results: &BenchmarkResults) -> Result<String> {
    let mut categories_html = String::new();
    
    for cat in TestCategory::all() {
        if let Some(r) = results.categories.get(&cat) {
            let bar_width = (r.score as i32).clamp(0, 100);
            let grade_color = match score_to_letter_grade(r.score).as_str() {
                "A+" | "A" => "#3fb950",
                "B" => "#58a6ff",
                "C" => "#d29922",
                _ => "#f85149",
            };
            
            // Get per-category baseline score if available
            let cat_baseline = results.baseline_categories
                .as_ref()
                .and_then(|bc| bc.get(&cat))
                .map(|br| br.score)
                .unwrap_or(0.0);
            
            let delta = r.score - cat_baseline;
            let delta_str = if cat_baseline > 0.0 {
                if delta >= 0.0 {
                    format!("+{:.1}", delta)
                } else {
                    format!("{:.1}", delta)
                }
            } else {
                "N/A".to_string()
            };
            
            categories_html.push_str(&format!(r#"
                <div class="category-row">
                    <div class="category-name">{}</div>
                    <div class="score-bar">
                        <div class="score-fill" style="width: {}%; background: {}"></div>
                    </div>
                    <div class="score-value" style="color: {}">{:.1} ({:.1} → {:.1}, {})</div>
                    <div class="grade" style="color: {}">{}</div>
                </div>
            "#, cat.name(), bar_width, grade_color, grade_color, r.score, cat_baseline, r.score, delta_str, grade_color, 
                score_to_letter_grade(r.score)));
        }
    }
    
    let final_grade_color = match results.grade.as_str() {
        "A+" | "A" => "#3fb950",
        "B" => "#58a6ff",
        "C" => "#d29922",
        _ => "#f85149",
    };
    
    let improvement_html = if let Some(imp) = results.improvement {
        format!(r#"<div class="improvement">📈 +{:.1}% improvement with VEX</div>"#, imp)
    } else {
        String::new()
    };

    let chart_svg = generate_cost_accuracy_chart(results);
    let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>VEX-HALT Report</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: #c9d1d9;
            background: #0d1117;
            padding: 2rem;
            max-width: 1000px;
            margin: 0 auto;
        }}
        .container {{ background: #161b22; border-radius: 16px; padding: 2rem; border: 1px solid #30363d; }}
        .header {{ text-align: center; margin-bottom: 3rem; border-bottom: 1px solid #21262d; padding-bottom: 2rem; }}
        .logo {{ font-size: 3rem; margin-bottom: 1rem; }}
        .title {{ font-size: 2.5rem; font-weight: 800; background: linear-gradient(90deg, #58a6ff, #88d1f1); -webkit-background-clip: text; -webkit-text-fill-color: transparent; margin: 0; }}
        .subtitle {{ color: #8b949e; font-size: 1.1rem; margin-top: 0.5rem; }}
        
        .final-score {{ text-align: center; margin-bottom: 3rem; position: relative; }}
        .score-big {{ font-size: 6rem; font-weight: 800; line-height: 1; color: {}; }}
        .grade-big {{ font-size: 3rem; font-weight: 700; color: {}; position: absolute; top: 0; right: 20%; opacity: 0.2; }}
        
        .chart-container {{
            background: rgba(22, 27, 34, 0.8);
            border-radius: 16px;
            padding: 1.5rem;
            margin-bottom: 2rem;
            border: 1px solid #30363d;
            text-align: center;
        }}
        .chart-title {{ color: #58a6ff; margin-bottom: 1rem; font-size: 1.25rem; font-weight: 600; }}
        
        .categories {{
            background: rgba(22, 27, 34, 0.8);
            border-radius: 16px;
            padding: 1.5rem;
            margin-bottom: 2rem;
            border: 1px solid #30363d;
        }}
        .categories h2 {{ color: #58a6ff; margin-bottom: 1.5rem; font-size: 1.25rem; }}
        .category-row {{
            display: grid;
            grid-template-columns: 200px 1fr 60px 40px;
            gap: 1rem;
            align-items: center;
            padding: 0.75rem 0;
            border-bottom: 1px solid #21262d;
        }}
        .category-row:last-child {{ border-bottom: none; }}
        .category-name {{ color: #c9d1d9; font-weight: 500; }}
        .score-bar {{ height: 8px; background: #21262d; border-radius: 4px; overflow: hidden; }}
        .score-fill {{ height: 100%; border-radius: 4px; transition: width 0.5s ease; }}
        .score-value {{ text-align: right; font-weight: 600; }}
        .grade {{ text-align: center; font-weight: 700; }}
        
        .metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }}
        .metric-card {{
            background: rgba(22, 27, 34, 0.8);
            border-radius: 12px;
            padding: 1.5rem;
            text-align: center;
            border: 1px solid #30363d;
        }}
        .metric-value {{ font-size: 2rem; font-weight: 600; color: #58a6ff; }}
        .metric-label {{ color: #8b949e; margin-top: 0.5rem; font-size: 0.875rem; }}
        
        .footer {{ text-align: center; margin-top: 2rem; color: #8b949e; font-size: 0.875rem; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🔬</div>
            <div class="title">VEX-HALT Benchmark Report</div>
            <div class="subtitle">{} • {} Mode • {}</div>
        </div>
        
        <div class="final-score">
            <div class="score-big">{:.1}</div>
            <div class="grade-big">{}</div>
            {}
        </div>
        
        <div class="chart-container">
            <div class="chart-title">📊 Cost vs Accuracy Analysis</div>
            {}
        </div>
        
        <div class="categories">
            <h2>🏆 Results by Category</h2>
            {}
        </div>
        
        <div class="metrics">
            <div class="metric-card"><div class="metric-value">{:.0}</div><div class="metric-label">Queries/Second</div></div>
            <div class="metric-card"><div class="metric-value">{:.0}ms</div><div class="metric-label">Latency (p50)</div></div>
            <div class="metric-card"><div class="metric-value">{}</div><div class="metric-label">Total Items</div></div>
            <div class="metric-card"><div class="metric-value">{:.1}ms</div><div class="metric-label">Merkle Overhead</div></div>
        </div>
        
        <div class="footer">
             Generated by VEX-HALT v2.0 • {}
        </div>
    </div>
</body>
</html>"#,
        final_grade_color, final_grade_color,
        results.provider, results.mode, results.timestamp.format("%Y-%m-%d %H:%M UTC"),
        results.final_score, results.grade,
        improvement_html,
        chart_svg,
        categories_html,
        results.performance.throughput_qps,
        results.performance.latency_p50_ms,
        results.categories.values().map(|c| c.total_tests).sum::<usize>(),
        results.performance.merkle_overhead_ms,
        results.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    );

    Ok(html)
}

/// Generate SVG scatter plot for Cost vs Accuracy
fn generate_cost_accuracy_chart(results: &BenchmarkResults) -> String {
    let width = 800;
    let height = 400;
    let padding = 60;
    
    // Calculate data points
    let mut points: Vec<(String, f64, f64, String)> = Vec::new(); // (Label, Score, Cost, Color)
    let mut max_cost = 0.0;
    
    for (cat, result) in &results.categories {
        let score = result.score;
        
        // Calculate total cost for category
        let mut total_cost = 0.0;
        for test in &result.test_results {
            if let Some(usage) = &test.token_usage {
                // Approximate cost: $3/M in, $15/M out (Mistral Large / GPT-4o blend)
                let cost_in = (usage.prompt_tokens as f64 / 1_000_000.0) * 3.0;
                let cost_out = (usage.completion_tokens as f64 / 1_000_000.0) * 15.0;
                total_cost += cost_in + cost_out;
            }
        }
        
        if total_cost > max_cost { max_cost = total_cost; }
        
        let color = match score_to_letter_grade(score).as_str() {
            "A+" | "A" => "#3fb950",
            "B" => "#58a6ff",
            "C" => "#d29922",
            _ => "#f85149",
        };
        
        points.push((cat.name().to_string(), score, total_cost, color.to_string()));
    }
    
    // Avoid division by zero
    if max_cost == 0.0 { max_cost = 1.0; }
    
    let mut svg = String::new();
    svg.push_str(&format!(r##"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" style="background:transparent; max-width:100%;">"##, width, height));
    
    // Axes
    svg.push_str(&format!(r##"
        <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#30363d" stroke-width="2"/>
        <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#30363d" stroke-width="2"/>
        <text x="{}" y="{}" fill="#8b949e" text-anchor="middle" font-size="12">Cost ($)</text>
        <text x="{}" y="{}" fill="#8b949e" text-anchor="middle" font-size="12" transform="rotate(-90, {}, {})">Score (%)</text>
    "##, 
        padding, height - padding, width - padding, height - padding, // X-axis
        padding, height - padding, padding, padding, // Y-axis
        width / 2, height - 10,
        20, height / 2, 20, height / 2
    ));
    
    // Grid lines (horizontal)
    for i in 0..=5 {
        let y = height - padding - (i * (height - 2 * padding) / 5);
        svg.push_str(&format!(r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#21262d" stroke-width="1"/>"##, 
            padding, y, width - padding, y));
        svg.push_str(&format!(r##"<text x="{}" y="{}" fill="#6e7681" text-anchor="end" font-size="10" alignment-baseline="middle">{}%</text>"##, 
            padding - 10, y, i * 20));
    }

    // Plot points
    for (label, score, cost, color) in points {
        let x = padding as f64 + (cost / max_cost) * (width - 2 * padding) as f64;
        let y = (height - padding) as f64 - (score / 100.0) * (height - 2 * padding) as f64;
        
        // Point
        svg.push_str(&format!(r##"
            <circle cx="{:.1}" cy="{:.1}" r="6" fill="{}" stroke="#0d1117" stroke-width="2">
                <title>{}: {:.1}% / ${:.5}</title>
            </circle>
            <text x="{:.1}" y="{:.1}" fill="#c9d1d9" font-size="11" font-weight="bold" text-anchor="middle" dy="-10">{}</text>
        "##, x, y, color, label, score, cost, x, y, label));
    }
    
    svg.push_str("</svg>");
    svg
}
