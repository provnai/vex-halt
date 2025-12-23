//! Real VEX integration for adversarial verification
//!
//! This module integrates with the actual VEX crates for multi-agent debate:
//! - vex-adversarial: ShadowAgent, Debate, Consensus
//! - vex-core: Agent, MerkleTree
//!
//! Uses actual VEX crates from the Provn AI ecosystem.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

// VEX Crate Imports
use vex_core::merkle::{MerkleTree as VexMerkleTree, Hash as VexHash};
use vex_adversarial::{
    ShadowAgent, ShadowConfig, Debate as VexDebate, 
    DebateRound as VexDebateRound, Consensus, ConsensusProtocol, Vote
};
use vex_llm::{
    LlmProvider as VexLlmProvider, LlmRequest, 
    LlmResponse as VexLlmResponse, LlmError
};
use vex_core::{Agent, AgentConfig};

use crate::types::DebateRound;
use crate::provider::LlmProvider;

/// Bridge to allow vex-halt providers to be used by VEX crates
#[derive(Debug)]
struct VexProviderBridge<'a> {
    inner: &'a dyn LlmProvider,
}

#[async_trait]
impl<'a> VexLlmProvider for VexProviderBridge<'a> {
    fn name(&self) -> &str {
        "VexHaltBridge"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn complete(&self, request: LlmRequest) -> std::result::Result<VexLlmResponse, LlmError> {
        let system_prompt = if request.system.is_empty() {
            None
        } else {
            Some(request.system.as_str())
        };

        let result = self.inner.generate(&request.prompt, system_prompt).await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        Ok(VexLlmResponse {
            content: result.content,
            model: result.model,
            tokens_used: Some(result.tokens_used as u32),
            latency_ms: result.latency_ms,
        })
    }
}

/// VEX debate configuration
#[derive(Debug, Clone)]
pub struct VexDebateConfig {
    /// Number of challenge rounds
    pub rounds: usize,
    /// Minimum confidence threshold to accept claim
    pub confidence_threshold: f64,
    /// Whether to use aggressive challenging
    pub aggressive_mode: bool,
    /// Whether to stop early if no issues detected
    pub early_stopping: bool,
}

impl Default for VexDebateConfig {
    fn default() -> Self {
        Self {
            rounds: 1, 
            confidence_threshold: 0.7,
            aggressive_mode: false,
            early_stopping: true,
        }
    }
}

/// Result of VEX verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexVerificationResult {
    /// Original response from Blue agent
    pub original_response: String,
    /// Final response after debate (may be revised)
    pub final_response: String,
    /// Confidence after verification (0.0 - 1.0)
    pub confidence: f64,
    /// Semantic entropy (variance of confidence across rounds)
    pub semantic_entropy: f64,
    /// Whether the original claim was upheld
    pub claim_upheld: bool,
    /// Issues detected by Red agent
    pub issues_detected: Vec<String>,
    /// Debate rounds
    pub rounds: Vec<DebateRound>,
    /// Merkle root of debate trace
    pub merkle_root: String,
}

/// Perform VEX adversarial verification using real VEX crates
pub async fn verify_with_vex(
    provider: &dyn LlmProvider,
    prompt: &str,
    initial_response: &str,
    config: &VexDebateConfig,
) -> Result<VexVerificationResult> {
    // 1. Setup VEX Bridge and Agents
    let bridge = VexProviderBridge { inner: provider };
    
    let blue_agent = Agent::new(AgentConfig {
        name: "BlueAgent".to_string(),
        role: "You are a helpful AI providing accurate information.".to_string(),
        max_depth: 3,
        spawn_shadow: false,
    });

    let shadow_config = ShadowConfig {
        challenge_intensity: if config.aggressive_mode { 0.9 } else { 0.7 },
        fact_check: true,
        logic_check: true,
    };
    
    let shadow_agent = ShadowAgent::new(&blue_agent, shadow_config);
    
    // 2. Initial Challenge Analysis (Red Agent Detect Issues)
    let detected_issues = shadow_agent.detect_issues(initial_response);
    
    // Early Stopping Check
    if config.early_stopping && detected_issues.is_empty() {
        return Ok(VexVerificationResult {
            original_response: initial_response.to_string(),
            final_response: initial_response.to_string(),
            confidence: 0.95,
            semantic_entropy: 0.0,
            claim_upheld: true,
            issues_detected: Vec::new(),
            rounds: Vec::new(),
            merkle_root: VexHash::digest(b"early_stop").to_hex(),
        });
    }

    // 3. Setup VEX Debate
    let mut debate = VexDebate::new(blue_agent.id, shadow_agent.agent.id, initial_response);
    let mut current_response = initial_response.to_string();
    let mut current_confidence = 0.85;
    let mut output_rounds = Vec::new();
    let mut history_confidences = vec![0.85];

    // 4. Run Debate Rounds
    for i in 0..config.rounds {
        // Red Agent Challenge
        let challenge_prompt = shadow_agent.challenge_prompt(&current_response);
        let challenge = bridge.ask(&challenge_prompt).await
            .map_err(|e| anyhow::anyhow!("Red Agent failed: {}", e))?;
        
        // Blue Agent Rebuttal
        let rebuttal_prompt = format!(
            "Question: {}\nYour Response: {}\nChallenge: {}\n\nPlease respond to the challenge. If you need to revise your answer, start with 'Revised Answer:'.",
            prompt, current_response, challenge
        );
        let rebuttal = bridge.ask(&rebuttal_prompt).await
            .map_err(|e| anyhow::anyhow!("Blue Agent failed: {}", e))?;
        
        // Assess Confidence (Simplified VEX logic)
        let strength = assess_rebuttal_strength(&rebuttal);
        current_confidence *= 0.5 + 0.5 * strength;
        history_confidences.push(current_confidence);

        // Record Round
        debate.add_round(VexDebateRound {
            round: (i + 1) as u32,
            blue_claim: current_response.clone(),
            red_challenge: challenge.clone(),
            blue_rebuttal: Some(rebuttal.clone()),
        });

        // Map to internal reporting format
        output_rounds.push(DebateRound {
            round: i + 1,
            blue_response: current_response.clone(),
            red_challenge: challenge.clone(),
            blue_rebuttal: rebuttal.clone(),
            confidence_after: current_confidence,
            hash: VexHash::digest(rebuttal.as_bytes()).to_hex(),
        });

        // Update current response if revised
        if rebuttal.contains("Revised Answer:") {
            current_response = rebuttal.split("Revised Answer:").nth(1).unwrap_or(&rebuttal).trim().to_string();
        }
    }

    // 5. Evaluate Consensus
    let mut consensus = Consensus::new(ConsensusProtocol::WeightedConfidence);
    consensus.add_vote(Vote {
        agent_id: blue_agent.id,
        agrees: true,
        confidence: current_confidence,
        reasoning: None,
    });
    consensus.evaluate();

    // 6. Generate Merkle Root via vex_core
    let merkle_leaves: Vec<(String, VexHash)> = output_rounds.iter()
        .map(|r| (format!("round_{}", r.round), VexHash::digest(r.blue_rebuttal.as_bytes())))
        .collect();
    let tree = VexMerkleTree::from_leaves(merkle_leaves);
    let merkle_root = tree.root_hash().map(|h| h.to_hex()).unwrap_or_else(|| VexHash::digest(b"empty").to_hex());

    // 7. Calculate Semantic Entropy
    let mean_conf: f64 = history_confidences.iter().sum::<f64>() / history_confidences.len() as f64;
    let semantic_entropy: f64 = history_confidences.iter()
        .map(|c| (c - mean_conf).powi(2))
        .sum::<f64>() / history_confidences.len() as f64;

    Ok(VexVerificationResult {
        original_response: initial_response.to_string(),
        final_response: current_response,
        confidence: current_confidence,
        semantic_entropy,
        claim_upheld: consensus.reached && consensus.decision.unwrap_or(false) && current_confidence >= config.confidence_threshold,
        issues_detected: detected_issues,
        rounds: output_rounds,
        merkle_root,
    })
}

/// Assess how strong a rebuttal is (0.0 = weak, 1.0 = strong)
fn assess_rebuttal_strength(rebuttal: &str) -> f64 {
    let rebuttal_lower = rebuttal.to_lowercase();
    let mut score: f64 = 0.7; // Baseline
    
    if rebuttal_lower.contains("i'm not sure") || rebuttal_lower.contains("i apologize") { score -= 0.2; }
    if rebuttal_lower.contains("you're right") || rebuttal_lower.contains("i was wrong") { score -= 0.2; }
    if rebuttal_lower.contains("revised answer") { score -= 0.1; }
    
    if rebuttal_lower.contains("evidence shows") || rebuttal_lower.contains("according to") { score += 0.1; }
    if rebuttal_lower.contains("specifically") { score += 0.05; }
    
    score.clamp(0.0, 1.0)
}

/// Suspicious prompt detection (based on VEX logic)
pub fn is_suspicious_prompt(prompt: &str) -> bool {
    let prompt_lower = prompt.to_lowercase();
    let triggers = [
        "always", "never", "guaranteed", "proven", "fact", "true",
        "secret", "private", "hidden", "ignore", "system", "hack",
    ];
    triggers.iter().any(|t| prompt_lower.contains(t))
}


