//! Helper functions for the chat completions api.

use std::collections::HashMap;

use async_openai::Client as OpenAiClient;
use async_openai::config::Config;
use async_openai::error::OpenAIError;
use async_openai::types::{
    ChatChoice, ChatChoiceStream, ChatCompletionMessageToolCall,
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartText, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestSystemMessageContentPart,
    ChatCompletionRequestToolMessage, ChatCompletionRequestToolMessageContent,
    ChatCompletionRequestToolMessageContentPart, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageArgs, ChatCompletionResponseMessage,
    ChatCompletionResponseStream, ChatCompletionToolType, CompletionTokensDetails, CompletionUsage,
    CreateChatCompletionResponse, CreateChatCompletionStreamResponse, FinishReason, FunctionCall,
    PromptTokensDetails, Role,
};
use futures::StreamExt;
use itertools::{Either, Itertools};
use serde::Serialize;

use crate::llm::{ChatRequest, LLMError};
use crate::schemas::{FunctionSpec, LLMStream, LLMStreamChunk, ToolCall, ToolSpec};
use crate::tools::{FunctionTool, McpTool, ToolData, ToolError, ToolOutput};
use crate::utils::helper::{FORCE_FINAL_ANSWER, add_option_numbers};

/// Merges two [`PromptTokensDetails`], summing their fields where both are `Some`.
fn merge_prompt_tokens_details(
    details: Option<PromptTokensDetails>,
    chunk_details: Option<PromptTokensDetails>,
) -> Option<PromptTokensDetails> {
    match (details, chunk_details) {
        (None, None) => None,
        (Some(details), None) => Some(details),
        (None, Some(details)) => Some(details),
        (Some(details), Some(chunk_details)) => Some(PromptTokensDetails {
            audio_tokens: add_option_numbers(details.audio_tokens, chunk_details.audio_tokens),
            cached_tokens: add_option_numbers(details.cached_tokens, chunk_details.cached_tokens),
        }),
    }
}

/// Merges two [`CompletionTokensDetails`], summing their fields where both are `Some`.
fn merge_completion_tokens_details(
    details: Option<CompletionTokensDetails>,
    chunk_details: Option<CompletionTokensDetails>,
) -> Option<CompletionTokensDetails> {
    match (details, chunk_details) {
        (None, None) => None,
        (Some(details), None) => Some(details),
        (None, Some(details)) => Some(details),
        (Some(details), Some(chunk_details)) => Some(CompletionTokensDetails {
            accepted_prediction_tokens: add_option_numbers(
                details.accepted_prediction_tokens,
                chunk_details.accepted_prediction_tokens,
            ),
            audio_tokens: add_option_numbers(details.audio_tokens, chunk_details.audio_tokens),
            reasoning_tokens: add_option_numbers(
                details.reasoning_tokens,
                chunk_details.reasoning_tokens,
            ),
            rejected_prediction_tokens: add_option_numbers(
                details.rejected_prediction_tokens,
                chunk_details.rejected_prediction_tokens,
            ),
        }),
    }
}

/// Merges two [`CompletionUsage`], summing their fields where both are `Some`.
fn merge_usage(
    usage: Option<CompletionUsage>,
    chunk_usage: Option<CompletionUsage>,
) -> Option<CompletionUsage> {
    match (usage, chunk_usage) {
        (None, None) => None,
        (Some(usage), None) => Some(usage),
        (None, Some(usage)) => Some(usage),
        (Some(usage), Some(chunk_usage)) => Some(CompletionUsage {
            prompt_tokens: usage.prompt_tokens + chunk_usage.prompt_tokens,
            completion_tokens: usage.completion_tokens + chunk_usage.completion_tokens,
            total_tokens: usage.total_tokens + chunk_usage.total_tokens,
            prompt_tokens_details: merge_prompt_tokens_details(
                usage.prompt_tokens_details,
                chunk_usage.prompt_tokens_details,
            ),
            completion_tokens_details: merge_completion_tokens_details(
                usage.completion_tokens_details,
                chunk_usage.completion_tokens_details,
            ),
        }),
    }
}

/// Aggregates a [`ChatChoiceStream`] into a [`ChatChoice`].
fn aggregate_choice(choice: &mut ChatChoice, choice_stream: ChatChoiceStream) {
    let delta = choice_stream.delta;

    if let Some(role) = delta.role {
        choice.message.role = role;
    }

    if let Some(content) = delta.content {
        let current = choice.message.content.get_or_insert_with(String::new);
        current.push_str(&content);
    }
    // Tool calls
    if let Some(tool_call_chunks) = delta.tool_calls {
        let calls = choice.message.tool_calls.get_or_insert_with(Vec::new);
        for chunk in tool_call_chunks {
            let idx = chunk.index as usize;
            if calls.len() <= idx {
                calls.resize_with(idx + 1, || ChatCompletionMessageToolCall {
                    id: String::new(),
                    r#type: ChatCompletionToolType::Function,
                    function: FunctionCall {
                        name: String::new(),
                        arguments: String::new(),
                    },
                });
            }

            let call = &mut calls[idx];
            if let Some(id) = chunk.id.clone() {
                call.id = id;
            }
            if let Some(typ) = chunk.r#type {
                call.r#type = typ;
            }
            if let Some(func) = chunk.function {
                if let Some(name) = func.name {
                    call.function.name = name;
                }
                if let Some(args) = func.arguments {
                    call.function.arguments.push_str(&args);
                }
            }
        }
    }

    // `function_call` is deprecated, but still included here for completeness.
    #[allow(deprecated)]
    if let Some(function_call) = delta.function_call {
        let fc = choice.message.function_call.get_or_insert(FunctionCall {
            name: String::new(),
            arguments: String::new(),
        });

        if let Some(name) = function_call.name {
            fc.name = name;
        }
        if let Some(args) = function_call.arguments {
            fc.arguments.push_str(&args);
        }
    }

    if let Some(refusal) = delta.refusal {
        choice.message.refusal = Some(refusal); // Overwrite with refusal
    }

    if let Some(logprobs) = choice_stream.logprobs {
        choice.logprobs = Some(logprobs);
    }

    if let Some(finish_reason) = choice_stream.finish_reason {
        choice.finish_reason = Some(finish_reason);
    }
}

/// Constructs a full [`CreateChatCompletionResponse`] from a stream of
/// [`CreateChatCompletionStreamResponse`].
pub async fn construct_chat_completion_response(
    mut stream: ChatCompletionResponseStream,
) -> Result<CreateChatCompletionResponse, OpenAIError> {
    let mut choices_map: HashMap<u32, ChatChoice> = HashMap::new();
    let mut usage: Option<CompletionUsage> = None;

    let mut id = String::new();
    let mut created = 0;
    let mut model = String::new();
    let mut service_tier = None;
    let mut system_fingerprint = None;
    let mut object = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        // Capture shared metadata
        if id.is_empty() {
            id = chunk.id;
            created = chunk.created;
            model = chunk.model;
            service_tier = chunk.service_tier;
            system_fingerprint = chunk.system_fingerprint;
            object = chunk.object; // always this value
        }

        usage = merge_usage(usage, chunk.usage);

        for choice_stream in chunk.choices {
            let choice = choices_map
                .entry(choice_stream.index)
                .or_insert_with(|| ChatChoice {
                    index: choice_stream.index,
                    // `function_call` is deprecated, but still included here for completeness.
                    #[allow(deprecated)]
                    message: ChatCompletionResponseMessage {
                        role: Role::Assistant,
                        content: None,
                        refusal: None,
                        tool_calls: None,
                        function_call: None,
                        audio: None,
                    },
                    finish_reason: None,
                    logprobs: None,
                });

            aggregate_choice(choice, choice_stream);
        }
    }

    Ok(CreateChatCompletionResponse {
        id,
        created,
        model,
        service_tier,
        system_fingerprint,
        object,
        choices: choices_map.into_values().collect(),
        usage,
    })
}

/// Selects the best [`ChatChoice`] from a list of choices based on finish reason and index.
pub fn select_choice(choices: Vec<ChatChoice>) -> Result<ChatChoice, LLMError> {
    if choices.is_empty() {
        return Err(LLMError::ContentNotFound("No choices.".to_string()));
    }

    let mut choices = choices.clone();
    choices.sort_by(|c1, c2| {
        let rank = |c: &ChatChoice| match c.finish_reason {
            // Stop is the most preferred finish reason.
            Some(FinishReason::Stop) => 0,
            // Tool calls is the next preferred finish reason, as long as the tool calls are
            // actually generated.
            Some(FinishReason::ToolCalls)
                if c.message
                    .tool_calls
                    .as_deref()
                    .is_some_and(|calls| !calls.is_empty()) =>
            {
                1
            }
            Some(FinishReason::ContentFilter) | Some(FinishReason::Length) => 2,
            _ => 3,
        };

        // First sort by rank
        // Then by index to preserve order within ranks
        rank(c1)
            .cmp(&rank(c2))
            .then_with(|| c1.index.cmp(&c2.index))
    });
    let selected_choice = choices
        .into_iter()
        .next()
        .ok_or(LLMError::ContentNotFound("No choices.".to_string()))?;

    Ok(selected_choice)
}

/// Generates a chat completion by choosing between using `create_stream` or `create` based on the
/// `stream` field of the request.
pub async fn generate<C: Config, M: Serialize>(
    client: &OpenAiClient<C>,
    request: ChatRequest<'_, M>,
) -> Result<CreateChatCompletionResponse, OpenAIError> {
    let response = if request.stream.unwrap_or_default() {
        let stream = client
            .chat()
            .create_stream_byot::<_, CreateChatCompletionStreamResponse>(request)
            .await?;
        construct_chat_completion_response(stream).await?
    } else {
        client
            .chat()
            .create_byot::<_, CreateChatCompletionResponse>(request)
            .await?
    };

    Ok(response)
}

/// Maps a [`ChatCompletionResponseStream`] into an `LLMStream`.
pub fn map_stream(original: ChatCompletionResponseStream) -> LLMStream {
    let new = original.map(|result| match result {
        Ok(completion) => {
            if let Some(usage) = completion.usage.clone() {
                let usage = usage.into();
                let completion =
                    serde_json::to_value(completion).map_err(LLMError::ResponseSerdeError)?;
                return Ok(LLMStreamChunk::new(completion, Some(usage), ""));
            }
            if let Some(content) = completion
                .choices
                .first()
                .and_then(|c| c.delta.content.clone())
            {
                let completion =
                    serde_json::to_value(completion).map_err(LLMError::ResponseSerdeError)?;
                return Ok(LLMStreamChunk::new(completion, None, content));
            }
            Err(LLMError::content_not_found("/choices/0/delta/content"))
        }
        Err(e) => Err(LLMError::from(e)),
    });
    Box::pin(new)
}

/// Initialize the mcp tools for a session.
pub async fn resolve_mcp_tools(
    tool_spec: Option<ToolSpec>,
) -> Result<(Vec<FunctionSpec>, HashMap<String, Box<dyn FunctionTool>>), ToolError> {
    let Some(spec) = tool_spec else {
        return Ok((Vec::new(), HashMap::new()));
    };

    let mcp_functions = McpTool::into_function_tools(spec.mcps).await?;
    let all_function_specs = spec
        .functions
        .into_iter()
        .chain(mcp_functions.values().map(|tool| tool.get_spec()))
        .collect::<Vec<_>>();

    Ok((all_function_specs, mcp_functions))
}

/// The result of [`handle_mcp_calls`].
pub struct McpHandleResult {
    /// The results of mcp tool calls.
    pub results: Vec<(String, String, Result<ToolOutput, ToolError>)>,
    /// The tool calls not handled.
    pub remaining_calls: Vec<ToolCall>,
}

/// From the list of [`ToolCall`], extract calls to mcp tools and handle them. The results of the
/// calls are returned along with their id and tool name.
pub async fn handle_mcp_calls(
    calls: Vec<ToolCall>,
    mcp_functions: &HashMap<String, Box<dyn FunctionTool>>,
) -> McpHandleResult {
    let (mcp_calls, remaining_calls): (Vec<_>, Vec<_>) = calls.into_iter().partition_map(|call| {
        if let Some(function) = mcp_functions.get(&call.name) {
            Either::Left((call, function.as_ref()))
        } else {
            Either::Right(call)
        }
    });

    let mut results = Vec::with_capacity(mcp_calls.len());
    for (call, function) in mcp_calls {
        log::debug!("\nMCP tool call:\n{call}");
        let result = function
            .call(call.arguments)
            .await
            .inspect(|output| log::debug!("\nMCP tool result:\n{}", output.data))
            .inspect_err(|e| log::warn!("\nMCP tool error:\n{e}"));
        results.push((call.id, call.name, result));
    }

    McpHandleResult {
        results,
        remaining_calls,
    }
}

/// Constructs a [`ChatCompletionRequestSystemMessageContentPart`]
fn system_message_content_part(part: String) -> ChatCompletionRequestSystemMessageContentPart {
    ChatCompletionRequestSystemMessageContentPart::Text(
        ChatCompletionRequestMessageContentPartText { text: part },
    )
}

/// Constructs a [`ChatCompletionRequestToolMessageContentPart`]
fn tool_message_content_part(part: String) -> ChatCompletionRequestToolMessageContentPart {
    ChatCompletionRequestToolMessageContentPart::Text(ChatCompletionRequestMessageContentPartText {
        text: part,
    })
}

/// Constructs a [`ChatCompletionRequestSystemMessage`] with the given contents.
pub fn system_message(contents: Vec<String>) -> ChatCompletionRequestSystemMessage {
    let content = if contents.len() == 1 {
        let c = contents.into_iter().next().expect("First element exists");
        ChatCompletionRequestSystemMessageContent::Text(c)
    } else {
        let items = contents
            .into_iter()
            .map(system_message_content_part)
            .collect::<Vec<_>>();
        ChatCompletionRequestSystemMessageContent::Array(items)
    };

    ChatCompletionRequestSystemMessage {
        name: None,
        content,
    }
}

/// Constructs a [`ChatCompletionRequestUserMessage`] with the given content.
pub fn user_message(content: impl Into<String>) -> ChatCompletionRequestUserMessage {
    let content: String = content.into();
    ChatCompletionRequestUserMessageArgs::default()
        .content(content)
        .build()
        .expect("All required fields are set.")
}

/// Constructs a [`ChatCompletionRequestAssistantMessage`] with the given content.
pub fn assistant_message(content: impl Into<String>) -> ChatCompletionRequestAssistantMessage {
    let content: String = content.into();
    ChatCompletionRequestAssistantMessageArgs::default()
        .content(content)
        .build()
        .expect("All required fields are set.")
}

/// Constructs a [`ChatCompletionRequestToolMessage`] with the given tool output.
pub fn tool_message(id: &str, data: ToolData) -> ChatCompletionRequestToolMessage {
    let content = match data {
        ToolData::Text(text) => ChatCompletionRequestToolMessageContent::Text(text),
        ToolData::List(items) => {
            let items = items.into_iter().map(tool_message_content_part).collect();
            ChatCompletionRequestToolMessageContent::Array(items)
        }
    };
    ChatCompletionRequestToolMessage {
        tool_call_id: id.into(),
        content,
    }
}

/// Appends additional content to a system message.
pub fn append_system(
    mut system: ChatCompletionRequestSystemMessage,
    content: String,
) -> ChatCompletionRequestSystemMessage {
    match &mut system.content {
        ChatCompletionRequestSystemMessageContent::Text(text) => {
            text.push('\n');
            text.push_str(&content);
        }
        ChatCompletionRequestSystemMessageContent::Array(items) => {
            items.push(ChatCompletionRequestSystemMessageContentPart::Text(
                ChatCompletionRequestMessageContentPartText { text: content },
            ))
        }
    };
    system
}

/// Adds a force final answer message to the given messages.
pub fn add_force_final_answer_message(messages: &mut Vec<ChatCompletionRequestMessage>) {
    if let Some(ChatCompletionRequestMessage::Assistant(msg)) = messages.last_mut() {
        if msg.tool_calls.is_some() {
            msg.tool_calls = None;
            msg.content = Some(ChatCompletionRequestAssistantMessageContent::Text(
                "".into(),
            ))
        }
    } else {
        messages.push(assistant_message("").into());
    }
    messages.push(user_message(FORCE_FINAL_ANSWER).into());
}
