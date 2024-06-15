mod auth;
mod rokit;

pub use self::auth::{AuthManifest, MANIFEST_FILE_NAME as AUTH_MANIFEST_FILE_NAME};
pub use self::rokit::{RokitManifest, MANIFEST_FILE_NAME as ROKIT_MANIFEST_FILE_NAME};

/**
    Helper function to make sure our authored manifest templates
    have consistent formatting and the correct repository URL.
*/
fn make_manifest_template(template: &'static str) -> String {
    let mut contents = unindent::unindent(template.trim())
        .replace("<|REPOSITORY_URL|>", env!("CARGO_PKG_REPOSITORY"));
    contents.push('\n');
    contents
}

// Let's also test the formatting a bit to make sure nothing slips through :-)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_no_indentation() {
        let auth_contents = make_manifest_template(auth::MANIFEST_DEFAULT_CONTENTS);
        let rokit_contents = make_manifest_template(rokit::MANIFEST_DEFAULT_CONTENTS);

        assert!(!auth_contents.contains('\t'));
        assert!(!rokit_contents.contains('\t'));

        assert!(!auth_contents.contains("\n  "));
        assert!(!rokit_contents.contains("\n  "));

        assert!(!auth_contents.contains("    "));
        assert!(!rokit_contents.contains("    "));
    }

    #[test]
    fn ends_with_newline() {
        assert!(make_manifest_template(auth::MANIFEST_DEFAULT_CONTENTS).ends_with('\n'));
        assert!(make_manifest_template(rokit::MANIFEST_DEFAULT_CONTENTS).ends_with('\n'));
    }

    #[test]
    fn contains_repo_url() {
        let auth_contents = make_manifest_template(auth::MANIFEST_DEFAULT_CONTENTS);
        let rokit_contents = make_manifest_template(rokit::MANIFEST_DEFAULT_CONTENTS);

        assert!(auth_contents.contains(env!("CARGO_PKG_REPOSITORY")));
        assert!(rokit_contents.contains(env!("CARGO_PKG_REPOSITORY")));

        assert!(!auth_contents.contains("REPOSITORY_URL"));
        assert!(!rokit_contents.contains("REPOSITORY_URL"));
    }
}
