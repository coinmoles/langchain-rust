use serde::{de::Error, Deserialize};
use serde_json::Value;

use crate::{
    agent::AgentOutput,
    output_parser::{
        extract_from_codeblock, extract_from_tag, is_malformed_event, is_malformed_event_str,
        parse_partial_json, remove_thought, OutputParseError,
    },
    schemas::ToolCall,
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

pub struct Qwen3Instructor;

impl Default for Qwen3Instructor {
    fn default() -> Self {
        Self
    }
}

impl Qwen3Instructor {
    fn value_to_agent_event(&self, value: Value) -> Result<AgentOutput, serde_json::Error> {
        #[derive(Deserialize)]
        struct AgentEventHelp {
            #[serde(default)]
            id: Option<String>,
            #[serde(default)]
            #[serde(alias = "action")]
            name: Option<String>,
            #[serde(default)]
            #[serde(alias = "action_input")]
            arguments: Option<Value>,
            #[serde(default)]
            final_answer: Option<Value>,
        }

        let AgentEventHelp {
            id,
            name: action,
            arguments: action_input,
            final_answer,
        } = serde_json::from_value(value)?;

        if let Some(name) = action {
            let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let arguments = action_input.unwrap_or(Value::Null);
            let tool_call = ToolCall::new(id, name, arguments);
            Ok(AgentOutput::Action(vec![tool_call]))
        } else if let Some(final_answer) = final_answer {
            let final_answer = match final_answer {
                Value::String(value) => value,
                other => serde_json::to_string_pretty(&other)?,
            };
            Ok(AgentOutput::Finish(final_answer))
        } else {
            Err(serde_json::Error::missing_field("action or final_answer"))
        }
    }
}

impl Instructor for Qwen3Instructor {
    fn create_suffix(&self, tools: &[&dyn Tool]) -> String {
        let tools_json = tools
            .iter()
            .map(|tool| {
                let tool = tool.as_openai_tool();
                serde_json::to_string_pretty(&tool).unwrap_or_else(|_| format!("{tool:#?}"))
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        QWEN3_TOOL_PROMPT.replace("{{tools}}", &tools_json)
    }

    fn parse_from_text<'a>(&self, output: String) -> Result<AgentOutput, OutputParseError> {
        let text = remove_thought(&output);
        let text = extract_from_tag(text, "tool_call");
        let text = extract_from_codeblock(text);

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

        match json.and_then(|json| self.value_to_agent_event(json)) {
            Ok(agent_event) => Ok(agent_event),
            Err(_) if !is_malformed_event => Ok(AgentOutput::Finish(text.into())),
            Err(e) => Err(e.into()),
        }
    }
}
