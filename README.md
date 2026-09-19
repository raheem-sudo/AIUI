# Quip / AIUI

AIUI is a calm, chat-first workspace for interacting with AI applications built with the Rust ecosystem. It includes a **Leptos 0.8** client with a shadcn-inspired, Tailwind-like visual language and a Rust API that owns provider access through **Rig (`rig.rs`)**.

## Quick start

1. Install the Rust toolchain and the `wasm32-unknown-unknown` target.
2. Install [Trunk](https://trunkrs.dev/) for the Leptos client.
3. Copy `.env.example` to `.env` and set `OPENAI_API_KEY` in your shell.
4. In one terminal, start the API:

   ```bash
   OPENAI_API_KEY="..." cargo run -p aiui-api
   ```

5. In another terminal, serve the client:

   ```bash
   trunk serve --cwd crates/web --port 8080
   ```

The API accepts a new conversation at `POST /api/conversations` and returns server-sent events from `POST /api/conversations/{id}/messages`. Keys never enter the browser bundle.

The goal is to provide a clean, responsive interface where users can send prompts, receive streamed responses, and later manage conversations, agents, and model settings without coupling the browser directly to an AI provider.

## Project Idea

AIUI will be a chat-oriented AI workspace with:

- A Leptos 0.8 frontend for conversations and interaction state.
- A Rust HTTP API that owns authentication, validation, provider configuration, and Rig agent execution.
- Streaming assistant responses so the UI updates as tokens arrive.
- A provider-neutral backend boundary, allowing models supported by Rig to be changed without rewriting the UI.
- A foundation for future features such as conversation history, system prompts, tool use, file context, and usage information.

## Suggested Architecture

```text
Browser
	|
	| HTTP / SSE or WebSocket
	v
Leptos 0.8 UI
	|
	| JSON API requests
	v
Rust API
	|- request validation and error handling
	|- conversation/session persistence
	|- streaming response coordination
	v
Rig agent and model provider
```

The browser should communicate only with the application API. The API should keep provider credentials and Rig configuration on the server, which makes the frontend simpler and avoids exposing secrets.

## Suggested Implementation

### 1. Create the Rust workspace

The repository now uses a Cargo workspace with separate crates for the frontend and backend:

```text
aiui/
├── Cargo.toml
├── crates/
│   ├── web/       # Leptos 0.8 application and CSS design system
│   ├── api/       # Axum routes and Rig agent integration
│   └── shared/    # Request/response types shared by web and API
└── README.md
```

The exact server framework can follow the chosen deployment setup, but an async Rust HTTP framework such as Axum is a natural fit alongside Leptos and Rig.

### 2. Build the Leptos UI

Start with a focused chat workflow:

1. Render a conversation list and message history.
2. Add a prompt input with submit, cancel, and retry actions.
3. Send typed requests to the API rather than calling Rig from the browser.
4. Read streamed assistant output and append it to the active message.
5. Represent loading, empty, error, and disconnected states explicitly.

Keep UI models independent from Rig's internal types. The frontend only needs stable application types such as `Conversation`, `Message`, and `ChatRequest`.

### 3. Add the API boundary

An initial API can expose:

```text
POST /api/conversations
GET  /api/conversations/:id
POST /api/conversations/:id/messages
GET  /api/conversations/:id/messages/stream
```

For a first version, `POST /api/conversations/:id/messages` may return a complete response. Add Server-Sent Events (SSE) or WebSockets once the basic request path works; streaming is important for the final experience but should not obscure the initial integration.

Example request:

```json
{
	"content": "Summarize this document.",
	"model": "configured-server-side"
}
```

The server should validate the request, load the conversation context, invoke a Rig agent, and return an application-level response. Provider names, API keys, and model credentials should come from server configuration or a secret manager.

### 4. Integrate Rig

Encapsulate Rig behind an application service, for example `AiService`, rather than calling it directly from route handlers. This keeps routing, persistence, and AI execution testable in isolation.

The service should be responsible for:

- Building or selecting the Rig agent.
- Supplying the conversation history and system instructions.
- Selecting the configured model and provider.
- Mapping Rig output and failures into application-level events or errors.
- Exposing a streaming interface when the selected model supports it.

### 5. Add persistence and operational concerns

After the in-memory prototype works, add a database for conversations and messages. Also add:

- Configuration through environment variables.
- Request timeouts and cancellation.
- Structured logging and request IDs.
- Authentication and authorization before production use.
- Rate limiting and provider error handling.
- Tests for API validation, service behavior, and key UI states.

## Development Phases

1. **Prototype:** Render a Leptos chat screen and connect it to one API endpoint with a mock response.
2. **Rig integration:** Replace the mock with a server-side Rig agent and one configured model provider.
3. **Conversation state:** Persist conversations and messages, then restore them in the UI.
4. **Streaming:** Add SSE or WebSockets and render partial assistant responses.
5. **Production hardening:** Add authentication, observability, limits, deployment configuration, and automated tests.

## Design Principles

- Keep AI provider details inside the backend.
- Use shared, versioned DTOs for the web/API contract.
- Prefer explicit application errors over leaking provider-specific failures.
- Keep the first vertical slice small: prompt, request, response, and rendered message.
- Treat streaming, cancellation, and reconnect behavior as first-class UI states.

## Initial Success Criteria

The first milestone is complete when a user can open the Leptos application, submit a prompt, receive a response generated through a Rig agent, and see useful loading and error states without exposing provider credentials to the browser.
