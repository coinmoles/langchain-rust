use serde::Deserialize;
use serde_json::Value;

use super::Instructor;
use crate::schemas::{FunctionSpec, LLMEvent, LLMOutput, ToolCall};
use crate::utils::parse::{
    ParseError, extract_from_codeblock, extract_from_tag, extract_thought, flatten_final_answer,
    is_malformed_event, is_malformed_event_str, parse_partial_json, remove_thought,
};

const QWEN3_TOOL_PROMPT: &str = r#"

# Tools

You may call one or more functions to assist with the user query.

You are provided with function signatures within <tools></tools> XML tags:
<tools>
{{?tools}}
</tools>

For each function call, return a json object with function name and arguments within <tool_call></tool-call> XML tags:
<tool_call>
{"name": <function-name>, "arguments": <args-json-object>}
</tool_call>"#;

const NAME_KEY: &str = "name";
const ARGUMENTS_KEY: &str = "arguments";
const ALTERNATIVE_NAME_KEY: &str = "action";
const ALTERNATIVE_ARGUMENTS_KEY: &str = "action_input";
const VALID_KEYS: &[&[&str]] = &[&[NAME_KEY, ARGUMENTS_KEY]];
const ALTERNATIVE_KEYS: &[&[&str]] = &[&[ALTERNATIVE_NAME_KEY, ALTERNATIVE_ARGUMENTS_KEY]];

/// The instructor implementation for Qwen3 models.
///
/// Uses the [Qwen3 chat template](https://huggingface.co/Qwen/Qwen3-235B-A22B-Thinking-2507?chat_template=default) for tool use instructions.
#[derive(Default)]
pub struct Qwen3Instructor;

impl Qwen3Instructor {
    /// Deserializes the LLM output json.
    fn deserialize_tool_call(&self, value: Value) -> Result<LLMEvent, serde_json::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum OutputHelp {
            Action {
                #[serde(default)]
                id: Option<String>,
                #[serde(alias = "action")]
                name: String,
                #[serde(default, alias = "action_input")]
                arguments: Option<Value>,
            },
            FinalAnswer {
                final_answer: Value,
            },
        }

        let helper: OutputHelp = serde_json::from_value(value)?;
        let event = match helper {
            OutputHelp::Action {
                id,
                name,
                arguments,
            } => {
                let tool_call = ToolCall::new(id, name, arguments);
                LLMEvent::ToolCall(vec![tool_call])
            }
            OutputHelp::FinalAnswer { final_answer } => {
                let final_answer = flatten_final_answer(final_answer)?;
                LLMEvent::Text(final_answer)
            }
        };
        Ok(event)
    }
}

impl Instructor for Qwen3Instructor {
    fn tool_use_instruction(&self, tools: &[FunctionSpec]) -> String {
        let tools_str = tools
            .iter()
            .map(FunctionSpec::as_json)
            .collect::<Vec<_>>()
            .join("\n");
        QWEN3_TOOL_PROMPT.replace("{{?tools}}", &tools_str)
    }

    fn parse_tool_use<'a>(&self, output: String) -> Result<LLMOutput, ParseError> {
        let text = remove_thought(&output);
        let text = extract_from_tag(text, "tool_call");
        let text = extract_from_codeblock(text);

        let thought = extract_thought(&output, text).map(Into::into);
        let json = parse_partial_json(text, false);

        let is_malformed_event = match json.as_ref() {
            Ok(json) => {
                is_malformed_event(json, VALID_KEYS) || is_malformed_event(json, ALTERNATIVE_KEYS)
            }
            Err(_) => {
                is_malformed_event_str(text, VALID_KEYS)
                    || is_malformed_event_str(text, ALTERNATIVE_KEYS)
            }
        };

        let event = match json.and_then(|json| self.deserialize_tool_call(json)) {
            Ok(event) => event,
            Err(_) if !is_malformed_event => LLMEvent::Text(text.into()),
            Err(e) => return Err(ParseError::Deserialize(e, text.into())),
        };

        Ok(LLMOutput { thought, event })
    }

    fn clone_box(&self) -> Box<dyn Instructor> {
        Box::new(Self)
    }
}
