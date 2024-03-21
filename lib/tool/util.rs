pub fn is_invalid_identifier(s: &str) -> bool {
    s.is_empty() // Must not be empty
        || s.chars().all(char::is_whitespace) // Must contain some information
        || s.chars().any(|c| c == '/') // Must not contain the separator character
}
