// Egui - Routes

use std::convert::Infallible;

use async_stream::try_stream;
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        Html, Sse,
    },
    Json,
};
use futures_core::Stream;
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
pub struct InputMessage {
    msg: String,
}

pub async fn create_message(
    State(app_state): State<AppState>,
    Json(input_message): Json<InputMessage>,
) -> Result<StatusCode, String> {
    let _ = app_state.tx.broadcast(input_message.msg).await;
    Ok(StatusCode::OK)
}

pub async fn message(
    State(app_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = app_state.tx.new_receiver();

    Sse::new(try_stream! {
        while let Ok(msg) = rx.recv().await {
            let ev = Event::default()
                .data(msg);
            yield ev;
        }
    })
    .keep_alive(KeepAlive::default())
}

pub async fn home() -> Html<String> {
    let template = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>SSE Chat</title>
            </head>
            <body>
                <h1>SSE Chat</h1>
                <div id="chat">
                    <p><em>Connected.</em></p>
                </div>
                <form id="msg-form">
                    <label for="msg">Message:</label>
                    <input type="text" id="msg" name="msg" required>
                    <button type="submit">Send</button>
                </form>
                <script>
                document.getElementById('msg-form').onsubmit = async function(event) {{
                    event.preventDefault();
                    const msg = document.getElementById('msg').value;
                    const response = await fetch('/create', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ msg }})
                    }});
                    document.getElementById('msg').value='';
                    const data = await response.json();
                    console.log('data', data);
                    }};
                    const eventSource = new EventSource('/sse');
                        eventSource.onmessage = function(event) {{
                            const para = document.createElement("p");
                            const node = document.createTextNode(event.data);
                            para.appendChild(node);

                            const element = document.getElementById("chat");
                            element.appendChild(para);
                    }};
                </script>
            </body>
        </html>
    "#,
    );
    Html(template)
}
