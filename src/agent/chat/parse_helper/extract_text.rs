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

pub fn extract_from_tag(text: &str) -> &str {
    let re = Regex::new(r"^<\w+>\s*([\s\S]+?)\s*</\w+>$").unwrap();
    if let Some(caps) = re.captures(text) {
        if let Some(tool_call_str) = caps.get(1) {
            return tool_call_str.as_str().trim();
        }
    }
    text
}
