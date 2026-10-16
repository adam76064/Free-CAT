// core/ai.rs — AI Translation Assistant module
// Integrates with local Ollama (100% offline) or OpenAI-compatible endpoints.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,        // "ollama" | "openai"
    pub endpoint: Option<String>,// e.g. "http://localhost:11434" or "https://api.openai.com/v1"
    pub model:    Option<String>,// e.g. "llama3.2", "qwen2.5:7b", "gpt-4o-mini"
    pub api_key:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub prompt:      String,
    pub source_text: Option<String>,
    pub target_text: Option<String>,
    pub src_lang:    Option<String>,
    pub tgt_lang:    Option<String>,
    pub action:      String, // "ask" | "rephrase" | "critique"
    pub config:      AiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub reply: String,
    pub model: String,
}

/// Execute an AI query via Ollama or OpenAI-compatible API
pub async fn query_ai(req: AiRequest) -> Result<AiResponse, String> {
    let system_prompt = "You are Free-CAT's built-in professional translation assistant and computational linguist. \
Provide accurate, concise, and helpful responses to aid the translator. \
When providing translation suggestions, ensure natural phrasing and terminology precision.";

    let user_message = match req.action.as_str() {
        "rephrase" => {
            format!(
                "Source sentence ({}): \"{}\"\nCurrent translation ({}): \"{}\"\n\n\
Please provide 3 alternative natural rephrasings for the target translation. Number them 1, 2, 3 and keep explanations brief.",
                req.src_lang.as_deref().unwrap_or("source"),
                req.source_text.as_deref().unwrap_or(""),
                req.tgt_lang.as_deref().unwrap_or("target"),
                req.target_text.as_deref().unwrap_or("")
            )
        }
        "critique" => {
            format!(
                "Source sentence ({}): \"{}\"\nTranslation ({}): \"{}\"\n\n\
Please evaluate this translation: check for grammar, spelling, fluency, and tone. Point out any errors or unnatural expressions, and suggest improvements.",
                req.src_lang.as_deref().unwrap_or("source"),
                req.source_text.as_deref().unwrap_or(""),
                req.tgt_lang.as_deref().unwrap_or("target"),
                req.target_text.as_deref().unwrap_or("")
            )
        }
        _ => {
            let mut prompt_parts = Vec::new();
            if let Some(src) = &req.source_text {
                prompt_parts.push(format!("Source context: \"{}\"", src));
            }
            if let Some(tgt) = &req.target_text {
                prompt_parts.push(format!("Target context: \"{}\"", tgt));
            }
            prompt_parts.push(req.prompt.clone());
            prompt_parts.join("\n\n")
        }
    };

    match req.config.provider.as_str() {
        "openai" => call_openai(&req.config, system_prompt, &user_message).await,
        _ => call_ollama(&req.config, system_prompt, &user_message).await,
    }
}

async fn call_ollama(config: &AiConfig, system: &str, user: &str) -> Result<AiResponse, String> {
    let base_url = config.endpoint.as_deref().unwrap_or("http://localhost:11434");
    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let model = config.model.as_deref().unwrap_or("llama3.2");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "stream": false
    });

    let resp = client.post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama at {}: {}. Ensure Ollama is running.", url, e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama error {}: check model '{}' is installed.", resp.status(), model));
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    let reply = json["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(AiResponse {
        reply,
        model: model.to_string(),
    })
}

async fn call_openai(config: &AiConfig, system: &str, user: &str) -> Result<AiResponse, String> {
    let base_url = config.endpoint.as_deref().unwrap_or("https://api.openai.com/v1");
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let model = config.model.as_deref().unwrap_or("gpt-4o-mini");
    let api_key = config.api_key.as_deref().ok_or("OpenAI API key is required")?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "temperature": 0.3
    });

    let resp = client.post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("AI API returned status {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("Failed to parse AI response: {}", e))?;

    let reply = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(AiResponse {
        reply,
        model: model.to_string(),
    })
}
