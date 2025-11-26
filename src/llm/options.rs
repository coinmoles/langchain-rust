use std::collections::HashMap;

use crate::schemas::{ReasoningEffort, ResponseFormat, ToolChoice};

/// Options for LLM calls.
#[derive(Clone, Debug)]
pub struct LLMOptions {
    /// If set to true, the model response data will be streamed to the client as it is generated
    /// using server-sent events.
    ///
    /// Corresponds to [`ChatCompletionRequest::stream`](crate::llm::ChatRequest::stream) or
    /// [`ResponsesRequest::stream`](crate::llm::ResponsesRequest::stream).
    pub stream: Option<bool>,

    /// Controls which (if any) tool is called by the model.
    ///
    /// Corresponds to [`ChatCompletionRequest::tool_choice`](crate::llm::ChatRequest::tool_choice)
    /// or [`ResponsesRequest::tool_choice`](crate::llm::ResponsesRequest::tool_choice).
    pub tool_choice: Option<ToolChoice>,

    /// Whether to enable parallel function calling during tool use.
    ///
    /// Corresponds to
    /// [`ChatCompletionRequest::parallel_tool_calls`](crate::llm::ChatRequest::parallel_tool_calls) or
    /// [`ResponsesRequest::parallel_tool_calls`](crate::llm::ResponsesRequest::parallel_tool_calls).
    pub parallel_tool_calls: Option<bool>,

    /// How many chat completion choices to generate for each input message.
    ///
    /// Corresponds to [`ChatCompletionRequest::n`](crate::llm::ChatRequest::n).
    ///
    /// Not compatible with the responses api.
    pub n: Option<u8>,

    /// Constrains effort on reasoning for reasoning models.
    ///
    /// Corresponds to
    /// [`ChatCompletionRequest::reasoning_effort`](crate::llm::ChatRequest::reasoning_effort) or
    /// [`ResponsesRequest::reasoning_effort`](crate::llm::ResponsesRequest::reasoning_effort).
    ///
    /// Only supported for certain OpenAI models.
    pub reasoning_effort: Option<ReasoningEffort>,

    /// The sampling temperature. Higher values like 0.8 will make the output more random, while
    /// lower values like 0.2 will make it more focused and deterministic.
    ///
    /// Corresponds to [`ChatCompletionRequest::temperature`](crate::llm::ChatRequest::temperature)
    /// or [`ResponsesRequest::temperature`](crate::llm::ResponsesRequest::temperature).
    pub temperature: Option<f32>,

    /// The parameter for the top-p sampling method. The model considers the results of the tokens
    /// with `top_p` probability mass. Lower values will make the output more deterministic.
    ///
    /// Corresponds to [`ChatCompletionRequest::top_p`](crate::llm::ChatRequest::top_p) or
    /// [`ResponsesRequest::top_p`](crate::llm::ResponsesRequest::top_p).
    pub top_p: Option<f32>,

    /// The parameter for the top-k sampling method. The model considers the top `top_k` tokens
    /// with the highest probability. Lower values will make the output more deterministic.
    ///
    /// Corresponds to [`ChatCompletionRequest::top_k`](crate::llm::ChatRequest::top_k) or
    /// [`ResponsesRequest::top_k`](crate::llm::ResponsesRequest::top_k).
    ///
    /// Not part of the OpenAI chat completions API or the responses API, but used by some other
    /// providers.
    pub top_k: Option<u32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing
    /// frequency in the text so far, decreasing the model's likelihood to repeat the same line
    /// verbatim.
    ///
    /// Corresponds to
    /// [`ChatCompletionRequest::frequency_penalty`](crate::llm::ChatRequest::frequency_penalty) or
    /// [`ResponsesRequest::frequency_penalty`](crate::llm::ResponsesRequest::frequency_penalty).
    pub frequency_penalty: Option<f32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they
    /// appear in the text so far, increasing the model's likelihood to talk about new topics.
    ///
    /// Corresponds to
    /// [`ChatCompletionRequest::presence_penalty`](crate::llm::ChatRequest::presence_penalty) or
    /// [`ResponsesRequest::presence_penalty`](crate::llm::ResponsesRequest::presence_penalty).
    pub presence_penalty: Option<f32>,

    /// Number between 0.0 and 2.0. Positive values discourage the model from repeating the same
    /// line verbatim.
    ///
    /// Corresponds to
    /// [`ChatCompletionRequest::repetition_penalty`](crate::llm::ChatRequest::repetition_penalty)
    /// or
    /// [`ResponsesRequest::repetition_penalty`](crate::llm::ResponsesRequest::repetition_penalty).
    ///
    /// Not part of the OpenAI chat completions API or the responses API, but used by some other
    /// providers.
    pub repetition_penalty: Option<f32>,

    /// The maximum number of tokens to generate in the completion.
    ///
    /// OpenAI API deprecated `max_tokens` in favor of `max_completion_tokens`, but other providers
    /// still use `max_tokens`.
    ///
    /// This value is ignored if [`max_completion_tokens`](Self::max_completion_tokens) is set.
    ///
    /// Corresponds to [`ChatCompletionRequest::max_tokens`](crate::llm::ChatRequest::max_tokens)
    /// or [`ResponsesRequest::max_output_tokens`](crate::llm::ResponsesRequest::max_output_tokens).
    pub max_tokens: Option<u32>,

    /// The maximum number of tokens to generate in the completion.
    ///
    /// This field takes precedence over [`max_tokens`](Self::max_tokens).
    ///
    /// Corresponds to
    /// [`ChatCompletionRequest::max_completion_tokens`](crate::llm::ChatRequest::max_completion_tokens)
    /// or [`ResponsesRequest::max_output_tokens`](crate::llm::ResponsesRequest::max_output_tokens).
    pub max_completion_tokens: Option<u32>,

    /// Up to 4 sequences where the API will stop generating further tokens. The returned text will
    /// not contain the stop sequence.
    ///
    /// Corresponds to [`ChatCompletionRequest::stop`](crate::llm::ChatRequest::stop).
    ///
    /// Not compatible with the responses api.
    pub stop_words: Option<Vec<String>>,

    /// An object specifying the format that the model must output.
    ///
    /// Corresponds to
    /// [`ChatCompletionRequest::response_format`](crate::llm::ChatRequest::response_format) or
    /// the [`format`](async_openai::types::responses::TextConfig::format) field of
    /// [`ResponsesRequest::text`](crate::llm::ResponsesRequest::text).
    pub response_format: Option<ResponseFormat>,

    /// Extra parameters for custom api.
    pub extra_params: Option<HashMap<String, serde_json::Value>>,

    /// Whether to convert system message into assistant message.
    ///
    /// Some LLMs do not support system messages. Set this field to `true` for those models.
    pub system_is_assistant: Option<bool>,

    /// Whether to drop the "thought" part of the response.
    ///
    /// The "thought" part is the part of the response that contains the model's reasoning
    /// process. By default, this is dropped in subsequent requests to save tokens. Setting this to
    /// `false` will keep the thought in the conversation history.
    pub drop_thought: Option<bool>,
}

impl Default for LLMOptions {
    fn default() -> Self {
        LLMOptions::new()
    }
}

impl LLMOptions {
    /// Constructs a new [`LLMOptions`].
    pub fn new() -> Self {
        LLMOptions {
            stream: None,
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
            stop_words: None,
            response_format: None,
            system_is_assistant: None,
            drop_thought: None,
            extra_params: None,
        }
    }

    /// Sets the [`stream`](Self::stream).
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets the [`tool_choice`](Self::tool_choice).
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the [`parallel_tool_calls`](Self::parallel_tool_calls).
    pub fn with_parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = Some(parallel_tool_calls);
        self
    }

    /// Sets the [`n`](Self::n).
    ///
    /// Not compatible with the responses api.
    pub fn with_n(mut self, n: u8) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets the [`reasoning_effort`](Self::reasoning_effort).
    ///
    /// Only supported for certain OpenAI models.
    pub fn with_reasoning_effort(mut self, reasoning_effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(reasoning_effort);
        self
    }

    /// Sets the [`max_tokens`](Self::max_tokens).
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the [`max_completion_tokens`](Self::max_completion_tokens).
    pub fn with_max_completion_tokens(mut self, max_completion_tokens: u32) -> Self {
        self.max_completion_tokens = Some(max_completion_tokens);
        self
    }

    /// Sets the [`top_p`](Self::top_p).
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Sets the [`top_k`](Self::top_k).
    ///
    /// Not part of the OpenAI chat completions API or the responses API, but used by some other
    /// providers.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Sets the [`frequency_penalty`](Self::frequency_penalty).
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Sets the [`presence_penalty`](Self::presence_penalty).
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    /// Sets the [`repetition_penalty`](Self::repetition_penalty).
    ///
    /// Not part of the OpenAI chat completions API or the responses API, but used by some other
    /// providers.
    pub fn with_repetition_penalty(mut self, repetition_penalty: f32) -> Self {
        self.repetition_penalty = Some(repetition_penalty);
        self
    }

    /// Sets the [`temperature`](Self::temperature).
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the [`stop_words`](Self::stop_words).
    ///
    /// Not compatible with the responses api.
    pub fn with_stop_words(mut self, stop_words: Vec<String>) -> Self {
        self.stop_words = Some(stop_words);
        self
    }

    /// Sets the [`response_format`](Self::response_format).
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Sets the [`system_is_assistant`](Self::system_is_assistant).
    pub fn with_system_is_assistant(mut self, system_is_assistant: bool) -> Self {
        self.system_is_assistant = Some(system_is_assistant);
        self
    }

    /// Sets the [`drop_thought`](Self::drop_thought).
    pub fn with_drop_thought(mut self, drop_thought: bool) -> Self {
        self.drop_thought = Some(drop_thought);
        self
    }

    /// Sets the [`extra_params`](Self::extra_params).
    pub fn with_extra_params(mut self, extra_params: HashMap<String, serde_json::Value>) -> Self {
        self.extra_params = Some(extra_params);
        self
    }

    /// Merges two [`LLMOptions`] into one.
    pub fn merge(mut self, other: Self) -> Self {
        self.merge_inplace(other);
        self
    }

    /// Merges another [`LLMOptions`] into this one.
    ///
    /// For each field, if the incoming option is `Some`, it will replace the existing value.
    /// Otherwise, the existing value is retained.
    pub fn merge_inplace(&mut self, other: LLMOptions) {
        // For simple scalar types wrapped in Option, prefer incoming option if it is Some
        self.max_tokens = other.max_tokens.or(self.max_tokens);
        self.temperature = other.temperature.or(self.temperature);
        self.top_k = other.top_k.or(self.top_k);
        self.top_p = other.top_p.or(self.top_p);
        self.n = other.n.or(self.n);
        self.frequency_penalty = other.frequency_penalty.or(self.frequency_penalty);
        self.presence_penalty = other.presence_penalty.or(self.presence_penalty);
        self.tool_choice = other.tool_choice.or(self.tool_choice.clone());
        self.response_format = other.response_format.or(self.response_format.clone());

        // For `Vec<String>`, merge if both are Some; prefer incoming if only incoming is Some
        if let Some(mut new_stop_words) = other.stop_words {
            if let Some(existing_stop_words) = &mut self.stop_words {
                existing_stop_words.append(&mut new_stop_words);
            } else {
                self.stop_words = Some(new_stop_words);
            }
        }

        self.system_is_assistant = other.system_is_assistant.or(self.system_is_assistant);
        self.drop_thought = other.drop_thought.and(self.drop_thought);
    }
}
