//! Stable application DTOs shared by the browser and HTTP API.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateConversationResponse {
    pub id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StreamEvent {
    pub kind: StreamEventKind,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEventKind {
    Delta,
    Done,
    Error,
}
