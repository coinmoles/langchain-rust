use async_openai::types::responses::{Input, InputItem, TextConfig, ToolChoice, ToolDefinition};
use serde::Serialize;

use crate::llm::LLMError;
use crate::llm::options::CallOptions;
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

    /// Configuration options for a text response from the model.
    ///
    /// See [`text`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,

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

    /// An upper bound for the number of tokens that can be generated for a response, including
    /// visible output tokens and reasoning tokens.
    ///
    /// See [`max_output_tokens`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-max_output_tokens)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,

    /// The maximum number of total calls to built-in tools that can be processed in a response.
    /// This maximum number applies across all built-in tool calls, not per individual tool. Any
    /// further attempts to call a tool by the model will be ignored.
    ///
    /// See [`max_tool_calls`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-max_tool_calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,

    /// Whether to allow the model to run tool calls in parallel.
    ///
    /// See [`parallel_tool_calls`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-parallel_tool_calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// gpt-5 and o-series models only
    ///
    /// Configuration options for reasoning models.
    ///
    /// See [`reasoning`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-reasoning)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
    /// output more random, while lower values like 0.2 will make it more focused and
    /// deterministic. We generally recommend altering this or top_p but not both.
    ///
    /// See [`temperature`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-temperature)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// An integer between 0 and 20 specifying the number of most likely tokens to return at each
    /// token position, each with an associated log probability.
    ///
    /// See [`top_logprobs`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-top_logprobs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the model
    /// considers the results of the tokens with top_p probability mass. So 0.1 means only the
    /// tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    ///
    /// See [`top_p`](https://platform.openai.com/docs/api-reference/responses/create#responses-create-top_p)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<usize>,
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
            text: None,
            tools,
            tool_choice: None,
            max_output_tokens: None,
            max_tool_calls: None,
            parallel_tool_calls: None,
            reasoning: None,
            temperature: None,
            top_logprobs: None,
            top_p: None,
            stop: None,
            top_k: None,
            seed: None,
            min_length: None,
            max_length: None,
            n: None,
            repetition_penalty: None,
            frequency_penalty: None,
            presence_penalty: None,
        })
    }

    /// Adds options to the request.
    pub fn with_options(self, options: CallOptions) -> Self {
        ResponsesRequest {
            stream: options.stream,
            // text: options.response_format, // TODO: reimplement response_format
            // tool_choice: options.tool_choice,
            max_output_tokens: options.max_tokens,
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
            ..self
        }
    }
}
