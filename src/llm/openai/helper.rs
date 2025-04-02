use futures::{Stream, StreamExt};
use std::{collections::HashMap, ops::Add, sync::Arc};
use tokio::sync::Mutex;

use async_openai::{
    error::OpenAIError,
    types::{
        ChatChoice, ChatChoiceStream, ChatCompletionMessageToolCall, ChatCompletionResponseMessage,
        ChatCompletionToolType, CompletionTokensDetails, CompletionUsage,
        CreateChatCompletionResponse, CreateChatCompletionStreamResponse, FinishReason,
        FunctionCall, PromptTokensDetails, Role,
    },
};

use crate::schemas::StreamingFunc;

fn add_option_numbers<T>(a: Option<T>, b: Option<T>) -> Option<T>
where
    T: Add<Output = T> + Default + Copy,
{
    match (a, b) {
        (None, None) => None,
        _ => Some(a.unwrap_or_default() + b.unwrap_or_default()),
    }
}

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

    // Deprecated: function_call
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

pub async fn construct_chat_completion_response(
    mut stream: impl Stream<Item = Result<CreateChatCompletionStreamResponse, async_openai::error::OpenAIError>>
        + Send
        + Unpin,
    streaming_func: &Option<Arc<Mutex<StreamingFunc>>>,
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
            if let Some(content) = choice_stream.delta.content.as_deref() {
                if let Some(streaming_func) = &streaming_func {
                    let mut func = streaming_func.lock().await;
                    let _ = func(content).await;
                }
            }

            #[allow(deprecated)]
            let choice = choices_map
                .entry(choice_stream.index)
                .or_insert_with(|| ChatChoice {
                    index: choice_stream.index,
                    message: ChatCompletionResponseMessage {
                        content: None,
                        refusal: None,
                        tool_calls: None,
                        role: Role::Assistant,
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

pub fn select_choice(choices: Vec<ChatChoice>) -> Option<ChatChoice> {
    if choices.is_empty() {
        return None;
    }

    let mut choices = choices.clone();
    choices.sort_by(|c1, c2| {
        let rank = |c: &ChatChoice| match c.finish_reason {
            Some(FinishReason::ContentFilter) | Some(FinishReason::Length) => 1,
            _ => 0,
        };

        // First sort by rank (0 = preferred, 1 = deprioritized)
        // Then by index to preserve order within ranks
        rank(c1)
            .cmp(&rank(c2))
            .then_with(|| c1.index.cmp(&c2.index))
    });
    let selected_choice = choices.first()?;

    Some(selected_choice.clone())
}
