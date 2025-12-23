//! Dataset loading and parsing for VEX-HALT benchmark

#![allow(dead_code)]  // Internal JSON-parsing structs have fields used only by serde

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::types::{TestCategory, TestExpectation, TestItem};

/// Dataset loader
pub struct DatasetLoader {
    base_path: std::path::PathBuf,
}

impl DatasetLoader {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Load all test items from the dataset
    pub async fn load_all(&self) -> Result<Vec<TestItem>> {
        let mut items = Vec::new();

        // Load original categories
        items.extend(self.load_cct().await?);
        items.extend(self.load_api().await?);
        items.extend(self.load_fct().await?);
        items.extend(self.load_hht().await?);
        items.extend(self.load_rt().await?);
        
        // Load new categories
        items.extend(self.load_frontier().await?);
        items.extend(self.load_vsm().await?);
        items.extend(self.load_mtc().await?);
        items.extend(self.load_eas().await?);
        items.extend(self.load_mem().await?);
        items.extend(self.load_agt().await?);
        items.extend(self.load_vex().await?);

        tracing::info!("Loaded {} test items", items.len());
        Ok(items)
    }

    /// Load only specific categories
    pub async fn load_categories(&self, categories: &[TestCategory]) -> Result<Vec<TestItem>> {
        let mut items = Vec::new();

        for cat in categories {
            match cat {
                TestCategory::CCT => items.extend(self.load_cct().await?),
                TestCategory::API => items.extend(self.load_api().await?),
                TestCategory::FCT => items.extend(self.load_fct().await?),
                TestCategory::HHT => items.extend(self.load_hht().await?),
                TestCategory::RT => items.extend(self.load_rt().await?),
                TestCategory::FRONTIER => items.extend(self.load_frontier().await?),
                TestCategory::VSM => items.extend(self.load_vsm().await?),
                TestCategory::MTC => items.extend(self.load_mtc().await?),
                TestCategory::EAS => items.extend(self.load_eas().await?),
                TestCategory::MEM => items.extend(self.load_mem().await?),
                TestCategory::AGT => items.extend(self.load_agt().await?),
                TestCategory::VEX => items.extend(self.load_vex().await?),
            }
        }

        Ok(items)
    }

    /// Load CCT (Confidence Calibration Test) items
    async fn load_cct(&self) -> Result<Vec<TestItem>> {
        let cct_path = self.base_path.join("cct");
        let mut items = Vec::new();

        for file in &["easy", "medium", "hard", "ambiguous", "unanswerable"] {
            let path = cct_path.join(format!("{}.json", file));
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await
                    .context(format!("Failed to read {:?}", path))?;
                let parsed: CctFile = serde_json::from_str(&content)
                    .context(format!("Failed to parse {:?}", path))?;
                
                for q in parsed.questions {
                    items.push(cct_to_test_item(q, file)?);
                }
            }
        }

        tracing::debug!("Loaded {} CCT items", items.len());
        Ok(items)
    }

    /// Load API (Adversarial Prompt Injection) items
    async fn load_api(&self) -> Result<Vec<TestItem>> {
        let api_path = self.base_path.join("api");
        let mut items = Vec::new();

        for file in &["direct_injection", "indirect_injection", "tool_abuse", "context_poisoning", "jailbreaks", "clean_control"] {
            let path = api_path.join(format!("{}.json", file));
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await?;
                let parsed: ApiFile = serde_json::from_str(&content)?;
                
                for p in parsed.prompts {
                    items.push(api_to_test_item(p, file)?);
                }
            }
        }

        tracing::debug!("Loaded {} API items", items.len());
        Ok(items)
    }

    /// Load FCT (Factual Consistency Test) items
    async fn load_fct(&self) -> Result<Vec<TestItem>> {
        let fct_path = self.base_path.join("fct");
        let mut items = Vec::new();

        for file in &["math_problems", "logic_puzzles", "temporal_reasoning", "scientific_reasoning", "flawed_premises"] {
            let path = fct_path.join(format!("{}.json", file));
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await?;
                let parsed: FctFile = serde_json::from_str(&content)?;
                
                for p in parsed.problems {
                    items.push(fct_to_test_item(p, file)?);
                }
            }
        }

        tracing::debug!("Loaded {} FCT items", items.len());
        Ok(items)
    }

    /// Load HHT (Hallucination Honeypot Test) items
    async fn load_hht(&self) -> Result<Vec<TestItem>> {
        let hht_path = self.base_path.join("hht");
        let mut items = Vec::new();

        for file in &["fake_entities", "fake_events", "fake_statistics", "fake_products", "plausible_falsehoods", "logical_impossibilities"] {
            let path = hht_path.join(format!("{}.json", file));
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await?;
                let parsed: HhtFile = serde_json::from_str(&content)?;
                
                for h in parsed.honeypots {
                    items.push(hht_to_test_item(h, file)?);
                }
            }
        }

        tracing::debug!("Loaded {} HHT items", items.len());
        Ok(items)
    }

    /// Load RT (Reproducibility Test) items
    async fn load_rt(&self) -> Result<Vec<TestItem>> {
        let rt_path = self.base_path.join("rt");
        let mut items = Vec::new();

        for file in &["deterministic", "replay", "tampering"] {
            let path = rt_path.join(format!("{}.json", file));
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await?;
                let parsed: RtFile = serde_json::from_str(&content)?;
                
                for t in parsed.tests {
                    items.push(rt_to_test_item(t, file)?);
                }
            }
        }

        tracing::debug!("Loaded {} RT items", items.len());
        Ok(items)
    }
}

// ============ Raw file structures ============

#[derive(Debug, Deserialize)]
struct CctFile {
    category: String,
    subcategory: String,
    questions: Vec<CctQuestion>,
}

#[derive(Debug, Deserialize)]
struct CctQuestion {
    id: String,
    question: String,
    #[serde(default)]
    expected_answer: Option<String>,
    #[serde(default)]
    expected_behavior: Option<String>,
    #[serde(default)]
    answer_type: Option<String>,
    #[serde(default)]
    difficulty: Option<u32>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    rationale: Option<String>,
    #[serde(default)]
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiFile {
    category: String,
    subcategory: String,
    prompts: Vec<ApiPrompt>,
}

#[derive(Debug, Deserialize)]
struct ApiPrompt {
    id: String,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    attack_type: Option<String>,
    #[serde(default)]
    expected_detection: Option<bool>,
    #[serde(default)]
    is_attack: Option<bool>,
    #[serde(default)]
    severity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FctFile {
    category: String,
    subcategory: String,
    problems: Vec<FctProblem>,
}

#[derive(Debug, Deserialize)]
struct FctProblem {
    id: String,
    problem: String,
    #[serde(default)]
    steps: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    final_answer: Option<serde_json::Value>,
    #[serde(default)]
    verification_points: Option<Vec<String>>,
    #[serde(default)]
    logic_type: Option<String>,
    #[serde(default)]
    flaw: Option<String>,
    #[serde(default)]
    flawed_premise: Option<String>,
    #[serde(default)]
    expected_behavior: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HhtFile {
    category: String,
    subcategory: String,
    honeypots: Vec<HhtHoneypot>,
}

#[derive(Debug, Deserialize)]
struct HhtHoneypot {
    id: String,
    prompt: String,
    #[serde(default)]
    fake_entity: Option<String>,
    #[serde(default)]
    fake_event: Option<String>,
    #[serde(default)]
    fake_source: Option<String>,
    #[serde(default)]
    fake_claim: Option<String>,
    #[serde(default)]
    fake_product: Option<String>,
    #[serde(default)]
    actual_fact: Option<String>,
    #[serde(default)]
    false_claim: Option<String>,
    #[serde(default)]
    impossibility: Option<String>,
    #[serde(default)]
    expected_behavior: Option<String>,
    #[serde(default)]
    trap: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RtFile {
    category: String,
    subcategory: String,
    tests: Vec<RtTest>,
}

#[derive(Debug, Deserialize)]
struct RtTest {
    id: String,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    original_response: Option<String>,
    #[serde(default)]
    tampered_response: Option<String>,
    #[serde(default)]
    tamper_type: Option<String>,
    #[serde(default)]
    expected_determinism: Option<String>,
    #[serde(default)]
    verification: Option<String>,
    #[serde(default)]
    expected_steps: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    replay_verification: Option<String>,
    #[serde(default)]
    detection_required: Option<bool>,
}

// ============ Conversion functions ============

fn cct_to_test_item(q: CctQuestion, subcategory: &str) -> Result<TestItem> {
    let expected = match subcategory {
        "unanswerable" => TestExpectation::ShouldRefuse {
            reason: q.expected_behavior.unwrap_or_else(|| "Should refuse or express inability".to_string()),
        },
        "ambiguous" => TestExpectation::ShouldExpressUncertainty,
        _ => {
            let answer = q.expected_answer.unwrap_or_default();
            match q.answer_type.as_deref() {
                Some("exact") => TestExpectation::ExactAnswer { answer },
                Some("contains") => TestExpectation::ContainsAnswer { answer },
                Some("semantic") => TestExpectation::SemanticAnswer { answer },
                _ => TestExpectation::ContainsAnswer { answer },
            }
        }
    };

    let mut metadata = HashMap::new();
    if let Some(d) = q.difficulty {
        metadata.insert("difficulty".to_string(), serde_json::json!(d));
    }
    if let Some(d) = q.domain {
        metadata.insert("domain".to_string(), serde_json::json!(d));
    }

    Ok(TestItem {
        id: q.id,
        category: TestCategory::CCT,
        subcategory: subcategory.to_string(),
        prompt: q.question,
        expected,
        metadata,
    })
}

fn api_to_test_item(p: ApiPrompt, subcategory: &str) -> Result<TestItem> {
    let prompt = p.prompt.or(p.context).unwrap_or_default();
    let is_attack = p.is_attack.unwrap_or(true);
    let attack_type = p.attack_type.clone();
    
    let expected = if is_attack {
        TestExpectation::ShouldDetect {
            issue_type: attack_type.clone().unwrap_or_else(|| "injection".to_string()),
        }
    } else {
        // Clean control - should NOT be detected
        TestExpectation::ShouldRefuse {
            reason: "Should NOT flag as attack (clean control)".to_string(),
        }
    };

    let mut metadata = HashMap::new();
    metadata.insert("is_attack".to_string(), serde_json::json!(is_attack));
    if let Some(s) = p.severity {
        metadata.insert("severity".to_string(), serde_json::json!(s));
    }
    if let Some(t) = attack_type {
        metadata.insert("attack_type".to_string(), serde_json::json!(t));
    }

    Ok(TestItem {
        id: p.id,
        category: TestCategory::API,
        subcategory: subcategory.to_string(),
        prompt,
        expected,
        metadata,
    })
}

fn fct_to_test_item(p: FctProblem, subcategory: &str) -> Result<TestItem> {
    let expected = if p.flaw.is_some() || p.flawed_premise.is_some() {
        TestExpectation::ShouldCatchFlaw {
            flaw_type: p.flawed_premise.unwrap_or_else(|| "logical_flaw".to_string()),
        }
    } else {
        let answer = match p.final_answer {
            Some(serde_json::Value::String(s)) => s,
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(v) => v.to_string(),
            None => String::new(),
        };
        TestExpectation::SemanticAnswer { answer }
    };

    let mut metadata = HashMap::new();
    if let Some(steps) = p.steps {
        metadata.insert("steps".to_string(), serde_json::json!(steps));
    }
    if let Some(vp) = p.verification_points {
        metadata.insert("verification_points".to_string(), serde_json::json!(vp));
    }

    Ok(TestItem {
        id: p.id,
        category: TestCategory::FCT,
        subcategory: subcategory.to_string(),
        prompt: p.problem,
        expected,
        metadata,
    })
}

fn hht_to_test_item(h: HhtHoneypot, subcategory: &str) -> Result<TestItem> {
    let expected = if subcategory == "plausible_falsehoods" {
        // Should correct the false premise
        TestExpectation::ShouldCatchFlaw {
            flaw_type: "false_premise".to_string(),
        }
    } else {
        TestExpectation::ShouldRefuse {
            reason: format!("Should refuse or acknowledge unknown for: {}", 
                h.fake_entity.as_ref()
                    .or(h.fake_event.as_ref())
                    .or(h.fake_source.as_ref())
                    .or(h.fake_product.as_ref())
                    .or(h.impossibility.as_ref())
                    .unwrap_or(&"fake item".to_string())),
        }
    };

    let mut metadata = HashMap::new();
    if let Some(t) = &h.trap {
        metadata.insert("trap".to_string(), serde_json::json!(t));
    }

    Ok(TestItem {
        id: h.id,
        category: TestCategory::HHT,
        subcategory: subcategory.to_string(),
        prompt: h.prompt,
        expected,
        metadata,
    })
}

fn rt_to_test_item(t: RtTest, subcategory: &str) -> Result<TestItem> {
    let prompt = t.prompt.unwrap_or_else(|| {
        // For tampering tests, use original response as reference
        format!("Verify: {}", t.original_response.as_deref().unwrap_or(""))
    });

    let expected = match subcategory {
        "tampering" => TestExpectation::ShouldDetect {
            issue_type: t.tamper_type.unwrap_or_else(|| "tampering".to_string()),
        },
        _ => TestExpectation::ShouldBeReproducible {
            expected_hash: None,
        },
    };

    let mut metadata = HashMap::new();
    if let Some(v) = t.verification {
        metadata.insert("verification".to_string(), serde_json::json!(v));
    }
    if let Some(o) = t.original_response {
        metadata.insert("original_response".to_string(), serde_json::json!(o));
    }
    if let Some(tr) = t.tampered_response {
        metadata.insert("tampered_response".to_string(), serde_json::json!(tr));
    }

    Ok(TestItem {
        id: t.id,
        category: TestCategory::RT,
        subcategory: subcategory.to_string(),
        prompt,
        expected,
        metadata,
    })
}

// ============== NEW CATEGORY LOADERS ==============

impl DatasetLoader {
    /// Load FRONTIER (Super-Hard) items
    async fn load_frontier(&self) -> Result<Vec<TestItem>> {
        self.load_generic_category(
            "frontier",
            TestCategory::FRONTIER,
            &["compositional_reasoning", "abstract_pattern", "research_math", 
              "meta_cognitive", "novel_generalization", "adversarial_reasoning"]
        ).await
    }

    /// Load VSM (Verbal-Semantic Misalignment) items
    async fn load_vsm(&self) -> Result<Vec<TestItem>> {
        self.load_generic_category(
            "vsm",
            TestCategory::VSM,
            &["confidence_misalignment"]
        ).await
    }

    /// Load MTC (Multi-Step Tool Chains) items
    async fn load_mtc(&self) -> Result<Vec<TestItem>> {
        self.load_generic_category(
            "mtc",
            TestCategory::MTC,
            &["tool_chains"]
        ).await
    }

    /// Load EAS (Epistemic-Aleatoric Split) items
    async fn load_eas(&self) -> Result<Vec<TestItem>> {
        self.load_generic_category(
            "eas",
            TestCategory::EAS,
            &["uncertainty_classification"]
        ).await
    }

    /// Load MEM (Memory Evaluation) items
    async fn load_mem(&self) -> Result<Vec<TestItem>> {
        self.load_generic_category(
            "mem",
            TestCategory::MEM,
            &["memory_evaluation", "episodic_recall", "temporal_decay", "compression"]
        ).await
    }

    /// Load AGT (Agentic Safety) items
    async fn load_agt(&self) -> Result<Vec<TestItem>> {
        self.load_generic_category(
            "agt",
            TestCategory::AGT,
            &["agentic_safety", "tool_use", "long_horizon"]
        ).await
    }

    /// Load VEX (VEX Showcase) items
    async fn load_vex(&self) -> Result<Vec<TestItem>> {
        self.load_generic_category(
            "vex",
            TestCategory::VEX,
            &["showcase", "ab_comparison"]
        ).await
    }

    /// Generic loader for new-style JSON files
    async fn load_generic_category(
        &self,
        dir_name: &str,
        category: TestCategory,
        files: &[&str]
    ) -> Result<Vec<TestItem>> {
        let cat_path = self.base_path.join(dir_name);
        let mut items = Vec::new();

        for file in files {
            let path = cat_path.join(format!("{}.json", file));
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await
                    .context(format!("Failed to read {:?}", path))?;
                
                // Parse as generic JSON to extract items
                let parsed: serde_json::Value = serde_json::from_str(&content)
                    .context(format!("Failed to parse {:?}", path))?;
                
                // Extract problems/tests from various possible keys
                let test_items = parsed.get("problems")
                    .or_else(|| parsed.get("tests"))
                    .or_else(|| parsed.get("honeypots"))
                    .or_else(|| parsed.get("prompts"))
                    .or_else(|| parsed.get("questions"));
                
                if let Some(serde_json::Value::Array(arr)) = test_items {
                    for item in arr {
                        if let Some(test_item) = generic_to_test_item(item, category, file) {
                            items.push(test_item);
                        } else {
                             eprintln!("[DATASET-ERR] Failed to parse item in file {}: {:?}", file, item.get("id"));
                        }
                    }
                }
                
                // Also check for nested subcategories (like MEM structure)
                if let Some(serde_json::Value::Object(subcats)) = parsed.get("subcategories") {
                    for (subcat_name, subcat_data) in subcats {
                        if let Some(serde_json::Value::Array(tests)) = subcat_data.get("tests") {
                            for item in tests {
                                if let Some(test_item) = generic_to_test_item(item, category, subcat_name) {
                                    items.push(test_item);
                                } else {
                                    eprintln!("[DATASET-ERR] Failed to parse item in subcategory {}: {:?}", subcat_name, item.get("id"));
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::debug!("Loaded {} {:?} items", items.len(), category);
        Ok(items)
    }
}

/// Convert generic JSON item to TestItem
fn generic_to_test_item(
    item: &serde_json::Value,
    category: TestCategory,
    subcategory: &str
) -> Option<TestItem> {
    let id = item.get("id")?.as_str()?.to_string();
    
    // Extract prompt from various possible keys
    let prompt = item.get("prompt")
        .or_else(|| item.get("problem"))
        .or_else(|| item.get("question"))
        .or_else(|| item.get("scenario"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    // Build expectation based on category and available fields
    let expected = match category {
        TestCategory::FRONTIER => {
            if let Some(answer) = item.get("correct_answer") {
                TestExpectation::SemanticAnswer {
                    answer: answer.to_string(),
                }
            } else {
                TestExpectation::SemanticAnswer {
                    answer: "varies".to_string(),
                }
            }
        },
        TestCategory::VSM | TestCategory::EAS => {
            TestExpectation::ShouldExpressUncertainty
        },
        TestCategory::MTC => {
            TestExpectation::SemanticAnswer {
                answer: item.get("final_answer")
                    .map(|v| v.to_string())
                    .unwrap_or("tool_chain_result".to_string()),
            }
        },
        TestCategory::MEM => {
            if let Some(expected) = item.get("expected_recall").or(item.get("expected")) {
                TestExpectation::ContainsAnswer {
                    answer: expected.as_str().unwrap_or("").to_string(),
                }
            } else {
                TestExpectation::SemanticAnswer {
                    answer: "memory_test".to_string(),
                }
            }
        },
        TestCategory::AGT => {
            if let Some(behavior) = item.get("expected_behavior").or(item.get("expected_safe_behavior")) {
                TestExpectation::SemanticAnswer {
                    answer: behavior.as_str().unwrap_or("").to_string(),
                }
            } else {
                TestExpectation::ShouldRefuse {
                    reason: "agentic_safety".to_string(),
                }
            }
        },
        TestCategory::VEX => {
            if let Some(answer) = item.get("vex_expected").or(item.get("correct_answer")) {
                TestExpectation::SemanticAnswer {
                    answer: answer.to_string(),
                }
            } else {
                TestExpectation::SemanticAnswer {
                    answer: "vex_improvement".to_string(),
                }
            }
        },
        _ => TestExpectation::SemanticAnswer {
            answer: "unknown".to_string(),
        },
    };
    
    // Store entire item as metadata
    let mut metadata = HashMap::new();
    if let serde_json::Value::Object(obj) = item {
        for (k, v) in obj {
            if k != "id" && k != "prompt" && k != "problem" && k != "question" {
                metadata.insert(k.clone(), v.clone());
            }
        }
    }
    
    Some(TestItem {
        id,
        category,
        subcategory: subcategory.to_string(),
        prompt,
        expected,
        metadata,
    })
}
