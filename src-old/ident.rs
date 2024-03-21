use anyhow::ensure;

/// Ensures that the given ident fits the rules that we set for Aftman, with
/// nice error reporting.
pub fn check_ident(ident_type: &str, ident: &str) -> anyhow::Result<()> {
    ensure!(!ident.is_empty(), "{} must be non-empty", ident_type);
    ensure!(
        !ident.chars().all(char::is_whitespace),
        "{} must be non-empty",
        ident_type
    );
    ensure!(
        ident.chars().all(|c| c != '/'),
        "{} must not contain a slash",
        ident_type
    );

    Ok(())
}
