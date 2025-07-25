use std::borrow::Cow;

pub(crate) fn to_xyz_version(v_str: &str) -> Cow<'_, str> {
    let (version_num_part, rest) = v_str
        .find(['-', '+'])
        .map_or((v_str, ""), |i| v_str.split_at(i));

    let num_dots = version_num_part.matches('.').count();

    if num_dots == 1 {
        // x.y
        return Cow::Owned(format!("{version_num_part}.0{rest}"));
    }

    Cow::Borrowed(v_str)
}

pub fn is_invalid_identifier(s: &str) -> bool {
    s.is_empty() // Must not be empty
        || s.chars().all(char::is_whitespace) // Must contain some information
        || s.chars().any(|c|
               c == ':' // Must not contain the provider separator character
            || c == '/' // Must not contain the author/name separator character
            || c == '@' // Must not contain the version separator character
        )
}
