use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use serde_json::json;
use tauri::ipc::Channel;

use crate::domain::llm::{ListLlmModelsInput, LlmConfig, LlmStreamEvent, TestLlmConnectionInput};
use crate::domain::session::Message;

fn normalized_endpoint_root(endpoint: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    for suffix in ["/chat/completions", "/models"] {
        if let Some(base) = trimmed.strip_suffix(suffix) {
            return base.trim_end_matches('/').to_string();
        }
    }
    trimmed.to_string()
}

fn completion_url(endpoint: &str) -> String {
    format!("{}/chat/completions", normalized_endpoint_root(endpoint))
}

fn models_url(endpoint: &str) -> String {
    format!("{}/models", normalized_endpoint_root(endpoint))
}

/// GET {endpoint}/models → 返回按 id 排好序的模型名列表。
/// 空列表不视为错误，由调用方决定如何处理。
pub async fn list_models(input: &ListLlmModelsInput) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let response = client
        .get(models_url(input.endpoint.as_str()))
        .bearer_auth(input.api_key.as_str())
        .send()
        .await
        .context("failed to call upstream models api")?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("upstream error status {status}: {body}"));
    }

    let payload: serde_json::Value = response
        .json()
        .await
        .context("failed to parse models response")?;
    let mut models: Vec<String> = payload["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item["id"].as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    models.sort();
    models.dedup();

    Ok(models)
}

/// POST {endpoint}/chat/completions with max_tokens=1 — 只验证 HTTP 状态，不解析响应体。
pub async fn test_connection(input: &TestLlmConnectionInput) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(completion_url(input.endpoint.as_str()))
        .bearer_auth(input.api_key.as_str())
        .json(&json!({
            "model": input.model,
            "stream": false,
            "max_tokens": 1,
            "messages": [{ "role": "user", "content": "ping" }],
        }))
        .send()
        .await
        .context("failed to call upstream llm api")?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("upstream error status {status}: {body}"));
    }

    Ok(())
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
            "role": message.role.as_str(),
            "content": message.content,
        }));
    }

    let mut body = json!({
        "model": config.model,
        "stream": true,
        "messages": request_messages,
    });
    // 仅在用户明确设置时才注入生成参数，避免向上游发送 null 值
    if let Some(t) = config.temperature {
        body["temperature"] = json!(t);
    }
    if let Some(mt) = config.max_tokens {
        body["max_tokens"] = json!(mt);
    }
    if let Some(tp) = config.top_p {
        body["top_p"] = json!(tp);
    }
    if let Some(fp) = config.frequency_penalty {
        body["frequency_penalty"] = json!(fp);
    }
    if let Some(pp) = config.presence_penalty {
        body["presence_penalty"] = json!(pp);
    }

    let client = reqwest::Client::new();
    let response = client
        .post(completion_url(config.endpoint.as_str()))
        .bearer_auth(config.api_key.as_str())
        .json(&body)
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

#[cfg(test)]
mod tests {
    use super::{completion_url, models_url};

    #[test]
    fn derives_completion_url_from_base_or_completion_endpoint() {
        assert_eq!(
            completion_url("https://api.deepseek.com"),
            "https://api.deepseek.com/chat/completions"
        );
        assert_eq!(
            completion_url("https://api.deepseek.com/v1/"),
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(
            completion_url("https://api.deepseek.com/v1/chat/completions"),
            "https://api.deepseek.com/v1/chat/completions"
        );
    }

    #[test]
    fn derives_models_url_from_base_or_completion_endpoint() {
        assert_eq!(
            models_url("https://api.deepseek.com"),
            "https://api.deepseek.com/models"
        );
        assert_eq!(
            models_url("https://api.deepseek.com/v1/models"),
            "https://api.deepseek.com/v1/models"
        );
        assert_eq!(
            models_url("https://api.deepseek.com/v1/chat/completions"),
            "https://api.deepseek.com/v1/models"
        );
    }
}
