// commands/ai.rs — IPC commands for AI Translation Assistant

use tauri::State;
use crate::state::AppState;
use crate::core::ai::{query_ai, query_document_ai, AiConfig, AiRequest, AiResponse, DocumentAiRequest};
use crate::core::secrets;

/// OpenAI-compatible requests pull their key from the OS keychain.
fn resolve_openai_key(config: &mut AiConfig) {
    if config.provider == "openai" {
        config.api_key = secrets::resolve(config.api_key.take(), "openai");
    }
}

#[tauri::command]
pub async fn query_ai_assistant(
    mut request: AiRequest,
    _state: State<'_, AppState>,
) -> Result<AiResponse, String> {
    resolve_openai_key(&mut request.config);
    query_ai(request).await
}

#[tauri::command]
pub async fn query_document_ai_copilot(
    mut request: DocumentAiRequest,
    _state: State<'_, AppState>,
) -> Result<AiResponse, String> {
    resolve_openai_key(&mut request.config);
    query_document_ai(request).await
}
