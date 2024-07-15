use std::path::Path;

const ALLOWED_EXTENSION_NAMES: [&str; 4] = ["zip", "tar", "gz", "tgz"];
const ALLOWED_EXTENSION_COUNT: usize = 2;

pub(super) fn split_filename_and_extensions(name: &str) -> (&str, Vec<&str>) {
    let mut path = Path::new(name);
    let mut exts = Vec::new();

    // Reverse-pop extensions off the path until we reach the
    // base name - we will then need to reverse afterwards, too
    while let Some(ext) = path.extension() {
        let ext = ext.to_str().expect("input was str");
        let stem = path.file_stem().expect("had an extension");

        if !ALLOWED_EXTENSION_NAMES
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
        {
            break;
        }

        exts.push(ext);
        path = Path::new(stem);

        if exts.len() >= ALLOWED_EXTENSION_COUNT {
            break;
        }
    }

    exts.reverse();

    let path = path.to_str().expect("input was str");
    (path, exts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_filename_ext_basic() {
        assert_eq!(
            split_filename_and_extensions("file.zip"),
            ("file", vec!["zip"])
        );
        assert_eq!(
            split_filename_and_extensions("file.tar"),
            ("file", vec!["tar"])
        );
        assert_eq!(
            split_filename_and_extensions("file.tar.gz"),
            ("file", vec!["tar", "gz"])
        );
        assert_eq!(
            split_filename_and_extensions("file.with.many.extensions.tar.gz.zip"),
            ("file.with.many.extensions.tar", vec!["gz", "zip"])
        );
        assert_eq!(
            split_filename_and_extensions("file.with.many.extensions.zip.gz.tar"),
            ("file.with.many.extensions.zip", vec!["gz", "tar"])
        );
        assert_eq!(
            split_filename_and_extensions("file.with.many.extensions.tar.gz"),
            ("file.with.many.extensions", vec!["tar", "gz"])
        );
    }

    #[test]
    fn split_filename_ext_real_tools() {
        assert_eq!(
            split_filename_and_extensions("wally-v0.3.2-linux.zip"),
            ("wally-v0.3.2-linux", vec!["zip"])
        );
        assert_eq!(
            split_filename_and_extensions("lune-0.8.6-macos-aarch64.zip"),
            ("lune-0.8.6-macos-aarch64", vec!["zip"])
        );
        assert_eq!(
            split_filename_and_extensions("just-1.31.0-aarch64-apple-darwin.tar.gz"),
            ("just-1.31.0-aarch64-apple-darwin", vec!["tar", "gz"])
        );
        assert_eq!(
            split_filename_and_extensions("sentry-cli-linux-i686-2.32.1.tgz"),
            ("sentry-cli-linux-i686-2.32.1", vec!["tgz"])
        );
    }
}
