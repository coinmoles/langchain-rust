use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionStreamOptions, ChatCompletionTool,
    ChatCompletionToolChoiceOption, ResponseFormat,
};
use serde::Serialize;

use crate::{
    llm::{options::CallOptions, LLMError},
    schemas::Message,
};

/// Request payload sent to an OpenAPI-compatible API.
#[derive(Serialize, Debug)]
pub struct OpenAIRequest {
    pub messages: Vec<ChatCompletionRequestMessage>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ChatCompletionStreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ChatCompletionTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ChatCompletionToolChoiceOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

impl OpenAIRequest {
    /// Creates a new [`OpenAIRequest`].
    pub fn new(
        model: impl Into<String>,
        messages: Vec<Message>,
    ) -> Result<OpenAIRequest, LLMError> {
        let messages = messages
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(OpenAIRequest {
            messages,
            model: model.into(),
            stream: None,
            stream_options: None,
            candidate_count: None,
            max_tokens: None,
            temperature: None,
            stop: None,
            top_k: None,
            top_p: None,
            seed: None,
            min_length: None,
            max_length: None,
            n: None,
            repetition_penalty: None,
            frequency_penalty: None,
            presence_penalty: None,
            tools: None,
            tool_choice: None,
            response_format: None,
        })
    }

    /// Adds options to the request.
    pub fn with_options(self, options: CallOptions) -> Self {
        OpenAIRequest {
            stream: Some(options.stream_option.is_some()),
            stream_options: options.stream_option.as_ref().map(|stream| {
                ChatCompletionStreamOptions {
                    include_usage: stream.include_usage,
                }
            }),
            candidate_count: options.candidate_count,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stop: options.stop_words,
            top_k: options.top_k,
            top_p: options.top_p,
            seed: options.seed,
            min_length: options.min_length,
            max_length: options.max_length,
            n: options.n,
            repetition_penalty: options.repetition_penalty,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            tools: options.tools,
            tool_choice: options.tool_choice,
            response_format: options.response_format,
            ..self
        }
    }
}
