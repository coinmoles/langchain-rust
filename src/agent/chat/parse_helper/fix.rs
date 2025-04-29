use std::collections::VecDeque;

use regex::Regex;

pub fn fix_text(text: &str) -> String {
    let re = Regex::new(r"\\(.)").unwrap();
    re.replace_all(text, |caps: &regex::Captures| match &caps[1] {
        "n" => "\n".to_owned(),
        "t" => "\t".to_owned(),
        "r" => "\r".to_owned(),
        other => other.to_owned(), // Leave unknown sequences unchanged
    })
    .to_string()
}

pub fn remove_multiline(s: &str) -> String {
    let mut cleaned = String::new();
    let mut inside_string = false;
    let mut escaped = false;

    for c in s.chars() {
        match c {
            '"' if !escaped => {
                inside_string = !inside_string;
                cleaned.push(c);
            }
            '\\' if inside_string => {
                escaped = !escaped;
                cleaned.push(c);
            }
            '\n' if inside_string => {
                cleaned.push_str("\\n");
            }
            _ => {
                escaped = false;
                cleaned.push(c);
            }
        }
    }

    cleaned
}

pub(super) fn remove_trailing_commas(s: &str) -> String {
    let mut cleaned = String::new();
    let mut chars = s.chars();
    let mut inside_string = false;
    let mut escaped = false;

    while let Some(c) = chars.next() {
        match c {
            '"' if !escaped => {
                inside_string = !inside_string;
                cleaned.push(c);
            }
            '\\' if inside_string => {
                escaped = !escaped;
                cleaned.push(c);
                continue;
            }
            ',' if !inside_string => {
                // Peek ahead for } or ]
                if let Some(next_non_ws) = chars.clone().find(|c| !c.is_whitespace()) {
                    if next_non_ws == '}' || next_non_ws == ']' {
                        // Skip this comma
                        continue;
                    }
                }
            }
            _ => {
                escaped = false;
                cleaned.push(c);
            }
        }
    }

    cleaned
}

pub fn balance_parenthesis(s: &str) -> String {
    let mut new_s = String::new();
    let mut stack: VecDeque<char> = VecDeque::new();
    let mut is_inside_string = false;
    let mut escaped = false;

    for char in s.chars() {
        match char {
            '"' if !escaped => is_inside_string = !is_inside_string,
            '{' if !is_inside_string => stack.push_back('}'),
            '[' if !is_inside_string => stack.push_back(']'),
            '}' | ']' if !is_inside_string => {
                if let Some(c) = stack.pop_back() {
                    if c != char {
                        return s.into(); // Mismatched closing character, return unmodified
                    }
                } else {
                    return s.into(); // Unbalanced closing character, return unmodified
                }
            }
            '\\' if is_inside_string => escaped = !escaped,
            _ => escaped = false,
        }
        new_s.push(char);
    }

    // Close any open structures.
    while let Some(c) = stack.pop_back() {
        new_s.push(c);
    }

    new_s
}
