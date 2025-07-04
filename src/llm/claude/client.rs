use crate::{
    llm::AnthropicError,
    llm::{options::CallOptions, LLMError, LLMOutput, LLM},
    schemas::{IntoWithUsage, Message, MessageType, StreamData, TokenUsage, WithUsage},
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::Value;
use std::{collections::HashMap, pin::Pin};

use super::models::{ApiResponse, ClaudeMessage, Payload};

pub enum ClaudeModel {
    Claude3pus20240229,
    Claude3sonnet20240229,
    Claude3haiku20240307,
    Claude3_5sonnet20240620,
}

use std::fmt;

impl fmt::Display for ClaudeModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaudeModel::Claude3pus20240229 => write!(f, "claude-3-opus-20240229"),
            ClaudeModel::Claude3sonnet20240229 => write!(f, "claude-3-sonnet-20240229"),
            ClaudeModel::Claude3haiku20240307 => write!(f, "claude-3-haiku-20240307"),
            ClaudeModel::Claude3_5sonnet20240620 => write!(f, "claude-3-5-sonnet-20240620"),
        }
    }
}

#[derive(Clone)]
pub struct Claude {
    model: String,
    options: CallOptions,
    api_key: String,
    anthropic_version: String,
}

impl Default for Claude {
    fn default() -> Self {
        Self::new()
    }
}

impl Claude {
    pub fn new() -> Self {
        Self {
            model: ClaudeModel::Claude3pus20240229.to_string(),
            options: CallOptions::default(),
            api_key: std::env::var("CLAUDE_API_KEY").unwrap_or_default(),
            anthropic_version: "2023-06-01".to_string(),
        }
    }

    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_options(mut self, options: CallOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = api_key.into();
        self
    }

    pub fn with_anthropic_version<S: Into<String>>(mut self, version: S) -> Self {
        self.anthropic_version = version.into();
        self
    }

    async fn generate(&self, messages: Vec<Message>) -> Result<WithUsage<LLMOutput>, LLMError> {
        let client = Client::new();
        let is_stream = self.options.stream_option.is_some();

        let payload = self.build_payload(messages, is_stream);
        let res = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", self.anthropic_version.clone())
            .header("content-type", "application/json; charset=utf-8")
            .json(&payload)
            .send()
            .await?;
        let res = match res.status().as_u16() {
            401 => Err(LLMError::AnthropicError(
                AnthropicError::AuthenticationError("Invalid API Key".to_string()),
            )),
            403 => Err(LLMError::AnthropicError(AnthropicError::PermissionError(
                "Permission Denied".to_string(),
            ))),
            404 => Err(LLMError::AnthropicError(AnthropicError::NotFoundError(
                "Not Found".to_string(),
            ))),
            429 => Err(LLMError::AnthropicError(AnthropicError::RateLimitError(
                "Rate Limit Exceeded".to_string(),
            ))),
            503 => Err(LLMError::AnthropicError(AnthropicError::OverloadedError(
                "Service Unavailable".to_string(),
            ))),
            _ => Ok(res.json::<ApiResponse>().await?),
        }?;

        let generation = res
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let usage = Some(TokenUsage {
            prompt_tokens: res.usage.input_tokens,
            completion_tokens: res.usage.output_tokens,
            total_tokens: res.usage.input_tokens + res.usage.output_tokens,
        });

        Ok(LLMOutput::Text(generation).with_usage(usage))
    }

    fn build_payload(&self, messages: Vec<Message>, stream: bool) -> Payload {
        let (system_message, other_messages): (Vec<_>, Vec<_>) = messages
            .iter()
            .partition(|m| m.message_type == MessageType::System);
        let mut payload = Payload {
            model: self.model.clone(),
            system: system_message.first().map(|m| m.content.clone()),
            messages: other_messages
                .into_iter()
                .map(ClaudeMessage::from_message)
                .collect::<Vec<_>>(),
            max_tokens: self.options.max_tokens.unwrap_or(1024),
            stream: None,
            stop_sequences: self.options.stop_words.clone(),
            temperature: self.options.temperature,
            top_p: self.options.top_p,
            top_k: self.options.top_k,
        };
        if stream {
            payload.stream = Some(true);
        }
        payload
    }
}

#[async_trait]
impl LLM for Claude {
    async fn generate(&self, messages: Vec<Message>) -> Result<WithUsage<LLMOutput>, LLMError> {
        match &self.options.stream_option {
            Some(stream_option) => {
                let mut complete_response = String::new();
                let mut usage = None;
                let mut stream = self.stream(messages).await?;
                while let Some(data) = stream.next().await {
                    let data = data?;
                    usage = TokenUsage::merge_options([&usage, &data.tokens]);
                    complete_response.push_str(&data.content);

                    if let Some(streaming_func) = &stream_option.streaming_func {
                        let mut func = streaming_func.lock().await;
                        let _ = func(&data.content).await;
                    }
                }

                Ok(LLMOutput::Text(complete_response).with_usage(usage))
            }
            None => self.generate(messages).await,
        }
    }

    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, LLMError>> + Send>>, LLMError> {
        let client = Client::new();
        let payload = self.build_payload(messages, true);
        let request = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.anthropic_version)
            .header("content-type", "application/json; charset=utf-8")
            .json(&payload)
            .build()?;

        // Instead of sending the request directly, return a stream wrapper
        let stream = client.execute(request).await?;
        let stream = stream.bytes_stream();
        // Process each chunk as it arrives
        let processed_stream = stream.then(move |result| {
            async move {
                match result {
                    Ok(bytes) => {
                        let value: Value = parse_sse_to_json(&String::from_utf8_lossy(&bytes))?;
                        if value["type"].as_str().unwrap_or("") == "content_block_delta" {
                            let content = value["delta"]["text"].clone();
                            // Return StreamData based on the parsed content
                            // TODO get tokens from the response
                            Ok(StreamData::new(value, None, content.as_str().unwrap_or("")))
                        } else {
                            Ok(StreamData::new(value, None, ""))
                        }
                    }
                    Err(e) => Err(LLMError::RequestError(e)),
                }
            }
        });

        Ok(Box::pin(processed_stream))
    }

    fn add_call_options(&mut self, call_options: CallOptions) {
        self.options.merge_options(call_options)
    }
}

fn parse_sse_to_json(sse_data: &str) -> Result<Value, LLMError> {
    if let Ok(json) = serde_json::from_str::<Value>(sse_data) {
        return parse_error(&json);
    }

    let lines: Vec<&str> = sse_data.trim().split('\n').collect();
    let mut event_data: HashMap<&str, String> = HashMap::new();

    for line in lines {
        if let Some((key, value)) = line.split_once(": ") {
            event_data.insert(key, value.to_string());
        }
    }

    if let Some(data) = event_data.get("data") {
        let data: Value = serde_json::from_str(data)?;
        return match data["type"].as_str() {
            Some("error") => parse_error(&data),
            _ => Ok(data),
        };
    }
    log::error!("No data field in the SSE event");
    Err(LLMError::ContentNotFound("data".to_string()))
}

fn parse_error(json: &Value) -> Result<Value, LLMError> {
    let error_type = json["error"]["type"].as_str().unwrap_or("");
    let message = json["error"]["message"].as_str().unwrap_or("").to_string();
    match error_type {
        "invalid_request_error" => Err(AnthropicError::InvalidRequestError(message))?,
        "authentication_error" => Err(AnthropicError::AuthenticationError(message))?,
        "permission_error" => Err(AnthropicError::PermissionError(message))?,
        "not_found_error" => Err(AnthropicError::NotFoundError(message))?,
        "rate_limit_error" => Err(AnthropicError::RateLimitError(message))?,
        "api_error" => Err(AnthropicError::ApiError(message))?,
        "overloaded_error" => Err(AnthropicError::OverloadedError(message))?,
        _ => Err(LLMError::OtherError("Unknown error".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    #[ignore]
    async fn test_cloudia_generate() {
        let cloudia = Claude::new();

        let res = cloudia
            .generate(vec![Message::new_human_message("Hi, how are you doing")])
            .await
            .unwrap();

        println!("{res:?}")
    }

    #[test]
    #[ignore]
    async fn test_cloudia_stream() {
        let cloudia = Claude::new();
        let mut stream = cloudia
            .stream(vec![Message::new_human_message("Hi, how are you doing")])
            .await
            .unwrap();
        while let Some(data) = stream.next().await {
            match data {
                Ok(value) => value.to_stdout().unwrap(),
                Err(e) => panic!("Error invoking LLMChain: {e:?}"),
            }
        }
    }
}
