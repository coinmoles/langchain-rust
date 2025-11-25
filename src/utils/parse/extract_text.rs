use std::ops::Not;

use regex::Regex;

/// Returns the trimmed text after the closing `</think>` tag. If no tag is found, returns the input
/// unchanged.
pub fn remove_thought(text: &str) -> &str {
    if text.contains("</think>") {
        let parts: Vec<&str> = text.split("</think>").collect();
        parts.last().unwrap_or(&"").trim()
    } else {
        text
    }
}

/// Returns the trimmed text inside a Markdown code block. If no code block is found, returns the
/// input unchanged.
pub fn extract_from_codeblock(text: &str) -> &str {
    let re_single = Regex::new(r"(?m)^\s*```[A-Za-z0-9_+.-]*").unwrap();

    let start = re_single
        .find_iter(text)
        .next()
        .map(|m| m.end())
        .unwrap_or(0);

    let out = text.get(start..).unwrap_or(text);
    let out = if out.is_empty() { text } else { out };
    out.trim_end_matches("```").trim_end()
}

/// Returns the trimmed content inside the specified XML-like tag. If no tag is found, returns the
/// input unchanged.
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

/// Returns the substring containing the first valid JSON object or array. If no valid JSON, returns
/// the input unchanged.
pub fn extract_json(text: &str) -> &str {
    if text.is_empty() {
        return "";
    }
    let text = extract_from_codeblock(text);

    let start = match (text.find('{'), text.find('[')) {
        (Some(pos1), Some(pos2)) => pos1.min(pos2),
        (Some(pos), None) | (None, Some(pos)) => pos,
        (None, None) => 0,
    };
    let end = match (text.rfind('}'), text.rfind(']')) {
        (Some(pos1), Some(pos2)) => pos1.max(pos2),
        (Some(pos), None) | (None, Some(pos)) => pos,
        (None, None) => text.len() - 1,
    };

    if end < start {
        log::warn!("Last closing brace/bracket found before the first opening one.");
        return text;
    }

    &text[start..=end]
}

/// Returns the trimmed text preceding the given JSON. If the preceding text is empty, returns
/// `None`.
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(indoc! {r#"
        ```json
        {
            "key": "value"
        }
        ```"#
    }, indoc! {r#"
        {
            "key": "value"
        }"#
    })]
    #[case(indoc! {r#"
        Pikachu

        ```json
        {
            "key": "value"
        }
        ```"#
    }, indoc! {r#"
        {
            "key": "value"
        }"#
    })]
    #[case(indoc! {r#"
        ```json
        {
            "key": "value"
        }"#
    }, indoc! {r#"
        {
            "key": "value"
        }"#
    })]
    #[case(indoc! {r#"
        {
            "key": "value"
        }
        ```"#
    }, indoc! {r#"
        {
            "key": "value"
        }"#
    })]
    fn test_extract_from_codeblock(#[case] text: &str, #[case] expected: &str) {
        let extracted = extract_from_codeblock(text);
        assert_eq!(extracted, expected);
    }

    #[rstest]
    #[case(
        r#"<tool_call> {"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call>"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"<tool_call> {"name": "test_tool", "arguments": {"arg1": "value1"}}"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call>"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call> <FINAL_ANSWER_FORMAT>final answer"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        indoc! {r#"
            <tool_call>
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            </tool_call>"#
        },
        "tool_call",
        indoc! {r#"
        {
            "name": "test_tool",
            "arguments": {
                "arg1": "value1"
            }
        }"#
    },
    )]
    fn test_extract_from_tag(#[case] text: &str, #[case] tag: &str, #[case] expected: &str) {
        let extracted = extract_from_tag(text, tag);
        assert_eq!(extracted, expected);
    }

    #[rstest]
    #[case(
        indoc! {r#"
            So I decided to call this because of that:

            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }"#
        },
        indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }"#
        }
    )]
    #[case(
        indoc! {r#"
            Here are some of the cities in East Asia:

            [
                {
                    "country": "South Korea",
                    "city": "Seoul",
                },
                {
                    "country": "Japan",
                    "city": "Tokyo",
                }
            ]"#
        },
        indoc! {r#"
            [
                {
                    "country": "South Korea",
                    "city": "Seoul",
                },
                {
                    "country": "Japan",
                    "city": "Tokyo",
                }
            ]"#
        }
    )]
    fn test_extract_json(#[case] text: &str, #[case] expected: &str) {
        let extracted = extract_json(text);
        assert_eq!(extracted, expected);
    }

    #[rstest]
    #[case(
        indoc! {r#"
            I think I should use the tool because of that.

            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            "#
        },
        indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            "#
        },
        Some("I think I should use the tool because of that.")
    )]
    #[case(
        indoc! {r#"
            I think I should use the tool because of that.

            ```json
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            ```
            "#
        },
        indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            "#
        },
        Some("I think I should use the tool because of that.")
    )]
    #[case(
        indoc! {r#"
            ```json
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            ```
            "#
        },
        indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            "#
        },
        None
    )]
    fn test_extract_thought(
        #[case] text: &str,
        #[case] json: &str,
        #[case] expected: Option<&str>,
    ) {
        let extracted = extract_thought(text, json);
        assert_eq!(extracted, expected);
    }
}
