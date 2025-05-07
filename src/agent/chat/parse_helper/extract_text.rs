use regex::Regex;

pub fn remove_thought(text: &str) -> &str {
    if text.contains("</think>") {
        let parts: Vec<&str> = text.split("</think>").collect();
        parts.last().unwrap_or(&"").trim()
    } else {
        text
    }
}

pub fn extract_from_codeblock(json_markdown: &str) -> &str {
    let re = Regex::new(r"```(?:(?:[\w+-]\s*)+)?\s*\n\s*([\s\S]+?)\s*```").unwrap();
    if let Some(caps) = re.captures(json_markdown) {
        if let Some(json_str) = caps.get(1) {
            return json_str.as_str().trim();
        }
    }
    json_markdown
}

pub fn extract_from_tag<'a>(text: &'a str, tag: &str) -> &'a str {
    fn try_extract<'a>(text: &'a str, pattern: &str) -> Option<&'a str> {
        Regex::new(pattern)
            .ok()
            .and_then(|re| re.captures(text))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim())
    }

    let full = format!(r"(?s)<{0}>\s*(.*?)\s*</{0}>", tag);
    let open_only = format!(r"(?s)<{}>\s*(.*?)\s*$", tag);
    let close_only = format!(r"(?s)^\s*(.*?)\s*</{}>", tag);

    try_extract(text, &full)
        .or_else(|| try_extract(text, &open_only))
        .or_else(|| try_extract(text, &close_only))
        .unwrap_or_else(|| text.trim())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_extract_from_tag() {
        let text =
            r#"<tool_call> {"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call>"#;
        let result = extract_from_tag(text, "tool_call");
        assert_eq!(
            result,
            r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
        );

        let text = r#"<tool_call> {"name": "test_tool", "arguments": {"arg1": "value1"}}"#;
        let result = extract_from_tag(text, "tool_call");
        assert_eq!(
            result,
            r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
        );

        let text = r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call>"#;
        let result = extract_from_tag(text, "tool_call");
        assert_eq!(
            result,
            r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
        );

        let text = r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#;
        let result = extract_from_tag(text, "tool_call");
        assert_eq!(
            result,
            r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
        );

        let text = r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call> <FINAL_ANSWER_FORMAT>final answer"#;
        let result = extract_from_tag(text, "tool_call");
        assert_eq!(
            result,
            r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
        );

        let text = indoc! {r#"
        <tool_call> 
        {
            "name": "test_tool",
            "arguments": {
                "arg1": "value1"
            }
        }
        </tool_call>"#};
        let result = extract_from_tag(text, "tool_call");
        assert_eq!(
            result,
            indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }"#}
        );
    }
}
