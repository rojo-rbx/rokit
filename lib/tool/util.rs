pub fn is_invalid_identifier(s: &str) -> bool {
    s.is_empty() // Must not be empty
        || s.chars().all(char::is_whitespace) // Must contain some information
        || s.chars().any(|c|
               c == ':' // Must not contain the provider separator character
            || c == '/' // Must not contain the author/name separator character
            || c == '@' // Must not contain the version separator character
        )
}
