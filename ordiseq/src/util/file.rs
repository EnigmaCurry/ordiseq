pub fn make_filename(input: &str, extension: &str) -> String {
    let safe_name: String = input
        .chars()
        .map(|c| {
            if c.is_whitespace() {
                '_'
            } else if c == '/' {
                '-'
            } else {
                c
            }
        })
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
        .collect();

    if extension.is_empty() {
        safe_name
    } else {
        format!(
            "{}.{}",
            safe_name.trim_end_matches('.'),
            extension.trim_start_matches('.')
        )
    }
}
