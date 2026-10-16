// commands/ai.rs — IPC commands for AI Translation Assistant

use tauri::State;
use crate::state::AppState;
use crate::core::ai::{query_ai, query_document_ai, AiRequest, AiResponse, DocumentAiRequest};

#[tauri::command]
pub async fn query_ai_assistant(
    request: AiRequest,
    _state: State<'_, AppState>,
) -> Result<AiResponse, String> {
    query_ai(request).await
}

#[tauri::command]
pub async fn query_document_ai_copilot(
    request: DocumentAiRequest,
    _state: State<'_, AppState>,
) -> Result<AiResponse, String> {
    query_document_ai(request).await
}
