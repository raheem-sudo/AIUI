//! HTTP boundary for the chat workspace. Provider keys stay on this process.

use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};

use aiui_shared::{CreateConversationResponse, SendMessageRequest, StreamEvent, StreamEventKind};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    routing::post,
};
use futures::{Stream, StreamExt, stream};
use rig::{
    completion::Message,
    prelude::*,
    providers::openai::{self, OpenAI},
};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

const PREAMBLE: &str = "You are a comedian here to entertain the user using humour and jokes. Be helpful, warm, and concise.";

#[derive(Clone, Default)]
struct AppState {
    conversations: Arc<RwLock<HashMap<Uuid, Vec<Message>>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app = Router::new()
        .route("/api/conversations", post(create_conversation))
        .route("/api/conversations/{id}/messages", post(stream_message))
        .with_state(AppState::default())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
                .allow_methods([axum::http::Method::POST]),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("API listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn create_conversation(State(state): State<AppState>) -> Json<CreateConversationResponse> {
    let id = Uuid::new_v4();
    state.conversations.write().await.insert(id, Vec::new());
    Json(CreateConversationResponse { id })
}

async fn stream_message(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<SendMessageRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let content = request.content.trim().to_owned();
    if content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Message cannot be empty.".into()));
    }
    if content.len() > 12_000 {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Message must be 12,000 characters or fewer.".into(),
        ));
    }

    let history = {
        let conversations = state.conversations.read().await;
        conversations
            .get(&id)
            .cloned()
            .ok_or((StatusCode::NOT_FOUND, "Conversation not found.".into()))?
    };
    state
        .conversations
        .write()
        .await
        .get_mut(&id)
        .expect("conversation checked above")
        .push(Message::user(&content));

    let result = stream_with_rig(history, content).await;
    let events = match result {
        Ok(events) => events,
        Err(message) => vec![StreamEvent {
            kind: StreamEventKind::Error,
            content: message,
        }],
    };
    Ok(Sse::new(stream::iter(events).map(to_sse_event)).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

/// Mirrors Rig's streamed-history example while returning application events rather than provider types.
async fn stream_with_rig(
    history: Vec<Message>,
    prompt: String,
) -> Result<Vec<StreamEvent>, String> {
    let client = OpenAI::from_env()
        .map_err(|error| format!("OpenAI is not configured: {error}"))?
        .bound()
        .map_err(|error| error.to_string())?;
    let agent = client.agent(openai::GPT_4).preamble(PREAMBLE).build();
    let mut stream = agent.prompt(&prompt).history(&history).stream();
    let mut response = String::new();
    while let Some(item) = stream.next().await {
        let item = item.map_err(|error| error.to_string())?;
        if let MultiTurnStreamItem::FinalResponse(final_response) = item {
            response = final_response.output().to_owned();
        }
    }
    if response.is_empty() {
        return Err("The model finished without a response.".into());
    }
    Ok(response
        .chars()
        .collect::<Vec<_>>()
        .chunks(14)
        .map(|chunk| StreamEvent {
            kind: StreamEventKind::Delta,
            content: chunk.iter().collect(),
        })
        .chain(std::iter::once(StreamEvent {
            kind: StreamEventKind::Done,
            content: String::new(),
        }))
        .collect())
}

fn to_sse_event(event: StreamEvent) -> Result<Event, Infallible> {
    Ok(Event::default()
        .event(match event.kind {
            StreamEventKind::Delta => "delta",
            StreamEventKind::Done => "done",
            StreamEventKind::Error => "error",
        })
        .json_data(event)
        .unwrap_or_else(|_| Event::default().event("error").data("serialization failed")))
}
