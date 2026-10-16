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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSegment {
    pub seq_number: i64,
    pub source_text: String,
    pub target_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAiRequest {
    pub current_source: String,
    pub current_target: Option<String>,
    pub preceding_segments: Vec<ContextSegment>,
    pub following_segments: Vec<ContextSegment>,
    pub termbase_matches: Vec<(String, String)>,
    pub style_guide: Option<String>,
    pub action: String, // "translate", "rephrase", "condense", "tone_formal", "tone_casual", "fix_grammar"
    pub selected_text: Option<String>,
    pub src_lang: String,
    pub tgt_lang: String,
    pub config: AiConfig,
}

/// Context-aware document agent query supporting 5-segment sliding window, glossary injection, and style guide
pub async fn query_document_ai(req: DocumentAiRequest) -> Result<AiResponse, String> {
    let system_prompt = "You are Free-CAT's elite computational linguist and professional translator. \
You maintain perfect discourse cohesion, correct cross-sentence pronouns and references, enforce the mandatory project termbase glossary, \
and produce natural, native-sounding translations. Output only the requested translation or refinement without conversational filler.";

    let mut parts = Vec::new();

    // 1. Preceding Discourse Context
    if !req.preceding_segments.is_empty() {
        parts.push("=== PRECEDING DISCOURSE CONTEXT ===".to_string());
        for s in &req.preceding_segments {
            parts.push(format!(
                "#{}: [{}] \"{}\" -> [{}] \"{}\"",
                s.seq_number,
                req.src_lang,
                s.source_text,
                req.tgt_lang,
                s.target_text.as_deref().unwrap_or("[untranslated]")
            ));
        }
        parts.push("".to_string());
    }

    // 2. Following Discourse Preview
    if !req.following_segments.is_empty() {
        parts.push("=== UPCOMING DISCOURSE PREVIEW ===".to_string());
        for s in &req.following_segments {
            parts.push(format!("#{}: [{}] \"{}\"", s.seq_number, req.src_lang, s.source_text));
        }
        parts.push("".to_string());
    }

    // 3. Mandatory Project Terminology
    if !req.termbase_matches.is_empty() {
        parts.push("=== MANDATORY PROJECT GLOSSARY ===".to_string());
        for (src_t, tgt_t) in &req.termbase_matches {
            parts.push(format!("* \"{}\" => \"{}\"", src_t, tgt_t));
        }
        parts.push("".to_string());
    }

    // 4. Style Guide Guidelines
    if let Some(sg) = &req.style_guide {
        if !sg.trim().is_empty() {
            parts.push(format!("=== STYLE GUIDE GUIDELINES ===\n{}\n", sg));
        }
    }

    // 5. Current Segment
    parts.push("=== CURRENT TARGET SEGMENT ===".to_string());
    parts.push(format!("Source ({}): \"{}\"", req.src_lang, req.current_source));
    if let Some(tgt) = &req.current_target {
        if !tgt.trim().is_empty() {
            parts.push(format!("Current Target ({}): \"{}\"", req.tgt_lang, tgt));
        }
    }

    let instruction = match req.action.as_str() {
        "rephrase" => {
            if let Some(sel) = &req.selected_text {
                format!("Provide 3 polished, natural alternative translations for the selected text: \"{}\". Number them 1., 2., 3.", sel)
            } else {
                "Provide 3 polished, natural alternative translations for this target segment. Number them 1., 2., 3.".to_string()
            }
        }
        "condense" => {
            if let Some(sel) = &req.selected_text {
                format!("Condense and shorten the selected text \"{}\" to be succinct while retaining full meaning.", sel)
            } else {
                "Condense and shorten the translation to be succinct while preserving full meaning.".to_string()
            }
        }
        "tone_formal" => {
            if let Some(sel) = &req.selected_text {
                format!("Rewrite the selected text \"{}\" in an elevated, formal, authoritative professional register.", sel)
            } else {
                "Rewrite the translation in an elevated, formal, authoritative professional register.".to_string()
            }
        }
        "tone_casual" => {
            if let Some(sel) = &req.selected_text {
                format!("Rewrite the selected text \"{}\" in a natural, friendly, accessible contemporary tone.", sel)
            } else {
                "Rewrite the translation in a natural, friendly, accessible contemporary tone.".to_string()
            }
        }
        "fix_grammar" => {
            if let Some(sel) = &req.selected_text {
                format!("Correct any grammatical, typographical, or stylistic errors in the selected text: \"{}\". Return only the corrected version.", sel)
            } else {
                "Correct any grammatical, typographical, agreement, or punctuation errors in this translation. Return only the corrected translation.".to_string()
            }
        }
        _ => {
            format!("Translate the source sentence accurately into fluent, natural {}. Output ONLY the direct translation without any explanation or quotes.", req.tgt_lang)
        }
    };

    parts.push(format!("\n=== TASK ===\n{}", instruction));
    let user_message = parts.join("\n");

    match req.config.provider.as_str() {
        "openai" => call_openai(&req.config, system_prompt, &user_message).await,
        _ => call_ollama(&req.config, system_prompt, &user_message).await,
    }
}
