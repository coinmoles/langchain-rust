use async_openai::types::responses::{
    Input, InputItem, ReasoningConfig, ReasoningSummary, TextConfig, ToolChoice, ToolDefinition,
};
use serde::Serialize;

use crate::llm::LLMError;
use crate::llm::options::LLMOptions;
use crate::schemas::{Message, ToolSpec};

/// Request payload sent to an OpenAPI-compatible API.
#[derive(Serialize, Debug)]
pub struct ResponsesRequest {
    // TODO: support background mode
    /// Text, image, or file inputs to the model, used to generate a response.
    ///
    /// See [`input`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-input)
    pub input: Input,

    /// Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range
    /// of models with different capabilities, performance characteristics, and price points.
    /// Refer to the model guide to browse and compare available models.
    ///
    /// See [`model`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-model)
    pub model: String,

    /// If set to true, the model response data will be streamed to the client as it is generated
    /// using server-sent events. See the Streaming section below for more information.
    ///
    /// See [`stream`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-stream)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// An array of tools the model may call while generating a response. You can specify which
    /// tool to use by setting the `tool_choice` parameter.
    ///
    /// See [`tools`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-tools)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    /// How the model should select which tool (or tools) to use when generating a response. See
    /// the `tools` parameter to see how to specify which tools the model can call.
    ///
    /// See [`tool_choice`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-tool_choice)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Whether to allow the model to run tool calls in parallel.
    ///
    /// See [`parallel_tool_calls`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-parallel_tool_calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// The maximum number of total calls to built-in tools that can be processed in a response.
    /// This maximum number applies across all built-in tool calls, not per individual tool. Any
    /// further attempts to call a tool by the model will be ignored.
    ///
    /// See [`max_tool_calls`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-max_tool_calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
    /// output more random, while lower values like 0.2 will make it more focused and
    /// deterministic.
    ///
    /// See [`temperature`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-temperature)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the model
    /// considers the results of the tokens with top_p probability mass. Lower values will make the
    /// output more deterministic.
    ///
    /// See [`top_p`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-top_p)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// The parameter for the top-k sampling method. The model considers the top `top_k` tokens
    /// with the highest probability. Lower values will make the output more deterministic.
    ///
    /// Not part of the OpenAI responses API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<usize>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing
    /// frequency in the text so far, decreasing the model's likelihood to repeat the same line
    /// verbatim.
    ///
    /// Not part of the OpenAI responses API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they
    /// appear in the text so far, increasing the model's likelihood to talk about new topics.
    ///
    /// Not part of the OpenAI responses API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Number between 0.0 and 2.0. Positive values discourage the model from repeating the same
    /// line verbatim.
    ///
    /// Not part of the OpenAI responses API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,

    /// Configuration options for reasoning models.
    ///
    /// Only supported for certain OpenAI models.
    ///
    /// See [`reasoning`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-reasoning)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,

    /// An upper bound for the number of tokens that can be generated for a response, including
    /// visible output tokens and reasoning tokens.
    ///
    /// See [`max_output_tokens`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-max_output_tokens)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,

    /// Up to 4 sequences where the API will stop generating further tokens. The returned text will
    /// not contain the stop sequence.
    ///
    /// Not part of the OpenAI responses API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Configuration options for a text response from the model.
    ///
    /// See [`text`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
}

impl ResponsesRequest {
    /// Creates a new [`OpenAIRequest`].
    pub fn new(
        model: impl Into<String>,
        messages: Vec<Message>,
        tools: Option<ToolSpec>,
    ) -> Result<ResponsesRequest, LLMError> {
        let msgs = messages
            .into_iter()
            .flat_map(Vec::<InputItem>::from)
            .collect();
        let input = Input::Items(msgs);
        let tools = tools.map(ToolSpec::into_tool_definitions);

        Ok(ResponsesRequest {
            input,
            model: model.into(),
            stream: None,
            tools,
            tool_choice: None,
            max_tool_calls: None,
            parallel_tool_calls: None,
            temperature: None,
            top_p: None,
            top_k: None,
            repetition_penalty: None,
            frequency_penalty: None,
            presence_penalty: None,
            reasoning: None,
            max_output_tokens: None,
            stop: None,
            text: None,
        })
    }

    /// Adds options to the request.
    pub fn with_options(self, options: LLMOptions) -> Self {
        ResponsesRequest {
            stream: options.stream,
            tool_choice: options.tool_choice.map(Into::into),
            reasoning: options.reasoning_effort.map(|re| ReasoningConfig {
                effort: Some(re),
                summary: Some(ReasoningSummary::Auto),
            }),
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: options.top_k,
            repetition_penalty: options.repetition_penalty,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            max_output_tokens: options.max_tokens,
            stop: options.stop_words,
            text: options
                .response_format
                .map(|rf| TextConfig { format: rf.into() }),
            ..self
        }
    }
}
