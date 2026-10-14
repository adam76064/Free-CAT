// commands/mt.rs — Machine Translation API integration

use tauri::State;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct MtArgs {
    pub source_text: String,
    pub src_lang:    String,
    pub tgt_lang:    String,
    pub engine:      String, // "deepl" | "libretranslate" | "google"
    pub api_key:     Option<String>,
    pub api_url:     Option<String>, // for self-hosted LibreTranslate
}

#[derive(serde::Serialize)]
pub struct MtResult {
    pub translated_text: String,
    pub engine:          String,
    pub score_pct:       Option<u8>, // MTQE score (Phase 2)
}

/// Get a machine translation suggestion.
#[tauri::command]
pub async fn get_mt_suggestion(
    args: MtArgs,
    _state: State<'_, AppState>,
) -> Result<MtResult, String> {
    match args.engine.as_str() {
        "deepl" => translate_deepl(args).await,
        "libretranslate" => translate_libre(args).await,
        "google" => translate_google(args).await,
        _ => Err(format!("Unknown MT engine: {}", args.engine)),
    }
}

async fn translate_deepl(args: MtArgs) -> Result<MtResult, String> {
    let api_key = args.api_key.ok_or("DeepL API key required")?;
    let client = reqwest::Client::new();

    // DeepL free API endpoint
    let url = if api_key.ends_with(":fx") {
        "https://api-free.deepl.com/v2/translate"
    } else {
        "https://api.deepl.com/v2/translate"
    };

    let body = serde_json::json!({
        "text": [args.source_text],
        "source_lang": args.src_lang.to_uppercase().split('-').next().unwrap_or("EN"),
        "target_lang": args.tgt_lang.to_uppercase()
    });

    let response = client
        .post(url)
        .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("DeepL request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("DeepL API error: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("DeepL response parse failed: {}", e))?;

    let translated = json["translations"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(MtResult {
        translated_text: translated,
        engine: "deepl".into(),
        score_pct: None,
    })
}

async fn translate_libre(args: MtArgs) -> Result<MtResult, String> {
    let url = args.api_url
        .as_deref()
        .unwrap_or("http://localhost:5000")
        .to_string() + "/translate";

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "q":      args.source_text,
        "source": args.src_lang.split('-').next().unwrap_or("en"),
        "target": args.tgt_lang.split('-').next().unwrap_or("en"),
        "format": "text",
        "api_key": args.api_key.unwrap_or_default()
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("LibreTranslate request failed: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("LibreTranslate parse failed: {}", e))?;

    let translated = json["translatedText"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(MtResult {
        translated_text: translated,
        engine: "libretranslate".into(),
        score_pct: None,
    })
}

async fn translate_google(args: MtArgs) -> Result<MtResult, String> {
    let api_key = args.api_key.ok_or("Google Translate API key required")?;
    let client = reqwest::Client::new();

    let tgt_lang = args.tgt_lang.split('-').next().unwrap_or("en");
    let url = format!(
        "https://translation.googleapis.com/language/translate/v2?key={}",
        api_key
    );

    let body = serde_json::json!({
        "q": args.source_text,
        "target": tgt_lang,
        "format": "text"
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Google Translate request failed: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Google Translate parse failed: {}", e))?;

    let translated = json["data"]["translations"][0]["translatedText"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(MtResult {
        translated_text: translated,
        engine: "google".into(),
        score_pct: None,
    })
}
