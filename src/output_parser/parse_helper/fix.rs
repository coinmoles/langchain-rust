use std::collections::VecDeque;

use regex::Regex;

/// Fixes escape sequence issues in the input text.
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

/// Removes newline characters within JSON strings by replacing them with `\n`.
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

/// Removes trailing commas before closing braces or brackets.
pub fn remove_trailing_commas(s: &str) -> String {
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
                cleaned.push(c);
            }
            _ => {
                escaped = false;
                cleaned.push(c);
            }
        }
    }

    cleaned
}

/// Balances unclosed JSON structures by adding the necessary closing braces or brackets.
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        r#"This is a test\nNew line\tTabbed\rCarriage return"#,
        "This is a test\nNew line\tTabbed\rCarriage return"
    )]
    fn test_fix_text(#[case] input: &str, #[case] expected: &str) {
        let fixed = fix_text(input);
        assert_eq!(fixed, expected);
    }

    #[rstest]
    #[case(
        indoc! {r#"
            {
                "key1": "This is a malformed JSON string.
            The newline within the string is incorrectly parsed as an actual newline.",
                "key2": "value2"
            }"#
        },
        indoc! {r#"
            {
                "key1": "This is a malformed JSON string.\nThe newline within the string is incorrectly parsed as an actual newline.",
                "key2": "value2"
            }"#
        }
    )]
    fn test_remove_multiline(#[case] input: &str, #[case] expected: &str) {
        let cleaned = remove_multiline(input);
        assert_eq!(cleaned, expected);
    }

    #[rstest]
    #[case(
        indoc! {r#"
            {
                "key1": "value1",
                "key2": "value2",
            }"#
        },
        indoc! {r#"
            {
                "key1": "value1",
                "key2": "value2"
            }"#
        }
    )]
    #[case(r#"["item1", "item2",]"#, r#"["item1", "item2"]"#)]
    fn test_remove_trailing_commas(#[case] input: &str, #[case] expected: &str) {
        let cleaned = remove_trailing_commas(input);
        assert_eq!(cleaned, expected);
    }

    #[rstest]
    #[case(
        r#"{"key1": "value1", "key2": ["item1", "item2""#,
        r#"{"key1": "value1", "key2": ["item1", "item2"]}"#
    )]
    fn test_balance_parenthesis(#[case] input: &str, #[case] expected: &str) {
        let balanced = balance_parenthesis(input);
        assert_eq!(balanced, expected);
    }
}
