use crate::{tools::Tool, utils::helper::normalize_tool_name};

pub const DEFAULT_TOOL_PROMPT: &str = r#"

<INSTRUCTIONS>
- You have two options:
    1. Use a tool
    2. Give your final answer
- You may repeat tool use cycle as many times as needed before giving your final answer
- When not using a tool, directly give your final answer
- ALL RESPONSES MUST BE IN JSON FORMAT

Option 1 : Use a tool (If you have tools and you need to use them)
The following is the description of the tools available to you:
{{tools}}
- IF YOU DON'T HAVE TOOLS, PASS THIS OPTION

<TOOL_USAGE_OUTPUT_FORMAT>
{
    "action": (string), The action to take; MUST BE one of [{{tool_names}}]
    "action_input": (object), The input to the action, JSON object. The structure object depends on the action you are taking, and is specified in the tool description below.
}
</TOOL_USAGE_OUTPUT_FORMAT>


Option 2 : Give your best final answer
- Only return a final answer once all required tools have been used
- **NEVER RETURN TOOL USE PLAN AS A FINAL ANSWER**

<FINAL_ANSWER_OUTPUT_FORMAT>
{
    "final_answer": (string), Your final answer must be the robust and COMPLETE; it must be outcome described
}
</FINAL_ANSWER_OUTPUT_FORMAT>

</INSTRUCTIONS>"#;

pub fn default_tool_prompt(tools: &[&dyn Tool]) -> String {
    let tool_names = tools
        .iter()
        .map(|tool| normalize_tool_name(&tool.name()))
        .collect::<Vec<_>>()
        .join(", ");
    let tool_string = tools
        .iter()
        .map(|tool| tool.to_plain_description())
        .collect::<Vec<_>>()
        .join("\n");
    DEFAULT_TOOL_PROMPT
        .replace("{{tool_names}}", &tool_names)
        .replace("{{tools}}", &tool_string)
}
