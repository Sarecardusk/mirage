use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use serde_json::json;
use tauri::ipc::Channel;

use crate::domain::llm::{LlmConfig, LlmStreamEvent};
use crate::domain::session::Message;

fn role_name(role: &crate::domain::session::ChatRole) -> &'static str {
    match role {
        crate::domain::session::ChatRole::User => "user",
        crate::domain::session::ChatRole::Assistant => "assistant",
        crate::domain::session::ChatRole::System => "system",
    }
}

fn completion_url(endpoint: &str) -> String {
    if endpoint.ends_with("/chat/completions") {
        endpoint.to_string()
    } else {
        format!("{}/chat/completions", endpoint.trim_end_matches('/'))
    }
}

pub async fn stream_chat_completion(
    config: &LlmConfig,
    theme_card_name: &str,
    system_prompt: &str,
    history: &[Message],
    channel: &Channel<LlmStreamEvent>,
) -> Result<String> {
    let full_system_prompt = format!("[角色名称: {theme_card_name}]\n\n{system_prompt}");
    let mut request_messages = vec![json!({
        "role": "system",
        "content": full_system_prompt,
    })];
    for message in history {
        request_messages.push(json!({
            "role": role_name(&message.role),
            "content": message.content,
        }));
    }

    let client = reqwest::Client::new();
    let response = client
        .post(completion_url(config.endpoint.as_str()))
        .bearer_auth(config.api_key.as_str())
        .json(&json!({
            "model": config.model,
            "stream": true,
            "messages": request_messages,
        }))
        .send()
        .await
        .context("failed to call upstream llm api")?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unable to read upstream response body".to_string());
        return Err(anyhow!("upstream error status {status}: {body}"));
    }

    let mut full_text = String::new();
    let mut buffer = String::new();
    let mut byte_stream = response.bytes_stream();

    while let Some(chunk_result) = byte_stream.next().await {
        let chunk = chunk_result.context("failed to read stream chunk")?;
        buffer.push_str(String::from_utf8_lossy(&chunk).as_ref());

        while let Some(newline_index) = buffer.find('\n') {
            let line = buffer[..newline_index].trim().to_string();
            buffer = buffer[(newline_index + 1)..].to_string();

            if !line.starts_with("data: ") {
                continue;
            }

            let payload = line.trim_start_matches("data: ").trim();
            if payload == "[DONE]" {
                continue;
            }

            let parsed: serde_json::Value = match serde_json::from_str(payload) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let maybe_token = parsed["choices"][0]["delta"]["content"].as_str();
            if let Some(token) = maybe_token {
                full_text.push_str(token);
                channel
                    .send(LlmStreamEvent::TokenChunk {
                        text: token.to_string(),
                    })
                    .map_err(|error| anyhow!(error.to_string()))?;
            }
        }
    }

    Ok(full_text)
}
