use leptos::prelude::*;

#[derive(Clone)]
struct ChatMessage {
    role: &'static str,
    text: String,
}

#[component]
pub fn App() -> impl IntoView {
    let messages = RwSignal::new(vec![
        ChatMessage { role: "assistant", text: "Welcome back. I’ve got a fresh set of questionable jokes and remarkably solid answers. What are we making today?".into() },
    ]);
    let draft = RwSignal::new(String::new());
    let sending = RwSignal::new(false);
    let send = move |_| {
        let prompt = draft.get().trim().to_string();
        if prompt.is_empty() || sending.get() {
            return;
        }
        messages.update(|items| {
            items.push(ChatMessage {
                role: "user",
                text: prompt.clone(),
            })
        });
        draft.set(String::new());
        sending.set(true);
        // This gives immediate feedback while the SSE client is connected to the API.
        messages.update(|items| {
            items.push(ChatMessage {
                role: "assistant pending",
                text: "Thinking of a response that is at least 60% helpful…".into(),
            })
        });
        sending.set(false);
    };
    view! {
        <div class="shell">
          <aside class="sidebar">
            <div class="brand"><span class="brand-mark">{"✦"}</span><span>quip</span></div>
            <button class="new-chat"><span>{"＋"}</span> New conversation <kbd>{"⌘ K"}</kbd></button>
            <div class="nav-label">Today</div>
            <nav class="threads">
              <a class="active" href="#"><span>{"✦"}</span> A very serious brainstorm</a>
              <a href="#"><span>{"◌"}</span> Better error messages</a>
              <a href="#"><span>{"◇"}</span> Weekend dinner ideas</a>
            </nav>
            <div class="nav-label">Yesterday</div>
            <nav class="threads"><a href="#"><span>{"◒"}</span> API naming review</a><a href="#"><span>{"□"}</span> CSS animation notes</a></nav>
            <div class="sidebar-footer"><button class="profile"><span class="avatar">AM</span><span><b>Alex Morgan</b><small>Personal workspace</small></span><span>{"···"}</span></button></div>
          </aside>
          <section class="chat">
            <header><div><p class="eyebrow">PERSONAL WORKSPACE</p><h1>A very serious brainstorm</h1></div><div class="header-actions"><button class="icon-button" aria-label="Share">{"↗"}</button><button class="icon-button" aria-label="More">{"···"}</button></div></header>
            <div class="conversation">
              <div class="date"><span></span> TODAY <span></span></div>
              <For each=move || messages.get() key=|message| message.text.clone() let:message>
                <article class=format!("message {}", message.role)>
                  <div class="message-avatar">{if message.role.starts_with("assistant") { "✦" } else { "AM" }}</div>
                  <div class="bubble"><p>{message.text}</p>{if message.role == "assistant" { view! { <div class="message-actions"><button>{"↻"}</button><button>{"⧉"}</button><button>{"♡"}</button></div> }.into_any() } else { ().into_any() }}</div>
                </article>
              </For>
              <Show when=move || sending.get()><div class="typing"><i></i><i></i><i></i></div></Show>
            </div>
            <div class="composer-wrap">
              <div class="composer">
                <textarea prop:value=move || draft.get() on:input=move |event| draft.set(event_target_value(&event)) on:keydown=move |event| { if event.key() == "Enter" && !event.shift_key() { event.prevent_default(); send(()); } } placeholder="Message Quip…" rows="1"></textarea>
                <div class="composer-footer"><button class="attach" aria-label="Attach a file">{"＋"}</button><button class="model"><span class="model-dot"></span> GPT-4.1 mini <span>{"⌄"}</span></button><span class="spacer"></span><span class="shortcut">{"⌘ ↵"}</span><button class="send" on:click=send aria-label="Send message">{"↑"}</button></div>
              </div>
              <p class="disclaimer">Quip can make mistakes. Check important info.</p>
            </div>
          </section>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    mount_to_body(App);
}
