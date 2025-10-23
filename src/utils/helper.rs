/// Normalize the tool names to avoid incorrect matches.
pub fn normalize_tool_name(name: &str) -> String {
    name.to_lowercase().replace(" ", "_")
}

/// Add indent to each line of the string.
pub fn add_indent(s: &str, indent: usize, indent_first_line: bool) -> String {
    let indent_str = " ".repeat(indent);
    s.lines()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 && !indent_first_line {
                line.into()
            } else {
                format!("{indent_str}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Capitalize the first letter of the string.
pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Adds two optional numbers, treating `None` as zero.
pub fn add_option_numbers<T>(a: Option<T>, b: Option<T>) -> Option<T>
where
    T: std::ops::Add<Output = T> + Default + Copy,
{
    match (a, b) {
        (None, None) => None,
        _ => Some(a.unwrap_or_default() + b.unwrap_or_default()),
    }
}

pub const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";
