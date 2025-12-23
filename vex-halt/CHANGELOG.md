# Changelog

All notable changes to VEX-HALT are documented in this file.

## [2.0.0] - 2025-12-16

### Added
- **7 New Test Categories**: FRONTIER, VSM, MTC, EAS, MEM, AGT, VEX (total: 12 categories)
- **LLM-as-Judge** (`llm_judge.rs`): 2025 best practices with 5 category-specific rubrics
- **Mock Tool Framework** (`tools.rs`): 7 sandboxed tools for agent testing
- **Real VEX Integration** (`vex_integration.rs`): Multi-round adversarial debate
- **HTML Report Generation**: Beautiful dark-mode reports with animated score bars
- **Integration Tests** (`tests/integration_test.rs`): Dataset validation tests
- **Generic Category Loader**: `load_generic_category()` for easy category additions

### Changed
- **Test Item Count**: Increased from 290 to 443+ items
- **Scoring Formula**: Updated weights to accommodate 12 categories (sum = 1.0)
- **Category Weights**: CCT reduced to 15%, new categories get remaining weight
- **Output Formats**: Added HTML alongside Console, JSON, Markdown

### New Source Files
| File | Lines | Purpose |
|------|-------|---------|
| `llm_judge.rs` | 270 | LLM-as-Judge with CoT prompting |
| `tools.rs` | 390 | 7 mock tools + chain execution |
| `vex_integration.rs` | 300 | VEX debate + Merkle proofing |
| `tests/integration_test.rs` | 90 | Dataset validation |

### New Dataset Files
| Category | Files |
|----------|-------|
| FRONTIER | `adversarial_reasoning.json`, `deceptive_patterns.json`, `meta_questions.json` |
| VSM | `overconfidence.json` |
| MTC | `tool_chains.json` |
| EAS | `uncertainty_classification.json` |
| MEM | `episodic_recall.json`, `temporal_decay.json`, `compression.json` |
| AGT | `agentic_safety.json`, `tool_use.json`, `long_horizon.json` |
| VEX | `showcase.json`, `ab_comparison.json` |

### Real VEX Integration
- **VEX Protocol**: Integrated with vex-core, vex-adversarial, vex-llm (v0.1.4)
- **Multi-round adversarial debate**: Blue/Red agent verification
- **Merkle tree audit trails**: Cryptographic proof of debate integrity
- **Shadow agent analysis**: Automated issue detection in responses

---

## [1.0.0] - 2024-12-15

### Initial Release
- 5 test categories: CCT, API, FCT, HHT, RT
- ~290 test items
- Mock, Mistral, DeepSeek, OpenAI providers
- Console, JSON, Markdown output
- Merkle proof verification
- Baseline vs VEX comparison mode
