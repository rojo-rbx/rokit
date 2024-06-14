use serde::{Deserialize, Serialize};

use crate::result::RokitResult;

const ROKIT_META_TRAILER: [u8; 10] = *b"ROKIT_LINK";
const ROKIT_META_VERSION: u16 = 1;

const CARGO_VERSION: &str = env!("CARGO_PKG_VERSION");

// FUTURE: We could probably accept impl Read + Seek / impl Write instead
// of [u8] for the metadata functions here to make things faster. For now
// it is fine because the Rokit links are pretty small (just a few MB).

/**
    Metadata for a Rokit link - typically used for storing version
    information and skipping unnecessary work (recreating the link).

    Metadata, as bytes, is stored in the following
    format to make it easy to read and write:

    - Contents
    - Contents length (4 bytes)
    - Metadata version (2 bytes)
    - Metadata trailer (10 bytes)

    This makes for a full 16 bytes for our little
    metadata block + whatever contents it stores.
*/
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RokitLinkMetadata {
    pub(crate) version: String,
}

impl RokitLinkMetadata {
    /**
        Creates new metadata with the current version of Rokit.
    */
    pub(crate) fn current() -> Self {
        Self {
            version: CARGO_VERSION.to_string(),
        }
    }

    /**
        Checks if the metadata is for the current version of Rokit.
    */
    pub(crate) fn is_current(&self) -> bool {
        self.version == CARGO_VERSION
    }

    /**
        Parses metadata from the end of a file.
    */
    pub(crate) fn parse_from(contents: impl AsRef<[u8]>) -> Option<RokitLinkMetadata> {
        let contents = contents.as_ref();
        let len = contents.len();

        if contents.ends_with(&ROKIT_META_TRAILER) && len >= 16 {
            let meta_version = u16::from_le_bytes([contents[len - 12], contents[len - 11]]);
            let meta_len = u32::from_le_bytes([
                contents[len - 16],
                contents[len - 15],
                contents[len - 14],
                contents[len - 13],
            ]) as usize;
            if len < (16 + meta_len) {
                return None;
            }
            // FUTURE: Handle multiple metadata versions?
            if meta_version == ROKIT_META_VERSION {
                return postcard::from_bytes(&contents[(len - 16 - meta_len)..(len - 16)]).ok();
            }
        }

        None
    }

    /**
        Appends metadata to the end of a file.
    */
    pub(crate) fn append_to(&self, contents: impl Into<Vec<u8>>) -> RokitResult<Vec<u8>> {
        let mut contents = contents.into();

        let metadata_bytes = postcard::to_allocvec(self)?;
        let metadata_len =
            u32::try_from(metadata_bytes.len()).expect("rokit does not support metadata > 4GB");
        let metadata_version = ROKIT_META_VERSION.to_le_bytes();
        let metadata_len = metadata_len.to_le_bytes();

        contents.extend_from_slice(&metadata_bytes);
        contents.extend_from_slice(&metadata_len);
        contents.extend_from_slice(&metadata_version);
        contents.extend_from_slice(&ROKIT_META_TRAILER);

        Ok(contents)
    }
}
