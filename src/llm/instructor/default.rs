use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use super::Instructor;
use crate::schemas::{FunctionSpec, LLMEvent, LLMOutput, ToolCall};
use crate::utils::helper::normalize_tool_name;
use crate::utils::parse::{
    ParseError, extract_from_codeblock, extract_json, extract_thought, fix_text,
    flatten_final_answer, is_malformed_event, parse_partial_json, remove_thought,
};

const DEFAULT_TOOL_PROMPT: &str = r#"<TOOL_INTEGRATION>
- You have access to a set of tools to help you answer questions.
- You can either use a tool or provide your final answer.
- You may use the tools iteratively as many times as needed.
</TOOL_INTEGRATION>

<TOOL_INSTRUCTIONS>
Available tools are described below:
{{?tools}}

To use a tool, return a JSON object with the following structure:
  - "name": The name of the tool to use (must be one of: [{{?tool_names}}])
  - "arguments": The arguments to pass to the tool (must conform to the tool's schema)

Format:
```json
{
    "name": (string),
    "arguments": (JSON-serializable value)
}
```
</TOOL_INSTRUCTIONS>"#;

const ACTION_KEY: &str = "name";
const ACTION_INPUT_KEY: &str = "arguments";
// const FINAL_ANSWER_KEY: &str = "final_answer";
const VALID_KEYS: &[&[&str]] = &[&[ACTION_KEY, ACTION_INPUT_KEY]];

/// The default instructor implementation.
#[derive(Default)]
pub struct DefaultInstructor;

impl DefaultInstructor {
    /// Deserializes the LLM output json.
    fn deserialize_llm_output(&self, value: Value) -> Result<LLMEvent, serde_json::Error> {
        #[derive(Debug, Deserialize)]
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

    /// Parses the LLM output using regex as a fallback.
    fn parse_with_regex(&self, text: &str) -> Option<LLMEvent> {
        let final_answer_re = Regex::new(r#"(?m)"final_answer"\s*:\s*"(.*)"\s*\n"#).unwrap();
        let action_regex = Regex::new(r#"(?m)"action"\s*:\s*"(.*)"\s*\n"#).unwrap();
        let action_input_regex = Regex::new(r#"(?m)"action_input"\s*:\s*"(.*)"\s*\n"#).unwrap();

        if let Some(final_answer) = final_answer_re.captures(text) {
            let final_answer = final_answer.get(1)?.as_str();
            return Some(LLMEvent::Text(fix_text(final_answer)));
        }

        if let (Some(action), Some(action_input)) = (
            action_regex.captures(text),
            action_input_regex.captures(text),
        ) {
            let action = action.get(1)?.as_str();
            let action_input = action_input.get(1)?.as_str();
            let action = fix_text(action);
            let action_input = serde_json::from_str(action_input).ok()?;
            let tool_call = ToolCall::new(None, action.to_string(), Some(action_input));
            return Some(LLMEvent::ToolCall(vec![tool_call]));
        }

        None
    }
}

impl Instructor for DefaultInstructor {
    fn tool_use_instruction(&self, tools: &[FunctionSpec]) -> String {
        let tool_names = tools
            .iter()
            .map(|t| normalize_tool_name(t.name.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        let tool_descriptions = tools
            .iter()
            .map(FunctionSpec::describe)
            .collect::<Vec<_>>()
            .join("\n");
        DEFAULT_TOOL_PROMPT
            .replace("{{?tool_names}}", &tool_names)
            .replace("{{?tools}}", &tool_descriptions)
    }

    fn parse_tool_use(&self, output: String) -> Result<LLMOutput, ParseError> {
        let text = remove_thought(&output);
        let text = extract_from_codeblock(text);
        let text = extract_json(text);

        let json = parse_partial_json(text, false);

        let is_malformed_event = json
            .as_ref()
            .is_ok_and(|json| is_malformed_event(json, VALID_KEYS));

        let event = match json
            .and_then(|json| self.deserialize_llm_output(json))
            .or_else(|e| self.parse_with_regex(text).ok_or(e))
        {
            Ok(evt) => evt,
            Err(_) if !is_malformed_event => return Ok(LLMOutput::from(LLMEvent::Text(output))),
            Err(e) => return Err(ParseError::Deserialize(e, output)),
        };
        let thought = extract_thought(&output, text).map(Into::into);

        Ok(LLMOutput { event, thought })
    }

    fn clone_box(&self) -> Box<dyn Instructor> {
        Box::new(Self)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case::simple(
        indoc! {r#"
            ```json
            {
                "name": "generate",
                "arguments": "Hello, world!"
            }
            ```"#
        },
        None,
        "generate",
        json!("Hello, world!"),
    )]
    #[case::complex_arguments(
        indoc! {r#"
            {
                "name": "get_weather",
                "arguments": {
                    "location": "New York",
                    "date": "2023-10-01"
                }
            }
            ```"#
        },
        None,
        "get_weather",
        json!({
            "location": "New York",
            "date": "2023-10-01"
        }),
    )]
    #[case::with_thought(
        indoc! {r#"
            First, I should gather basic information about the rust programming language.
            ```json
            {
                "action": "search",
                "action_input": {
                    "query": "Rust programming language",
                }
            }"#
        },
        Some("First, I should gather basic information about the rust programming language."),
        "search",
        json!({
            "query": "Rust programming language"
        }),
    )]
    fn test_parse_tool_call(
        #[case] output: &str,
        #[case] thought: Option<&str>,
        #[case] name: &str,
        #[case] argument: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = DefaultInstructor.parse_tool_use(output.into())?;
        assert_eq!(output.thought.as_deref(), thought);
        let LLMEvent::ToolCall(tool_calls) = output.event else {
            panic!("Expected `LLMEvent::ToolCall` got {output:#?}");
        };
        let tool_call = &tool_calls[0];
        assert_eq!(tool_call.name, name);
        assert_eq!(tool_call.arguments, argument);

        Ok(())
    }

    #[rstest]
    #[case::simple(r#"Goodbye, world!"#, None, "Goodbye, world!")]
    #[case::simple_with_json(
        "The possible options were: [A, B, C, D], and I think A is the best.",
        None,
        "The possible options were: [A, B, C, D], and I think A is the best."
    )]
    #[case::json(
        indoc! {r#"
            ```json
            [
                {
                    "recipe": "Pancakes",
                    "ingredients": ["egg", "milk", "flour"]
                },
                {
                    "recipe": "Omelette",
                    "ingredients": ["egg", "cheese", "ham"]
                }
            ]
            ```"#
        },
        None,
        indoc! {r#"
            ```json
            [
                {
                    "recipe": "Pancakes",
                    "ingredients": ["egg", "milk", "flour"]
                },
                {
                    "recipe": "Omelette",
                    "ingredients": ["egg", "cheese", "ham"]
                }
            ]
            ```"#
        }
    )]
    #[case::final_answer(
        indoc! {r#"
            After considering all options, I conclude that the capital of France is Paris.
            ```json
            {
                "final_answer": "The capital of France is Paris."
            }
            ```
        "#},
        Some("After considering all options, I conclude that the capital of France is Paris."),
        "The capital of France is Paris."
    )]
    #[case::final_answer_messy(
        indoc! {r#"
            Here are some suggestions for the dinner menu :)
            ```json
            {
            "final_answer": "\## Thinking process\n\n### Step 1:\n- Candidate 1: Mac and Cheese\n- Candidate 2: Dumplings\n- Candidate 3: Pancakes"
            }
            ```
        "#},
        Some("Here are some suggestions for the dinner menu :)"),
        indoc! {
            r#"## Thinking process

            ### Step 1:
            - Candidate 1: Mac and Cheese
            - Candidate 2: Dumplings
            - Candidate 3: Pancakes"#
        }
    )]
    #[case::final_answer_multilne(
        indoc! {r#"
            {
                "final_answer": "I don't think pancakes are good for dinner. Here is a detailed explanation:

            - Pancakes are typically considered a breakfast food and may not provide the necessary nutrients for a balanced dinner.
            - They are often high in carbohydrates and sugars, which can lead to energy crashes later in the evening.
            - For dinner, it's generally better to have a meal that includes a good balance of protein, vegetables, and healthy fats."
            }"#},
        None,
        indoc! {r#"
            I don't think pancakes are good for dinner. Here is a detailed explanation:

            - Pancakes are typically considered a breakfast food and may not provide the necessary nutrients for a balanced dinner.
            - They are often high in carbohydrates and sugars, which can lead to energy crashes later in the evening.
            - For dinner, it's generally better to have a meal that includes a good balance of protein, vegetables, and healthy fats."#
        },
    )]
    #[case::final_answer_nested(
        indoc! {r#"
            ```json
            {
                "final_answer": {
                    "final_answer": "The capital of France is Paris.",
                }
            }
            ```
        "#},
        None,
        "The capital of France is Paris.",
    )]
    fn test_parse_final_answer(
        #[case] output: &str,
        #[case] thought: Option<&str>,
        #[case] text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = DefaultInstructor.parse_tool_use(output.into())?;
        assert_eq!(output.thought.as_deref(), thought);
        let LLMEvent::Text(evt_text) = output.event else {
            panic!("Expected `LLMEvent::Text` got {output:#?}");
        };
        assert_eq!(evt_text, text);
        Ok(())
    }
}
