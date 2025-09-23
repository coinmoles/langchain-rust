use async_openai::Client as OpenAiClient;
use async_openai::config::Config;
use async_openai::error::OpenAIError;
use async_openai::types::responses::{Content, OutputContent, Response, ResponseStream};
use futures::StreamExt;

use crate::llm::{LLMError, LLMEvent, LLMOutput, LLMStream, LLMStreamChunk, ResponsesRequest};
use crate::schemas::{TokenUsage, ToolCall};

// fn add_option_numbers<T>(a: Option<T>, b: Option<T>) -> Option<T>
// where
//     T: Add<Output = T> + Default + Copy,
// {
//     match (a, b) {
//         (None, None) => None,
//         _ => Some(a.unwrap_or_default() + b.unwrap_or_default()),
//     }
// }

// fn merge_prompt_tokens_details(
//     details: Option<PromptTokensDetails>,
//     chunk_details: Option<PromptTokensDetails>,
// ) -> Option<PromptTokensDetails> {
//     match (details, chunk_details) {
//         (None, None) => None,
//         (Some(details), None) => Some(details),
//         (None, Some(details)) => Some(details),
//         (Some(details), Some(chunk_details)) => Some(PromptTokensDetails {
//             audio_tokens: add_option_numbers(details.audio_tokens, chunk_details.audio_tokens),
//             cached_tokens: add_option_numbers(details.cached_tokens,
// chunk_details.cached_tokens),         }),
//     }
// }

// fn merge_completion_tokens_details(
//     details: Option<CompletionTokensDetails>,
//     chunk_details: Option<CompletionTokensDetails>,
// ) -> Option<CompletionTokensDetails> {
//     match (details, chunk_details) {
//         (None, None) => None,
//         (Some(details), None) => Some(details),
//         (None, Some(details)) => Some(details),
//         (Some(details), Some(chunk_details)) => Some(CompletionTokensDetails {
//             accepted_prediction_tokens: add_option_numbers(
//                 details.accepted_prediction_tokens,
//                 chunk_details.accepted_prediction_tokens,
//             ),
//             audio_tokens: add_option_numbers(details.audio_tokens, chunk_details.audio_tokens),
//             reasoning_tokens: add_option_numbers(
//                 details.reasoning_tokens,
//                 chunk_details.reasoning_tokens,
//             ),
//             rejected_prediction_tokens: add_option_numbers(
//                 details.rejected_prediction_tokens,
//                 chunk_details.rejected_prediction_tokens,
//             ),
//         }),
//     }
// }

// fn merge_usage(
//     usage: Option<CompletionUsage>,
//     chunk_usage: Option<CompletionUsage>,
// ) -> Option<CompletionUsage> {
//     match (usage, chunk_usage) {
//         (None, None) => None,
//         (Some(usage), None) => Some(usage),
//         (None, Some(usage)) => Some(usage),
//         (Some(usage), Some(chunk_usage)) => Some(CompletionUsage {
//             prompt_tokens: usage.prompt_tokens + chunk_usage.prompt_tokens,
//             completion_tokens: usage.completion_tokens + chunk_usage.completion_tokens,
//             total_tokens: usage.total_tokens + chunk_usage.total_tokens,
//             prompt_tokens_details: merge_prompt_tokens_details(
//                 usage.prompt_tokens_details,
//                 chunk_usage.prompt_tokens_details,
//             ),
//             completion_tokens_details: merge_completion_tokens_details(
//                 usage.completion_tokens_details,
//                 chunk_usage.completion_tokens_details,
//             ),
//         }),
//     }
// }

// pub async fn collect_response(mut stream: ResponseStream) -> Result<Response, OpenAIError> {
//     let mut meta: Option<ResponseMetadata> = None;
//     let mut finished_ok = false;

//     let mut items: Vec<OutputItem> = Vec::new();

//     while let Some(evt) = stream.next().await {
//         let evt = evt?;

//         match evt {
//             ResponseEvent::ResponseCreated(created) => meta = Some(created.response),
//             ResponseEvent::ResponseInProgress(in_prog) => meta = Some(in_prog.response),
//             ResponseEvent::ResponseQueued(queued) => meta = Some(queued.response),
//             ResponseEvent::ResponseCompleted(completed) => {
//                 meta = Some(completed.response);
//                 finished_ok = true;
//             }
//             ResponseEvent::ResponseFailed(failed) => {
//                 if let Some(err) = failed.response.error {
//                     let msg = format!("Stream failed (code: {}): {}", err.code, err.message);
//                     return Err(OpenAIError::StreamError(msg));
//                 } else {
//                     let msg = "Stream failed, no error message provided".to_string();
//                     return Err(OpenAIError::StreamError(msg));
//                 }
//             }
//             ResponseEvent::ResponseIncomplete(incomplete) => {
//                 if let Some(details) = incomplete.response.incomplete_details {
//                     let msg = format!("Stream incomplete, reason: {}", details.reason);
//                     return Err(OpenAIError::StreamError(msg));
//                 } else {
//                     let msg = "Stream incomplete, no reason provided".to_string();
//                     return Err(OpenAIError::StreamError(msg));
//                 }
//             }
//             ResponseEvent::ResponseOutputItemDone(done) => {
//                 items.push(done.item);
//             }
//             _ => {}
//         }
//     }

//     let Some(meta) = meta else {
//         return Err(OpenAIError::StreamError(
//             "Stream yielded no metadata before closing".to_string(),
//         ));
//     };

//     let response = Response {
//         id: meta.id,
//         object: meta.object.unwrap_or("response".to_string()),
//         created_at: meta.created_at,
//         model: meta.model.unwrap_or("unknown".to_string()),
//         status: meta.status,
//         usage: meta.usage,
//         error: meta.error,
//         incomplete_details: meta.incomplete_details,
//         instructions: meta.instructions,
//         max_output_tokens: meta.max_output_tokens,
//         metadata: meta.metadata,
//         output: meta
//             .output
//             .unwrap_or_default()
//             .into_iter()
//             .map(Into::into)
//             .collect(),
//         output_text: None,
//         parallel_tool_calls: meta.parallel_tool_calls,
//         previous_response_id: meta.previous_response_id,
//         reasoning: meta.reasoning,
//         store: meta.store,
//         service_tier: meta.service_tier,
//         temperature: meta.temperature,
//         text: meta.text,
//         tool_choice: meta.tool_choice,
//         tools: meta.tools,
//         top_p: meta.top_p,
//         truncation: meta.truncation,
//         user: meta.user,
//     };

//     Ok(response)
// }

pub fn construct_output(output: Vec<OutputContent>) -> Result<LLMOutput, LLMError> {
    let mut text = None;
    let mut tool_calls: Vec<ToolCall> = vec![];

    // TODO: improve output handling
    for item in output {
        match item {
            OutputContent::Message(msg) => match msg.content.into_iter().next() {
                Some(Content::OutputText(t)) => text = Some(t.text),
                Some(Content::Refusal(r)) => return Err(LLMError::Refused(r.refusal)),
                None => continue,
            },
            OutputContent::FunctionCall(call) => {
                let tc = ToolCall::try_from(call)?;
                tool_calls.push(tc);
            }
            _ => continue,
        }
    }

    let event = if !tool_calls.is_empty() {
        LLMEvent::ToolCall(tool_calls)
    } else if let Some(t) = text {
        LLMEvent::Text(t)
    } else {
        return Err(LLMError::ContentNotFound(
            "No text or tool call found in output".to_string(),
        ));
    };

    Ok(LLMOutput {
        thought: None,
        event,
    })
}

pub async fn generate<C: Config>(
    client: &OpenAiClient<C>,
    mut request: ResponsesRequest,
    stream: bool,
) -> Result<Response, OpenAIError> {
    if stream {
        request.stream = Some(false);
    }
    // if stream {
    //     let stream = client
    //         .responses()
    //         .create_stream_byot::<_, ResponseEvent>(request)
    //         .await?;

    //     let response = collect_response(stream).await?;
    //     return Ok(response);
    // }

    let response = client.chat().create_byot::<_, Response>(request).await?;
    Ok(response)
}

pub fn map_stream(original: ResponseStream) -> LLMStream {
    let new = original.map(|result| match result {
        Ok(completion) => {
            let value_completion = serde_json::to_value(completion).map_err(LLMError::from)?;
            let usage = value_completion.pointer("/usage");
            if usage.is_some() && !usage.unwrap().is_null() {
                let usage = serde_json::from_value::<TokenUsage>(usage.unwrap().clone())
                    .map_err(LLMError::from)?;
                return Ok(LLMStreamChunk::new(value_completion, Some(usage), ""));
            }
            let content = value_completion
                .pointer("/choices/0/delta/content")
                .ok_or(LLMError::ContentNotFound(
                    "/choices/0/delta/content".to_string(),
                ))?
                .clone();

            Ok(LLMStreamChunk::new(
                value_completion,
                None,
                content.as_str().unwrap_or(""),
            ))
        }
        Err(e) => Err(LLMError::from(e)),
    });
    Box::pin(new)
}
