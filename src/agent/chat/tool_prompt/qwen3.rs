use crate::tools::Tool;

const QWEN_SUFFIX: &str = r#"

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

pub fn qwen3_custom_tool_prompt(tools: &[&dyn Tool]) -> String {
    let tools_json = tools
        .iter()
        .map(|tool| {
            let tool = tool.into_openai_tool();
            serde_json::to_string_pretty(&tool).unwrap_or_else(|_| format!("{:#?}", tool))
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    QWEN_SUFFIX.replace("{{tools}}", &tools_json)
}
