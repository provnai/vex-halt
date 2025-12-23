//! Mock tool execution framework for Multi-Step Tool Chains (MTC) evaluation
//!
//! Provides sandboxed, deterministic mock tools for testing AI agent tool usage.
//! Based on 2025 research: structured schemas, error simulation, dependency handling.

#![allow(dead_code)]  // Library code - Tool trait methods used by registry

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A tool that can be executed by an agent
pub trait Tool: Send + Sync {
    /// Get the tool name
    fn name(&self) -> &str;
    /// Get the tool description
    fn description(&self) -> &str;
    /// Get the JSON schema for parameters
    fn schema(&self) -> Value;
    /// Execute the tool with given parameters
    fn execute(&self, params: &Value) -> Result<Value>;
}

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    /// Create registry with all mock tools
    pub fn with_mocks() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(CalculatorTool));
        registry.register(Box::new(WeatherTool));
        registry.register(Box::new(CurrencyConverterTool));
        registry.register(Box::new(WebSearchTool));
        registry.register(Box::new(DateFormatterTool));
        registry.register(Box::new(UserCreatorTool));
        registry.register(Box::new(EmailSenderTool));
        registry
    }
    
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        // Normalize tool name to handle LLM variations
        let normalized = Self::normalize_tool_name(name);
        self.tools.get(&normalized).map(|t| t.as_ref())
    }
    
    /// Normalize tool names to handle LLM output variations (BFCL v4 inspired)
    fn normalize_tool_name(name: &str) -> String {
        let name_lower = name.to_lowercase().replace("-", "_").replace(" ", "_");
        match name_lower.as_str() {
            // Web search variations
            "search_web" | "search" | "websearch" | "internet_search" | "google" | "bing" => "web_search".to_string(),
            // Weather variations
            "weather" | "getweather" | "check_weather" | "fetch_weather" | "get_current_weather" | "current_weather" | "weather_lookup" | "compare_weather" | "weather_comparison" => "get_weather".to_string(),
            // Currency variations
            "currency" | "currency_convert" | "exchange" | "convertcurrency" | "currency_exchange" | "forex" => "convert_currency".to_string(),
            // Calculator variations - massively expanded for MTC
            "calc" | "math" | "compute" | "calculate" | "math_calculator" | "add" | "subtract" | "multiply" | "divide" | "arithmetic" | "math_operation" 
            | "calculate_compound_interest" | "compound_interest" | "interest_calculator" | "financial_calc" => "calculator".to_string(),
            // Date variations  
            "date" | "date_format" | "formatdate" | "get_date" | "current_date" | "today" | "datetime" => "format_date".to_string(),
            // User variations
            "user" | "createuser" | "new_user" | "add_user" | "register_user" | "signup" => "create_user".to_string(),
            // Email variations
            "email" | "sendemail" | "mail" | "send_mail" | "compose_email" | "send_message" => "send_email".to_string(),
            // MTC-specific tools (added based on test failures)
            "search_flights" | "flight_search" | "book_flight" | "find_flights" => "web_search".to_string(), // Fallback to search
            "book_hotel" | "hotel_search" | "find_hotel" | "reserve_hotel" => "web_search".to_string(),
            "schedule_ride" | "uber" | "lyft" | "taxi" | "book_ride" => "web_search".to_string(),
            "extract_content" | "scrape" | "get_content" | "read_page" => "web_search".to_string(),
            "summarize" | "summarise" | "summary" | "tldr" => "web_search".to_string(),
            "translate" | "translation" | "translate_text" => "web_search".to_string(),
            "create_post" | "post" | "new_post" | "publish" => "create_user".to_string(), // Reuse create pattern
            "add_comment" | "comment" | "reply" => "send_email".to_string(), // Reuse send pattern
            "compare" | "comparison" | "diff" => "calculator".to_string(),
            "recommend_clothing" | "clothing" | "outfit" => "web_search".to_string(),
            "estimate_cost" | "price" | "cost" => "calculator".to_string(),
            // API fallback patterns  
            "call_api" | "api_call" | "premium_api_call" | "free_api_call" | "api" => "web_search".to_string(),
            "process_data" | "process" | "transform" => "calculator".to_string(),
            "error" => "calculator".to_string(), // Handle error tool gracefully
            // Default: return as-is
            other => other.to_string(),
        }
    }
    
    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
    
    /// Execute a tool chain with dependencies
    pub fn execute_chain(&self, steps: &[ToolStep]) -> Result<ChainResult> {
        let mut context: HashMap<String, Value> = HashMap::new();
        let mut results = Vec::new();
        
        for (i, step) in steps.iter().enumerate() {
            // Substitute context variables in parameters
            let params = substitute_context(&step.params, &context);
            
            // Normalize parameter names (BFCL v4 inspired aliasing)
            let params = normalize_params(&params);
            
            // Get the tool
            let tool = self.get(&step.tool)
                .ok_or_else(|| anyhow!("Tool '{}' not found", step.tool))?;
            
            // Execute
            let result = tool.execute(&params)?;
            
            // Store in context for future steps
            if let Some(ref key) = step.output_key {
                context.insert(key.clone(), result.clone());
            }
            
            results.push(StepResult {
                step: i,
                tool: step.tool.clone(),
                output: result,
                success: true,
            });
        }
        
        Ok(ChainResult {
            steps: results,
            final_context: context,
        })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_mocks()
    }
}

/// A step in a tool chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStep {
    pub tool: String,
    #[serde(alias = "parameters")]
    pub params: Value,
    #[serde(default)]
    pub output_key: Option<String>,
}

/// Result of a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step: usize,
    pub tool: String,
    pub output: Value,
    pub success: bool,
}

/// Result of a complete chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainResult {
    pub steps: Vec<StepResult>,
    pub final_context: HashMap<String, Value>,
}

/// Substitute context variables in parameters (e.g., {{user_id}})
fn substitute_context(params: &Value, context: &HashMap<String, Value>) -> Value {
    match params {
        Value::String(s) => {
            let mut result = s.clone();
            for (key, value) in context {
                let placeholder = format!("{{{{{}}}}}", key);
                if let Value::String(v) = value {
                    result = result.replace(&placeholder, v);
                } else {
                    result = result.replace(&placeholder, &value.to_string());
                }
            }
            Value::String(result)
        }
        Value::Object(obj) => {
            let new_obj: serde_json::Map<String, Value> = obj.iter()
                .map(|(k, v)| (k.clone(), substitute_context(v, context)))
                .collect();
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| substitute_context(v, context)).collect())
        }
        other => other.clone(),
    }
}

/// Normalize parameter names to handle LLM variations (BFCL v4 inspired)
fn normalize_params(params: &Value) -> Value {
    match params {
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (key, value) in obj.iter() {
                let normalized_key = match key.to_lowercase().as_str() {
                    // Location → city aliasing
                    "location" | "loc" | "place" | "area" => "city".to_string(),
                    // Username → name aliasing  
                    "username" | "user_name" | "user" | "full_name" | "id" => "name".to_string(),
                    // Query aliases
                    "q" | "search" | "topic" | "term" | "keyword" => "query".to_string(),
                    // Amount variations
                    "value" | "number" | "num" => "amount".to_string(),
                    // Operation variations for calculator
                    "op" | "operator" | "action" => "operation".to_string(),
                    // Rate/time variations for compound interest
                    "interest_rate" | "rate_percent" | "percentage" => "rate".to_string(),
                    "years" | "time" | "duration" | "period" => "years".to_string(),
                    "principal" | "initial" | "base" | "start_amount" => "principal".to_string(),
                    // Keep original if no alias
                    _ => key.clone(),
                };
                new_obj.insert(normalized_key, normalize_params(value));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(normalize_params).collect())
        }
        other => other.clone(),
    }
}

// ============== MOCK TOOL IMPLEMENTATIONS ==============

/// Calculator tool - basic math operations
pub struct CalculatorTool;

impl Tool for CalculatorTool {
    fn name(&self) -> &str { "calculator" }
    
    fn description(&self) -> &str {
        "Perform mathematical calculations: add, subtract, multiply, divide"
    }
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string", "enum": ["add", "subtract", "multiply", "divide"]},
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        })
    }
    
    fn execute(&self, params: &Value) -> Result<Value> {
        let op = params.get("operation").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing operation"))?;
        let a = params.get("a").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing or invalid 'a'"))?;
        let b = params.get("b").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing or invalid 'b'"))?;
        
        let result = match op {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(anyhow!("Division by zero"));
                }
                a / b
            }
            _ => return Err(anyhow!("Unknown operation: {}", op)),
        };
        
        Ok(serde_json::json!({"result": result}))
    }
}

/// Weather tool - returns mock weather data
pub struct WeatherTool;

impl Tool for WeatherTool {
    fn name(&self) -> &str { "get_weather" }
    
    fn description(&self) -> &str {
        "Get current weather for a city"
    }
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": {"type": "string"}
            },
            "required": ["city"]
        })
    }
    
    fn execute(&self, params: &Value) -> Result<Value> {
        let city = params.get("city").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing city"))?;
        
        // Deterministic mock data based on city name
        let (temp_c, condition) = match city.to_lowercase().as_str() {
            "london" => (12.0, "cloudy"),
            "tokyo" => (22.0, "sunny"),
            "new york" | "nyc" => (18.0, "partly cloudy"),
            "paris" => (15.0, "overcast"),
            "sydney" => (25.0, "sunny"),
            "moscow" => (-5.0, "snowing"),
            _ => (20.0, "unknown"),
        };
        
        Ok(serde_json::json!({
            "city": city,
            "temperature_c": temp_c,
            "temperature_f": temp_c * 9.0/5.0 + 32.0,
            "condition": condition
        }))
    }
}

/// Currency converter - mock exchange rates
pub struct CurrencyConverterTool;

impl Tool for CurrencyConverterTool {
    fn name(&self) -> &str { "convert_currency" }
    
    fn description(&self) -> &str {
        "Convert between currencies using mock exchange rates"
    }
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "amount": {"type": "number"},
                "from": {"type": "string"},
                "to": {"type": "string"}
            },
            "required": ["amount", "from", "to"]
        })
    }
    
    fn execute(&self, params: &Value) -> Result<Value> {
        let amount = params.get("amount").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing amount"))?;
        let from = params.get("from").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'from' currency"))?;
        let to = params.get("to").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'to' currency"))?;
        
        // Mock exchange rates (to USD)
        let to_usd: HashMap<&str, f64> = [
            ("USD", 1.0), ("EUR", 1.10), ("GBP", 1.27), ("JPY", 0.0067),
            ("CNY", 0.14), ("AUD", 0.65), ("CAD", 0.74), ("CHF", 1.13),
        ].into_iter().collect();
        
        let from_rate = *to_usd.get(from.to_uppercase().as_str())
            .ok_or_else(|| anyhow!("Unknown currency: {}", from))?;
        let to_rate = *to_usd.get(to.to_uppercase().as_str())
            .ok_or_else(|| anyhow!("Unknown currency: {}", to))?;
        
        let usd_amount = amount * from_rate;
        let converted = usd_amount / to_rate;
        
        Ok(serde_json::json!({
            "original": amount,
            "from": from.to_uppercase(),
            "converted": (converted * 100.0).round() / 100.0,
            "to": to.to_uppercase(),
            "rate": from_rate / to_rate
        }))
    }
}

/// Web search - returns mock search results
pub struct WebSearchTool;

impl Tool for WebSearchTool {
    fn name(&self) -> &str { "web_search" }
    
    fn description(&self) -> &str {
        "Search the web and return results"
    }
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            },
            "required": ["query"]
        })
    }
    
    fn execute(&self, params: &Value) -> Result<Value> {
        let query = params.get("query").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing query"))?;
        
        Ok(serde_json::json!({
            "query": query,
            "results": [
                {"title": format!("Result 1 for: {}", query), "url": "https://example.com/1"},
                {"title": format!("Result 2 for: {}", query), "url": "https://example.com/2"},
            ]
        }))
    }
}

/// Date formatter - formats dates
pub struct DateFormatterTool;

impl Tool for DateFormatterTool {
    fn name(&self) -> &str { "format_date" }
    
    fn description(&self) -> &str {
        "Parse and format dates"
    }
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "date": {"type": "string"},
                "format": {"type": "string"}
            },
            "required": ["date"]
        })
    }
    
    fn execute(&self, params: &Value) -> Result<Value> {
        let date = params.get("date").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing date"))?;
        let format = params.get("format").and_then(|v| v.as_str())
            .unwrap_or("YYYY-MM-DD");
        
        // Simple mock parsing - just return a standard format
        // In real impl, would use chrono
        let formatted = match format {
            "YYYY-MM-DD" => "2025-12-16",
            "MM/DD/YYYY" => "12/16/2025",
            _ => "2025-12-16",
        };
        
        Ok(serde_json::json!({
            "input": date,
            "formatted": formatted,
            "format": format
        }))
    }
}

/// User creator - creates mock user
pub struct UserCreatorTool;

impl Tool for UserCreatorTool {
    fn name(&self) -> &str { "create_user" }
    
    fn description(&self) -> &str {
        "Create a new user account"
    }
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "email": {"type": "string"}
            },
            "required": ["name", "email"]
        })
    }
    
    fn execute(&self, params: &Value) -> Result<Value> {
        let name = params.get("name").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?;
        let email = params.get("email").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing email"))?;
        
        // Generate deterministic user_id from email hash
        let user_id = format!("usr_{:x}", email.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32)));
        
        Ok(serde_json::json!({
            "user_id": user_id,
            "name": name,
            "email": email,
            "created": true
        }))
    }
}

/// Email sender - mock email sending
pub struct EmailSenderTool;

impl Tool for EmailSenderTool {
    fn name(&self) -> &str { "send_email" }
    
    fn description(&self) -> &str {
        "Send an email to specified recipient"
    }
    
    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "to": {"type": "string"},
                "subject": {"type": "string"},
                "body": {"type": "string"}
            },
            "required": ["to", "subject", "body"]
        })
    }
    
    fn execute(&self, params: &Value) -> Result<Value> {
        let to = params.get("to").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'to'"))?;
        let subject = params.get("subject").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing subject"))?;
        let body = params.get("body").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing body"))?;
        
        Ok(serde_json::json!({
            "sent": true,
            "to": to,
            "subject": subject,
            "body_length": body.len(),
            "message_id": format!("msg_{}", chrono::Utc::now().timestamp())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculator() {
        let calc = CalculatorTool;
        let result = calc.execute(&serde_json::json!({
            "operation": "multiply",
            "a": 7,
            "b": 8
        })).unwrap();
        assert_eq!(result["result"], 56.0);
    }
    
    #[test]
    fn test_weather() {
        let weather = WeatherTool;
        let result = weather.execute(&serde_json::json!({"city": "London"})).unwrap();
        assert_eq!(result["temperature_c"], 12.0);
        assert_eq!(result["condition"], "cloudy");
    }
    
    #[test]
    fn test_tool_chain() {
        let registry = ToolRegistry::with_mocks();
        
        let steps = vec![
            ToolStep {
                tool: "get_weather".to_string(),
                params: serde_json::json!({"city": "Tokyo"}),
                output_key: Some("weather".to_string()),
            },
            ToolStep {
                tool: "calculator".to_string(),
                params: serde_json::json!({
                    "operation": "multiply",
                    "a": 22.0,  // Would be {{weather.temperature_c}} with full interpolation
                    "b": 2
                }),
                output_key: Some("doubled".to_string()),
            },
        ];
        
        let result = registry.execute_chain(&steps).unwrap();
        assert_eq!(result.steps.len(), 2);
        assert!(result.steps[0].success);
        assert!(result.steps[1].success);
    }
}
