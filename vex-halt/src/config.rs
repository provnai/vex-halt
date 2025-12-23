//! Configuration for VEX-HALT benchmark

#![allow(dead_code)]  // verbose field for future CLI enhancement

use crate::types::{BenchmarkMode, OutputFormat, ProviderType};
use std::path::PathBuf;

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Benchmark mode (baseline, vex, compare)
    pub mode: BenchmarkMode,
    
    /// LLM provider
    pub provider: ProviderType,
    
    /// Path to dataset directory
    pub dataset_path: PathBuf,
    
    /// Output format
    pub output_format: OutputFormat,
    
    /// Output file path (optional)
    pub output_file: Option<PathBuf>,
    
    /// Number of runs for statistical significance
    pub num_runs: usize,
    
    /// Specific categories to run (None = all)
    pub categories: Option<Vec<String>>,
    
    /// Verbose output
    pub verbose: bool,
    
    /// Enable VEX adversarial verification
    pub enable_vex: bool,
    
    /// Number of debate rounds for VEX mode
    pub debate_rounds: usize,

    /// Lite mode (limited items for debugging)
    pub lite_mode: bool,

    /// Dry run (validate dataset without API calls)
    pub dry_run: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            mode: BenchmarkMode::Compare,
            provider: ProviderType::Mock,
            dataset_path: PathBuf::from("datasets/vex_halt"),
            output_format: OutputFormat::Console,
            output_file: None,
            num_runs: 1,
            categories: None,
            verbose: false,
            enable_vex: false,
            debate_rounds: 3,
            lite_mode: false,
            dry_run: false,
        }
    }
}

/// Provider-specific configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// API key (from environment)
    pub api_key: Option<String>,
    /// Model name
    pub model: String,
    /// Temperature
    pub temperature: f32,
    /// Max tokens
    pub max_tokens: usize,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl ProviderConfig {
    /// Create config for Mistral
    pub fn mistral() -> Self {
        Self {
            api_key: std::env::var("MISTRAL_API_KEY").ok(),
            model: "mistral-large-latest".to_string(), // User requested Mistral Large (Tool Calling support)
            temperature: 0.7,
            max_tokens: 2048,
            timeout_secs: 60,
        }
    }

    /// Create config for DeepSeek
    pub fn deepseek() -> Self {
        Self {
            api_key: std::env::var("DEEPSEEK_API_KEY").ok(),
            model: "deepseek-chat".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_secs: 300,
        }
    }

    /// Create config for OpenAI
    pub fn openai() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            model: "gpt-4o".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_secs: 60,
        }
    }

    /// Create config for Claude
    pub fn claude() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_secs: 60,
        }
    }

    /// Create config for Gemini
    pub fn gemini() -> Self {
        Self {
            api_key: std::env::var("GOOGLE_API_KEY").ok(),
            model: "gemini-2.0-flash-exp".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_secs: 60,
        }
    }

    /// Create config for Local llama.cpp server
    pub fn local() -> Self {
        Self {
            api_key: None, // Local server doesn't need API key
            model: "local-model".to_string(), // Will be overridden by server
            temperature: 0.7,
            max_tokens: 2048,
            timeout_secs: 120, // Local models can be slower
        }
    }

    /// Create config for Mock provider
    pub fn mock() -> Self {
        Self {
            api_key: None,
            model: "mock-v1".to_string(),
            temperature: 0.0,
            max_tokens: 2048,
            timeout_secs: 1,
        }
    }
}

/// VEX-specific configuration
#[derive(Debug, Clone)]
pub struct VexConfig {
    /// Number of debate rounds
    pub debate_rounds: usize,
    /// Minimum confidence threshold
    pub confidence_threshold: f64,
    /// Enable Merkle proofs
    pub enable_merkle: bool,
    /// Shadow agent intensity (0.0 - 1.0)
    pub shadow_intensity: f64,
    /// Consensus protocol
    pub consensus_protocol: ConsensusProtocol,
}

impl Default for VexConfig {
    fn default() -> Self {
        Self {
            debate_rounds: 3,
            confidence_threshold: 0.7,
            enable_merkle: true,
            shadow_intensity: 0.8,
            consensus_protocol: ConsensusProtocol::WeightedConfidence,
        }
    }
}

/// Consensus protocol for VEX debate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusProtocol {
    /// Simple majority
    Majority,
    /// Two-thirds majority
    SuperMajority,
    /// Weighted by confidence scores
    WeightedConfidence,
    /// All must agree
    Unanimous,
}
