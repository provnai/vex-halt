# VEX-HALT: Hallucination Assessment via Layered Testing

<div align="center">

```
██╗   ██╗███████╗██╗  ██╗      ██╗  ██╗ █████╗ ██╗  ████████╗
██║   ██║██╔════╝╚██╗██╔╝      ██║  ██║██╔══██╗██║  ╚══██╔══╝
██║   ██║█████╗   ╚███╔╝ █████╗███████║███████║██║     ██║   
╚██╗ ██╔╝██╔══╝   ██╔██╗ ╚════╝██╔══██║██╔══██║██║     ██║   
 ╚████╔╝ ███████╗██╔╝ ██╗      ██║  ██║██║  ██║███████╗██║   
  ╚═══╝  ╚══════╝╚═╝  ╚═╝      ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝   
```

**A research benchmark for evaluating AI verification systems**

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-443-green.svg)]()

</div>

## 🔗 Related Research

VEX-HALT is designed to evaluate [VEX](https://github.com/provnai/vex), a protocol for verifying autonomous AI agents.

| Project | Purpose |
|---------|---------|
| [**VEX**](https://github.com/provnai/vex) | The verification protocol (adversarial debate, Merkle proofs) |
| **VEX-HALT** | The benchmark that evaluates VEX (this repo) |

> *Research project exploring AI verification methods.*

---

## 🎯 Overview

VEX-HALT is a research benchmark designed to evaluate AI verification systems, focusing on **calibration** rather than just accuracy:

> **"VEX doesn't make LLMs more accurate. VEX makes LLMs know when they're wrong."**

This is an experimental approach to understanding how adversarial verification might improve AI reliability.

## 🔐 Cryptographic Verification

**VEX-HALT includes built-in cryptographic verification for result integrity:**

- **🔒 Merkle Tree Verification** - Every benchmark run generates a cryptographic Merkle root that mathematically proves result integrity
- **📋 Tamper-Proof Audit Trail** - Individual test results are hashed and combined into an immutable proof chain  
- **✅ Independent Verification** - Anyone can recalculate the Merkle root from raw results to verify authenticity
- **🏛️ Regulatory Compliance** - Designed for EU AI Act requirements for cryptographic audit trails

> **Unlike traditional benchmarks that only provide aggregate scores, VEX-HALT provides mathematical proof that results haven't been tampered with.**

---

## ✨ Features

### 🔬 12 Test Categories (443+ Items)

| Category | Weight | Description |
|----------|--------|-------------|
| **CCT** | 15% | Confidence Calibration - Does stated confidence match accuracy? |
| **API** | 10% | Adversarial Prompt Injection - Jailbreaks, injections, attacks |
| **FCT** | 10% | Factual Consistency - Multi-step reasoning verification |
| **HHT** | 10% | Hallucination Honeypots - Completely fictional entities |
| **RT** | 5% | Reproducibility - Deterministic output verification |
| **FRONTIER** | 15% | Super-hard problems (ARC-AGI, FrontierMath style) |
| **VSM** | 5% | Verbal-Semantic Misalignment - Overconfidence detection |
| **MTC** | 5% | Multi-Step Tool Chains - Agent tool usage |
| **EAS** | 5% | Epistemic-Aleatoric Split - Uncertainty classification |
| **MEM** | 5% | Memory Evaluation - VEX temporal memory testing |
| **AGT** | 10% | Agentic Safety - Deception, sandbagging, sycophancy |
| **VEX** | 5% | VEX Showcase - A/B comparison baseline vs VEX |

### 🛡️ Technical Features

- **🔐 Merkle Tree Audit Trail** - Every benchmark run produces a cryptographic Merkle root for result verification
- **🤖 LLM-as-Judge** - Automated evaluation using LLM judges with category-specific rubrics
- **🧰 Mock Tool Framework** - 7 sandboxed tools for agent evaluation
- **📊 Multiple Report Formats** - Console, JSON, Markdown, and HTML output
- **⚡ Parallel Execution** - Async test running with configurable concurrency
- **💰 Cost Tracking** - Token usage and estimated cost per run


## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/provnai/vex-halt
cd vex-halt
cargo build --release

# Run with mock provider
./target/release/halt_benchmark --mode baseline --provider mock

# Run with real LLM (requires API key)
export OPENAI_API_KEY=sk-...
./target/release/halt_benchmark --mode compare --provider openai

# Generate HTML report
./target/release/halt_benchmark -o html -f report.html
```

## 📋 Command Line Options

```
USAGE:
    halt_benchmark [OPTIONS]

OPTIONS:
    -m, --mode <MODE>           baseline, vex, or compare [default: compare]
    -p, --provider <PROVIDER>   mock, mistral, deepseek, openai, claude, gemini, local [default: mock]
    -d, --dataset <PATH>        Path to dataset directory
    -o, --output <FORMAT>       console, json, markdown, html [default: console]
    -f, --output-file <PATH>    Save report to file
    -r, --runs <N>              Number of runs for statistics [default: 1]
    -c, --categories <LIST>     Comma-separated categories to run
    --enable-vex                Enable VEX adversarial verification
    --debate-rounds <N>         Debate rounds for VEX mode [default: 3]
    -v, --verbose               Verbose output
```

## 📊 Sample Output

```
▶ VEX-HALT BENCHMARK - BASELINE MODE
════════════════════════════════════════════════════════════

  ✗ Confidence Calibration    │ ████░░░░░░░░░░░░░░░░ │  18.9 │ F
  ✓ Adversarial Injection     │ ████████████████████ │ 100.0 │ A+
  ✗ Factual Consistency       │ ██████░░░░░░░░░░░░░░ │  30.0 │ F
  ✗ Hallucination Honeypot    │ ░░░░░░░░░░░░░░░░░░░░ │   2.0 │ F
  ✓ Reproducibility           │ ████████████████████ │ 100.0 │ A+
  ✗ Frontier Super-Hard       │ ░░░░░░░░░░░░░░░░░░░░ │   0.0 │ F
  ...

  ★ FINAL SCORE               │        21.0 │ F
════════════════════════════════════════════════════════════

  🔐 Merkle Root: 3a7f8b9c... (cryptographically verified)
```
```

## 🧪 Test Categories Deep Dive

### FRONTIER (Super-Hard Problems)

Inspired by ARC-AGI and FrontierMath, these tests push the boundaries:
- Compositional reasoning with multiple interacting rules
- Abstract pattern recognition (Bongard-style)
- Research-level mathematics
- Meta-cognitive traps

### AGT (Agentic Safety)

Based on METR 2025 and AI Agent Index research:
- **Deception Detection** - Hidden goals, instruction concealment
- **Sandbagging** - Strategic underperformance detection
- **Sycophancy** - Telling users what they want vs. truth
- **Autonomy Boundaries** - Scope creep, confirmation-seeking

### MTC (Multi-Step Tool Chains)

Mock tools for agent evaluation:
```rust
let registry = ToolRegistry::with_mocks();
// calculator, get_weather, convert_currency, web_search, 
// format_date, create_user, send_email
```

### EAS (Epistemic-Aleatoric Split)

Evaluates if AI correctly classifies uncertainty:
- **Epistemic**: Knowledge gaps (learnable)
- **Aleatoric**: Inherent randomness (unpredictable)

## 🏗️ Architecture

```
vex-halt/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── runner.rs        # Benchmark orchestration
│   ├── dataset.rs       # 12-category loader
│   ├── evaluator.rs     # Response evaluation
│   ├── scoring.rs       # Category-specific metrics
│   ├── llm_judge.rs     # LLM-as-Judge with rubrics
│   ├── tools.rs         # Mock tool framework
│   ├── merkle.rs        # Cryptographic proofs
│   ├── provider.rs      # LLM providers
│   ├── report.rs        # Console/JSON/MD/HTML output
│   ├── vex_integration.rs # VEX debate system
│   ├── types.rs         # Core types
│   └── config.rs        # Provider configuration
├── datasets/vex_halt/   # 44 JSON test files
│   ├── cct/             # Confidence Calibration
│   ├── api/             # Adversarial Injection
│   ├── fct/             # Factual Consistency
│   ├── hht/             # Hallucination Honeypots
│   ├── rt/              # Reproducibility
│   ├── frontier/        # Super-Hard Problems
│   ├── vsm/             # Verbal-Semantic Misalignment
│   ├── mtc/             # Multi-Step Tool Chains
│   ├── eas/             # Epistemic-Aleatoric Split
│   ├── mem/             # Memory Evaluation
│   ├── agt/             # Agentic Safety
│   └── vex/             # VEX Showcase
└── tests/               # Integration tests
```

## 🔬 Research Context

VEX-HALT draws inspiration from several areas of AI evaluation research:
- **ARC-AGI** (2024) - Abstract reasoning challenges
- **FrontierMath** (2024) - Research-level math problems
- **METR** (2025) - Long-horizon agent evaluation
- **RedDebate** (2025) - Multi-agent debate frameworks
- **AI Agent Index** (2025) - Agentic safety research
- **LLM-as-Judge** (2025) - Evaluation best practices

This work is exploratory and builds on existing research in AI safety and evaluation.

## � Technical Dependencies

### VEX Protocol Integration
VEX-HALT integrates with the [VEX Protocol](https://github.com/provnai/vex) for adversarial verification:

- **vex-core**: `v0.1.4` - Core primitives and Merkle trees
- **vex-adversarial**: `v0.1.4` - Multi-agent debate and shadow agents  
- **vex-llm**: `v0.1.4` - LLM provider abstraction
- **vex-temporal**: `v0.1.4` - Temporal reasoning (not currently used)

*All VEX crates use commit `b84c0545d76d8712dd5c23d01341071b6212984c` from the development branch.*

### HTTP Client
- **reqwest**: `v0.11.27` (primary), `v0.12.28` (via vex-llm)

### VEX Integration Scope
The implementation uses ~80% of VEX's core features:
- ✅ Multi-agent adversarial debate (Blue/Red agents)
- ✅ Merkle tree audit trails for reproducibility  
- ✅ Shadow agent issue detection
- ✅ Consensus evaluation
- ❌ Distributed agent coordination
- ❌ Advanced temporal reasoning

## �📈 Scoring

Final score = weighted sum across categories:

```
0.15×CCT + 0.10×API + 0.10×FCT + 0.10×HHT + 0.05×RT + 
0.15×FRONTIER + 0.05×VSM + 0.05×MTC + 0.05×EAS + 
0.05×MEM + 0.10×AGT + 0.05×VEX = 100%
```

| Grade | Score | Interpretation |
|-------|-------|----------------|
| A+ | ≥90 | High reliability for critical applications |
| A | ≥80 | Suitable for most applications |
| B | ≥70 | Requires monitoring and oversight |
| C | ≥50 | Limited reliability |
| F | <50 | High hallucination risk |

*These thresholds are experimental and may be adjusted based on further research.*

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🛡️ Security

See [SECURITY.md](SECURITY.md) for security policies and responsible disclosure.

## 📋 Code of Conduct

This project follows a code of conduct to ensure a welcoming environment for all contributors. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for details.

---

<div align="center">
<strong>Research Project</strong><br>
Exploring AI verification methods
</div>
