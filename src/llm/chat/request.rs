use std::borrow::Cow;
use std::collections::HashMap;

use async_openai::types::{
    ChatCompletionStreamOptions, ChatCompletionTool, ChatCompletionToolChoiceOption,
    ReasoningEffort, ResponseFormat,
};
use serde::Serialize;

use crate::llm::options::LLMOptions;

/// Request payload sent to an OpenAPI-compatible API.
#[derive(Serialize, Debug)]
pub struct ChatRequest<'a, M> {
    /// A list of messages comprising the conversation so far.
    ///
    /// Generic to accept both
    /// [`ChatCompletionRequestMessage`](async_openai::types::ChatCompletionRequestMessage) and
    /// [`ChatHistory`](crate::llm::ChatHistory)
    ///
    /// See [`messages`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-messages).
    pub messages: M,

    /// Model ID used to generate the response, like `gpt-4o` or `o3`.
    ///
    /// See [`model`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-model).
    pub model: String,

    /// If set to true, the model response data will be streamed to the client as it is generated
    /// using server-sent events.
    ///
    /// See [`stream`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-stream).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Options for streaming response. When `stream` is set to true, the option is automatically
    /// configured to `{ "include_usage": true }`
    ///
    /// See [`stream_options`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-stream_options).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ChatCompletionStreamOptions>,

    /// A list of tools the model may call. You can provide either custom tools or function tools.
    ///
    /// See [`tools`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-tools)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Cow<'a, [ChatCompletionTool]>>,

    /// Controls which (if any) tool is called by the model.
    ///
    /// See [`tool_choice`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-tool_choice).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ChatCompletionToolChoiceOption>,

    /// Whether to enable parallel function calling during tool use.
    ///
    /// See [`parallel_tool_calls`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-parallel_tool_calls).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// How many chat completion choices to generate for each input message.
    ///
    /// See [`n`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-n).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,

    /// Constrains effort on reasoning for reasoning models.
    ///
    /// Only supported for certain OpenAI models.
    ///
    /// See [`reasoning_effort`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-reasoning_effort).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
    /// output more random, while lower values like 0.2 will make it more focused and
    /// deterministic.
    ///
    /// See [`temperature`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-temperature)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the model
    /// considers the results of the tokens with top_p probability mass. Lower values will make the
    /// output more deterministic.
    ///
    /// See [`top_p`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-top_p).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// The parameter for the top-k sampling method. The model considers the top `top_k` tokens
    /// with the highest probability. Lower values will make the output more deterministic.
    ///
    /// Not part of the OpenAI chat completions API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing
    /// frequency in the text so far, decreasing the model's likelihood to repeat the same line
    /// verbatim.
    ///
    /// See [`frequency_penalty`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-frequency_penalty).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they
    /// appear in the text so far, increasing the model's likelihood to talk about new topics.
    ///
    /// See [`presence_penalty`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-presence_penalty).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Number between 0.0 and 2.0. Positive values discourage the model from repeating the same
    /// line verbatim.
    ///
    /// Not part of the OpenAI chat completions API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,

    /// The maximum number of tokens that can be generated in the chat completion.
    ///
    /// Deprecated in the OpenAI API, but still used by some other providers.
    ///
    /// See [`max_tokens`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-max_tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// An upper bound for the number of tokens that can be generated for a completion, including
    /// visible output tokens and reasoning tokens.
    ///
    /// See [`max_completion_tokens`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-max_completion_tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// Up to 4 sequences where the API will stop generating further tokens. The returned text will
    /// not contain the stop sequence.
    ///
    /// See [`stop`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-stop).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// An object specifying the format that the model must output.
    ///
    /// See [`response_format`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-response_format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Extra parameters for custom api.
    #[serde(flatten)]
    pub extra_params: HashMap<String, serde_json::Value>,
}

impl<'a, M> ChatRequest<'a, M> {
    /// Constructs a new [`ChatRequest`].
    pub fn new(
        model: impl Into<String>,
        messages: M,
        function_specs: Option<Cow<'a, [ChatCompletionTool]>>,
    ) -> Self {
        ChatRequest {
            messages,
            model: model.into(),
            stream: None,
            stream_options: None,
            tools: function_specs,
            tool_choice: None,
            parallel_tool_calls: None,
            n: None,
            reasoning_effort: None,
            temperature: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            max_tokens: None,
            max_completion_tokens: None,
            stop: None,
            response_format: None,
            extra_params: HashMap::new(),
        }
    }

    /// Applies the given [`LLMOptions`] to the request.
    pub fn with_options(self, options: LLMOptions) -> Self {
        let stream_options =
            options
                .stream
                .unwrap_or_default()
                .then_some(ChatCompletionStreamOptions {
                    include_usage: true,
                });

        // `max_tokens` is ignored if `max_completion_tokens` is set.
        let max_tokens = match options.max_completion_tokens {
            Some(_) => None,
            None => options.max_tokens,
        };

        ChatRequest {
            tool_choice: options.tool_choice.map(Into::into),
            parallel_tool_calls: options.parallel_tool_calls,
            stream: options.stream,
            stream_options,
            n: options.n,
            reasoning_effort: options.reasoning_effort,
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: options.top_k,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            repetition_penalty: options.repetition_penalty,
            max_tokens,
            max_completion_tokens: options.max_completion_tokens,
            stop: options.stop_words,
            response_format: options.response_format.map(Into::into),
            extra_params: options.extra_params.unwrap_or(self.extra_params),
            ..self
        }
    }
}
