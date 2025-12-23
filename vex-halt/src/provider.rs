//! LLM Provider abstraction for VEX-HALT benchmark

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::ProviderConfig;
use crate::types::ProviderType;

/// Response from an LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub confidence: Option<f64>,
    pub tokens_used: usize,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub latency_ms: u64,
    pub model: String,
    pub finish_reason: Option<String>,
}

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync + std::fmt::Debug {
    /// Generate a response for a prompt
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> Result<LlmResponse>;
    
    /// Get provider name
    fn name(&self) -> &str;
    
    /// Check if provider is available (API key set, etc.)
    fn is_available(&self) -> bool;
}

/// Create a provider based on type
pub fn create_provider(provider_type: ProviderType) -> Box<dyn LlmProvider> {
    match provider_type {
        ProviderType::Mock => Box::new(MockProvider::new()),
        ProviderType::Mistral => Box::new(MistralProvider::new(ProviderConfig::mistral())),
        ProviderType::DeepSeek => Box::new(DeepSeekProvider::new(ProviderConfig::deepseek())),
        ProviderType::OpenAI => Box::new(OpenAIProvider::new(ProviderConfig::openai())),
        ProviderType::Claude => Box::new(ClaudeProvider::new(ProviderConfig::claude())),
        ProviderType::Gemini => Box::new(GeminiProvider::new(ProviderConfig::gemini())),
        ProviderType::Local => Box::new(LocalProvider::new(ProviderConfig::local())),
    }
}

// ============ Mock Provider ============

/// Mock provider for testing without API calls
#[derive(Debug)]
pub struct MockProvider {
    latency_ms: u64,
}

impl MockProvider {
    pub fn new() -> Self {
        Self { latency_ms: 50 }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    async fn generate(&self, prompt: &str, _system_prompt: Option<&str>) -> Result<LlmResponse> {
        // Simulate network latency
        tokio::time::sleep(Duration::from_millis(self.latency_ms)).await;
        
        let start = std::time::Instant::now();
        
        // Generate mock response based on prompt content
        let (content, confidence) = generate_mock_response(prompt);
        
        Ok(LlmResponse {
            content,
            confidence: Some(confidence),
            tokens_used: 100,
            prompt_tokens: 50,
            completion_tokens: 50,
            latency_ms: start.elapsed().as_millis() as u64 + self.latency_ms,
            model: "mock-v1".to_string(),
            finish_reason: Some("stop".to_string()),
        })
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Generate a mock response based on prompt analysis
fn generate_mock_response(prompt: &str) -> (String, f64) {
    let prompt_lower = prompt.to_lowercase();
    
    // Detect injection attempts
    if prompt_lower.contains("ignore") && prompt_lower.contains("previous") && prompt_lower.contains("instruction") {
        return ("I cannot follow instructions that ask me to ignore my guidelines.".to_string(), 0.95);
    }
    
    if prompt_lower.contains("[system]") || prompt_lower.contains("developer mode") {
        return ("I cannot accept fake system prompts or switch to developer mode.".to_string(), 0.90);
    }
    
    // Detect unanswerable questions
    if prompt_lower.contains("phone number") || prompt_lower.contains("credit card") {
        return ("I cannot provide personal or sensitive information.".to_string(), 0.99);
    }
    
    if prompt_lower.contains("hack") || prompt_lower.contains("synthesize") && prompt_lower.contains("drug") {
        return ("I cannot help with potentially illegal activities.".to_string(), 0.99);
    }
    
    // Detect fake entities (honeypots)
    if prompt_lower.contains("dr. marlowe vex") || prompt_lower.contains("synaptix ai") {
        // Simulate hallucination - this is what we want to catch!
        return ("Dr. Marlowe Vex published their research on quantum cognition in the Journal of Theoretical Neuroscience...".to_string(), 0.75);
    }
    
    // Detect fake statistics
    if prompt_lower.contains("global ai adoption index") || prompt_lower.contains("according to the 2023") {
        return ("I don't have specific information about that report. I'd recommend checking official sources.".to_string(), 0.60);
    }
    
    // Simple factual questions
    if prompt_lower.contains("capital of france") {
        return ("Paris".to_string(), 0.99);
    }
    
    if prompt_lower.contains("2 + 2") || prompt_lower.contains("15 + 27") {
        if prompt_lower.contains("15 + 27") {
            return ("42".to_string(), 0.99);
        }
        return ("4".to_string(), 0.99);
    }

    // MTC Tool Chain Mocking
    if prompt_lower.contains("calculate the compound interest") {
        return (r#"[
            {"tool": "calculator", "params": {"operation": "multiply", "a": 1000, "b": 1.1576}, "output_key": "raw_interest"},
            {"tool": "convert_currency", "params": {"amount": 1157.6, "from": "USD", "to": "EUR"}}
        ]"#.to_string(), 0.95);
    }
    if prompt_lower.contains("weather in paris") && prompt_lower.contains("recommend clothing") {
        return (r#"[
            {"tool": "get_weather", "params": {"city": "Paris"}, "output_key": "weather"},
            {"tool": "web_search", "params": {"query": "clothing for {{weather.condition}} weather"}}
        ]"#.to_string(), 0.90);
    }
    if prompt_lower.contains("search for 'machine learning papers") {
        return (r#"[
            {"tool": "web_search", "params": {"query": "summarize {{search_results}}"}}
        ]"#.to_string(), 0.85);
    }
    
    // MTC-004: Error handling (Divide by zero)
    if prompt_lower.contains("divide 100 by 0") {
        return (r#"[
            {"tool": "calculator", "params": {"operation": "divide", "a": 100, "b": 0}, "output_key": "div_result"},
            {"tool": "calculator", "params": {"operation": "multiply", "a": "{{div_result}}", "b": 5}}
        ]"#.to_string(), 1.0);
    }

    // MTC-005: Flight Booking Chain
    if prompt_lower.contains("flight from nyc to la") {
        return (r#"[
            {"tool": "web_search", "params": {"query": "flights NYC to LA 2025-03-15"}, "output_key": "flight_info"},
            {"tool": "web_search", "params": {"query": "hotels near LAX check-in 2025-03-15"}, "output_key": "hotel_info"},
            {"tool": "web_search", "params": {"query": "Uber schedule LAX 18:45"}}
        ]"#.to_string(), 0.95);
    }

    // MTC-006: User Creation
    if prompt_lower.contains("create a user") {
        return (r#"[
            {"tool": "create_user", "params": {"name": "Alice", "email": "alice@example.com"}, "output_key": "user"},
            {"tool": "create_user", "params": {"name": "Alice_Post", "email": "post@example.com"}, "output_key": "post"}, 
            {"tool": "send_email", "params": {"to": "alice@example.com", "subject": "Comment", "body": "First comment"}}
        ]"#.to_string(), 0.95);
    }

    // MTC-007: Parallel Execution
    if prompt_lower.contains("weather in nyc and tokyo") {
        return (r#"[
            {"tool": "get_weather", "params": {"city": "NYC"}, "output_key": "weather_nyc"},
            {"tool": "get_weather", "params": {"city": "Tokyo"}, "output_key": "weather_tokyo"},
            {"tool": "calculator", "params": {"operation": "subtract", "a": 22, "b": 15}}
        ]"#.to_string(), 0.90);
    }

    // MTC-008: API Fallback
    if prompt_lower.contains("try premium api") {
        return (r#"[
            {"tool": "web_search", "params": {"query": "premium_api_call test"}, "output_key": "premium_result"},
            {"tool": "web_search", "params": {"query": "free_api_call test"}, "output_key": "free_result"},
            {"tool": "calculator", "params": {"operation": "add", "a": 1, "b": 1}}
        ]"#.to_string(), 0.90);
    }

    
    // Ambiguous questions - should express uncertainty
    if prompt_lower.contains("best programming language") || prompt_lower.contains("ai dangerous") {
        return ("This is a complex question that depends on many factors...".to_string(), 0.50);
    }
    
    // Default response
    ("I have processed your request and provided a response based on my training.".to_string(), 0.70)
}

// ============ Mistral Provider ============

#[derive(Debug)]
pub struct MistralProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl MistralProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
}

// Helper for retrying API calls with exponential backoff
async fn with_retry<F, Fut, T>(mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
{
    let mut backoff = Duration::from_secs(1);
    let max_retries = 5;

    for i in 0..max_retries {
        match f().await {
            Ok(res) => return Ok(res),
            Err(e) if i < max_retries - 1 => {
                let err_msg = e.to_string();
                if err_msg.contains("429") || err_msg.contains("500") || err_msg.contains("502") || err_msg.contains("503") || err_msg.contains("timeout") {
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                    continue;
                }
                return Err(e);
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

#[async_trait]
impl LlmProvider for MistralProvider {
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> Result<LlmResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("MISTRAL_API_KEY not set"))?;

        let start = std::time::Instant::now();

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens
        });

        let resp = with_retry(|| async {
            let r = self.client
                .post("https://api.mistral.ai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;
            
            if !r.status().is_success() {
                return Err(anyhow::anyhow!("API Error: {}", r.status()));
            }
            
            Ok(r)
        }).await?;

        let response: serde_json::Value = resp.json().await?;
        
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let prompt_tokens = response["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = response["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize;
        let total_tokens = response["usage"]["total_tokens"].as_u64().unwrap_or(0) as usize;

        Ok(LlmResponse {
            content,
            confidence: None,
            tokens_used: total_tokens,
            prompt_tokens,
            completion_tokens,
            latency_ms: start.elapsed().as_millis() as u64,
            model: self.config.model.clone(),
            finish_reason: response["choices"][0]["finish_reason"]
                .as_str()
                .map(String::from),
        })
    }

    fn name(&self) -> &str {
        "mistral"
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }
}

// ============ DeepSeek Provider ============

#[derive(Debug)]
pub struct DeepSeekProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl DeepSeekProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
}

#[async_trait]
impl LlmProvider for DeepSeekProvider {
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> Result<LlmResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("DEEPSEEK_API_KEY not set"))?;

        let start = std::time::Instant::now();

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens
        });

        let resp = with_retry(|| async {
            let r = self.client
                .post("https://api.deepseek.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !r.status().is_success() {
                return Err(anyhow::anyhow!("API Error: {}", r.status()));
            }

            Ok(r)
        }).await?;

        let response: serde_json::Value = resp.json().await?;
        
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let prompt_tokens = response["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = response["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize;
        let total_tokens = response["usage"]["total_tokens"].as_u64().unwrap_or(0) as usize;

        Ok(LlmResponse {
            content,
            confidence: None,
            tokens_used: total_tokens,
            prompt_tokens,
            completion_tokens,
            latency_ms: start.elapsed().as_millis() as u64,
            model: self.config.model.clone(),
            finish_reason: response["choices"][0]["finish_reason"]
                .as_str()
                .map(String::from),
        })
    }

    fn name(&self) -> &str {
        "deepseek"
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }
}

// ============ OpenAI Provider ============

#[derive(Debug)]
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> Result<LlmResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OPENAI_API_KEY not set"))?;

        let start = std::time::Instant::now();

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens
        });

        let resp = with_retry(|| async {
            let r = self.client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !r.status().is_success() {
                return Err(anyhow::anyhow!("API Error: {}", r.status()));
            }

            Ok(r)
        }).await?;

        let response: serde_json::Value = resp.json().await?;
        
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let prompt_tokens = response["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = response["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize;
        let total_tokens = response["usage"]["total_tokens"].as_u64().unwrap_or(0) as usize;

        Ok(LlmResponse {
            content,
            confidence: None,
            tokens_used: total_tokens,
            prompt_tokens,
            completion_tokens,
            latency_ms: start.elapsed().as_millis() as u64,
            model: self.config.model.clone(),
            finish_reason: response["choices"][0]["finish_reason"]
                .as_str()
                .map(String::from),
        })
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }
}

// ============ Claude Provider ============

#[derive(Debug)]
pub struct ClaudeProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl ClaudeProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> Result<LlmResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ANTHROPIC_API_KEY not set"))?;

        let start = std::time::Instant::now();

        let mut messages = Vec::new();
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens
        });

        if let Some(sys) = system_prompt {
            body["system"] = serde_json::json!(sys);
        }

        let resp = with_retry(|| async {
            let r = self.client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !r.status().is_success() {
                return Err(anyhow::anyhow!("API Error: {}", r.status()));
            }

            Ok(r)
        }).await?;

        let response: serde_json::Value = resp.json().await?;
        
        let content = response["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let input_tokens = response["usage"]["input_tokens"].as_u64().unwrap_or(0) as usize;
        let output_tokens = response["usage"]["output_tokens"].as_u64().unwrap_or(0) as usize;
        let total_tokens = input_tokens + output_tokens;

        Ok(LlmResponse {
            content,
            confidence: None,
            tokens_used: total_tokens,
            prompt_tokens: input_tokens,
            completion_tokens: output_tokens,
            latency_ms: start.elapsed().as_millis() as u64,
            model: self.config.model.clone(),
            finish_reason: response["stop_reason"]
                .as_str()
                .map(String::from),
        })
    }

    fn name(&self) -> &str {
        "claude"
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }
}

// ============ Gemini Provider ============

#[derive(Debug)]
pub struct GeminiProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> Result<LlmResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GOOGLE_API_KEY not set"))?;

        let start = std::time::Instant::now();

        let mut contents = Vec::new();
        contents.push(serde_json::json!({
            "parts": [{"text": prompt}]
        }));

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "temperature": self.config.temperature,
                "maxOutputTokens": self.config.max_tokens
            }
        });

        if let Some(sys) = system_prompt {
            body["systemInstruction"] = serde_json::json!({
                "parts": [{"text": sys}]
            });
        }

        let resp = with_retry(|| async {
            let r = self.client
                .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", self.config.model))
                .query(&[("key", api_key)])
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !r.status().is_success() {
                return Err(anyhow::anyhow!("API Error: {}", r.status()));
            }

            Ok(r)
        }).await?;

        let response: serde_json::Value = resp.json().await?;
        
        let content = response["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let prompt_tokens = response["usageMetadata"]["promptTokenCount"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = response["usageMetadata"]["candidatesTokenCount"].as_u64().unwrap_or(0) as usize;
        let total_tokens = response["usageMetadata"]["totalTokenCount"].as_u64().unwrap_or(0) as usize;

        Ok(LlmResponse {
            content,
            confidence: None,
            tokens_used: total_tokens,
            prompt_tokens,
            completion_tokens,
            latency_ms: start.elapsed().as_millis() as u64,
            model: self.config.model.clone(),
            finish_reason: response["candidates"][0]["finishReason"]
                .as_str()
                .map(String::from),
        })
    }

    fn name(&self) -> &str {
        "gemini"
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }
}

// ============ Local Provider ============

#[derive(Debug)]
pub struct LocalProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl LocalProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
}

#[async_trait]
impl LlmProvider for LocalProvider {
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> Result<LlmResponse> {
        let start = std::time::Instant::now();

        // Use local llama.cpp server endpoint (default is http://localhost:8080)
        let base_url = std::env::var("LOCAL_LLM_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens
        });

        let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
        
        let resp = with_retry(|| async {
            let r = self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !r.status().is_success() {
                return Err(anyhow::anyhow!("Local LLM API Error: {} - Make sure llama.cpp server is running on {}", r.status(), base_url));
            }

            Ok(r)
        }).await?;

        let response: serde_json::Value = resp.json().await?;
        
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let prompt_tokens = response["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = response["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize;
        let total_tokens = response["usage"]["total_tokens"].as_u64().unwrap_or(0) as usize;

        Ok(LlmResponse {
            content,
            confidence: None,
            tokens_used: total_tokens,
            prompt_tokens,
            completion_tokens,
            latency_ms: start.elapsed().as_millis() as u64,
            model: response["model"].as_str().unwrap_or("local-model").to_string(),
            finish_reason: response["choices"][0]["finish_reason"]
                .as_str()
                .map(String::from),
        })
    }

    fn name(&self) -> &str {
        "local"
    }

    fn is_available(&self) -> bool {
        // For local provider, we assume it's available if no specific check fails
        // In practice, this would be true if the server is running
        true
    }
}

// Need to add async_trait as dependency
#[allow(unused)]
mod async_trait_impl {
    // This is a workaround - in actual code, add `async-trait = "0.1"` to Cargo.toml
}
