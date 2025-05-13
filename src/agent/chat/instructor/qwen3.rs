use serde_json::Value;

use crate::{
    agent::{
        chat::parse_helper::{
            extract_from_tag, is_malformed_event, is_malformed_event_str, parse_partial_json,
            remove_thought, take_action,
        },
        AgentError,
    },
    schemas::{AgentEvent, ToolCall},
    tools::Tool,
};

use super::Instructor;

const QWEN3_TOOL_PROMPT: &str = r#"

# Tools

You may call one or more functions to assist with the user query.

You are provided with function signatures within <tools></tools> XML tags:
<tools>
{{tools}}
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

pub struct Qwen3Instructor {}

impl Default for Qwen3Instructor {
    fn default() -> Self {
        Self::new()
    }
}

impl Qwen3Instructor {
    pub fn new() -> Self {
        Self {}
    }

    fn value_to_agent_event(&self, value: Value) -> Option<AgentEvent> {
        let Value::Object(mut obj) = value else {
            return None;
        };

        if let Some((id, name, arguments)) = take_action(&mut obj, NAME_KEY, ARGUMENTS_KEY) {
            let action = AgentEvent::Action(vec![ToolCall {
                id,
                name,
                arguments,
            }]);
            Some(action)
        } else if let Some((id, name, arguments)) =
            take_action(&mut obj, ALTERNATIVE_NAME_KEY, ALTERNATIVE_ARGUMENTS_KEY)
        {
            let action = AgentEvent::Action(vec![ToolCall {
                id,
                name,
                arguments,
            }]);
            Some(action)
        } else {
            None
        }
    }
}

impl Instructor for Qwen3Instructor {
    fn create_suffix(&self, tools: &[&dyn Tool]) -> String {
        let tools_json = tools
            .iter()
            .map(|tool| {
                let tool = tool.as_openai_tool();
                serde_json::to_string_pretty(&tool).unwrap_or_else(|_| format!("{:#?}", tool))
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        QWEN3_TOOL_PROMPT.replace("{{tools}}", &tools_json)
    }

    fn parse_output(&self, output: &str) -> Result<AgentEvent, AgentError> {
        let text = remove_thought(output);
        let text = extract_from_tag(text, "tool_call");

        let json = parse_partial_json(text, false);

        let is_malformed_event = match json.as_ref() {
            Some(json) => {
                is_malformed_event(json, VALID_KEYS) || is_malformed_event(json, ALTERNATIVE_KEYS)
            }
            None => {
                is_malformed_event_str(text, VALID_KEYS)
                    || is_malformed_event_str(text, ALTERNATIVE_KEYS)
            }
        };

        match json.and_then(|json| self.value_to_agent_event(json)) {
            Some(agent_event) => Ok(agent_event),
            None if !is_malformed_event => Ok(AgentEvent::Finish(text.into())),
            _ => Err(AgentError::InvalidFormatError(text.into())),
        }
    }
}
