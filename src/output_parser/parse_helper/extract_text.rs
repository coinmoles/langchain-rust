use std::ops::Not;

use regex::Regex;

pub fn extract_thought<'a>(text: &'a str, json: &str) -> Option<&'a str> {
    let re_codeblock = Regex::new(r"\s*```[\w+-]*").unwrap();

    let thought_end = text.find(json)?;
    let thought = text[..thought_end].trim();

    let codeblock_start = re_codeblock
        .find_iter(thought)
        .last()
        .map(|m| m.start())
        .unwrap_or(thought.len());
    let trimmed = thought[..codeblock_start].trim();

    trimmed.is_empty().not().then_some(trimmed)
}

pub fn remove_thought(text: &str) -> &str {
    if text.contains("</think>") {
        let parts: Vec<&str> = text.split("</think>").collect();
        parts.last().unwrap_or(&"").trim()
    } else {
        text
    }
}

pub fn extract_from_codeblock(json_markdown: &str) -> &str {
    let re_single_start = Regex::new(r"^\s*```[\w+-]*").unwrap();
    let re_single_end = Regex::new(r"```\s*$").unwrap();

    let start = re_single_start
        .find_iter(json_markdown)
        .next()
        .map(|m| m.end())
        .unwrap_or(0);

    let end = re_single_end
        .find_iter(json_markdown)
        .last()
        .map(|m| m.start())
        .unwrap_or(json_markdown.len());

    json_markdown[start..end].trim()
}

pub fn extract_from_tag<'a>(text: &'a str, tag: &str) -> &'a str {
    let tag = regex::escape(tag);
    let re_start = Regex::new(&format!(r"<{tag}>")).unwrap();
    let re_end = Regex::new(&format!(r"</{tag}>")).unwrap();

    let start = re_start
        .find_iter(text)
        .next()
        .map(|m| m.end())
        .unwrap_or(0);

    let end = re_end
        .find_iter(text)
        .last()
        .map(|m| m.start())
        .unwrap_or(text.len());

    text[start..end].trim()
}

pub fn extract_json(s: &str) -> &str {
    if s.is_empty() {
        return "";
    }

    let start = match (s.find('{'), s.find('[')) {
        (Some(pos1), Some(pos2)) => pos1.min(pos2),
        (Some(pos), None) | (None, Some(pos)) => pos,
        (None, None) => 0,
    };
    let end = match (s.rfind('}'), s.rfind(']')) {
        (Some(pos1), Some(pos2)) => pos1.max(pos2),
        (Some(pos), None) | (None, Some(pos)) => pos,
        (None, None) => s.len() - 1,
    };

    if end < start {
        log::warn!("Last closing brace/bracket found before the first opening one.");
        return s;
    }

    &s[start..=end]
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_extract_thought() {
        let output = indoc! {r#"
        I think I should use the tool because of that.

        ```json
        {
            "name": "test_tool",
            "arguments": {
                "arg1": "value1"
            }
        }
        ```
        "#};
        let json = indoc! {r#"
        {
            "name": "test_tool",
            "arguments": {
                "arg1": "value1"
            }
        }
        "#};
        let result = extract_thought(output, json);
        let expected = "I think I should use the tool because of that.";
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_extract_from_codeblock() {
        let text = indoc! {r#"
        ```json
        {
            "key": "value"
        }
        ```
        "#};
        let result = extract_from_codeblock(text);
        let expected = indoc! {r#"
            {
                "key": "value"
            }"#
        };
        assert_eq!(result, expected);

        let text = indoc! {r#"
        ```json
        {
            "key": "value"
        }
        "#};
        let result = extract_from_codeblock(text);
        let expected = indoc! {r#"
            {
                "key": "value"
            }"#
        };
        assert_eq!(result, expected);

        let text = indoc! {r#"
        {
            "key": "value"
        }
        ```"#};
        let expected = indoc! {r#"
            {
                "key": "value"
            }"#
        };
        let result = extract_from_codeblock(text);
        assert_eq!(result, expected);
    }

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

    #[test]
    fn test_extract_json() {
        let text = indoc! {r#"
        So I decided to call this because of that:
        
        {
            "name": "test_tool",
            "arguments": {
                "arg1": "value1"
            }
        }"#};

        let result = extract_json(text);
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
