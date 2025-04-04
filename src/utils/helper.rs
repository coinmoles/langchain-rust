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
