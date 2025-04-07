use serde::de::Unexpected;
use serde_json::Value;

pub fn add_indent(s: &str, indent: usize, indent_first_line: bool) -> String {
    let indent_str = " ".repeat(indent);
    s.lines()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 && !indent_first_line {
                line.into()
            } else {
                format!("{}{}", indent_str, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn to_unexpected(value: &Value) -> Unexpected {
    match value {
        Value::Null => Unexpected::Unit,
        Value::Bool(b) => Unexpected::Bool(*b),
        Value::Number(n) => Unexpected::Signed(n.as_i64().unwrap_or(0)),
        Value::String(s) => Unexpected::Str(s),
        Value::Array(_) => Unexpected::Seq,
        Value::Object(_) => Unexpected::Map,
    }
}
