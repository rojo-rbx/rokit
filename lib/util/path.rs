#![allow(dead_code)] // Some path utilities are only used on Windows

use std::path::{Path, PathBuf};

/**
    Splits a filename into its base name and a list of extensions.

    This is useful for handling files with multiple extensions, such as `file-name.ext1.ext2`.

    # Example

    ```rust ignore
    let (name, exts) = split_filename_and_extensions("file-name.ext1.ext2");
    assert_eq!(name, "file-name");
    assert_eq!(exts, vec!["ext1", "ext2"]);
    ```
*/
pub(crate) fn split_filename_and_extensions(name: &str) -> (&str, Vec<&str>) {
    let mut path = Path::new(name);
    let mut exts = Vec::new();

    // Reverse-pop extensions off the path until we reach the
    // base name - we will then need to reverse afterwards, too
    while let Some(ext) = path.extension() {
        exts.push(ext.to_str().expect("input was str"));
        path = Path::new(path.file_stem().expect("had an extension"));
    }
    exts.reverse();

    let path = path.to_str().expect("input was str");
    (path, exts)
}

/**
    Cleans up a path and simplifies it for writing to storage or environment variables.

    This will currently:

    - De-UNC a path, removing the `\\?\` prefix
*/
pub fn simplify_path(path: impl AsRef<Path>) -> PathBuf {
    dunce::simplified(path.as_ref()).to_path_buf()
}
