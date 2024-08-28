use std::{fmt, str::FromStr};

use super::util::split_filename_and_extensions;

/**
    An artifact format supported by Rokit.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactFormat {
    TarGz,
    Tar,
    Zip,
    Gz,
}

impl ArtifactFormat {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::TarGz => "tar.gz",
            Self::Gz => "gz",
        }
    }

    #[must_use]
    pub fn from_extensions<'a>(extensions: impl AsRef<[&'a str]>) -> Option<Self> {
        match extensions.as_ref() {
            [.., ext] if ext.eq_ignore_ascii_case("zip") => Some(Self::Zip),
            [.., ext] if ext.eq_ignore_ascii_case("tar") => Some(Self::Tar),
            [.., ext] if ext.eq_ignore_ascii_case("tgz") => Some(Self::TarGz),
            [.., ext1, ext2]
                if ext1.eq_ignore_ascii_case("tar") && ext2.eq_ignore_ascii_case("gz") =>
            {
                Some(Self::TarGz)
            }
            [.., ext] if ext.eq_ignore_ascii_case("gz") => Some(Self::Gz),
            _ => None,
        }
    }

    #[must_use]
    pub fn from_path_or_url(path_or_url: impl AsRef<str>) -> Option<Self> {
        let path_or_url = path_or_url.as_ref();
        let (_, extensions) = split_filename_and_extensions(path_or_url);
        Self::from_extensions(extensions)
    }
}

impl FromStr for ArtifactFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let l = s.trim().to_lowercase();
        match l.as_str() {
            "zip" => Ok(Self::Zip),
            "tar" => Ok(Self::Tar),
            "tar.gz" | "tgz" => Ok(Self::TarGz),
            _ => Err(format!("unknown artifact format '{l}'")),
        }
    }
}

impl fmt::Display for ArtifactFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_from_str(s: &str) -> Option<ArtifactFormat> {
        let (_, extensions) = split_filename_and_extensions(s);
        ArtifactFormat::from_extensions(extensions)
    }

    #[test]
    fn format_from_extensions_valid() {
        assert_eq!(format_from_str("file.zip"), Some(ArtifactFormat::Zip));
        assert_eq!(format_from_str("file.tar"), Some(ArtifactFormat::Tar));
        assert_eq!(format_from_str("file.tar.gz"), Some(ArtifactFormat::TarGz));
        assert_eq!(format_from_str("file.tgz"), Some(ArtifactFormat::TarGz));
        assert_eq!(format_from_str("file.gz"), Some(ArtifactFormat::Gz));
        assert_eq!(
            format_from_str("file.with.many.extensions.tar.gz.zip"),
            Some(ArtifactFormat::Zip)
        );
        assert_eq!(
            format_from_str("file.with.many.extensions.zip.gz.tar"),
            Some(ArtifactFormat::Tar)
        );
        assert_eq!(
            format_from_str("file.with.many.extensions.tar.gz"),
            Some(ArtifactFormat::TarGz)
        );
    }

    #[test]
    fn format_from_extensions_invalid() {
        assert_eq!(format_from_str("file-name"), None);
        assert_eq!(format_from_str("some/file.exe"), None);
        assert_eq!(format_from_str("really.long.file.name"), None);
    }

    #[test]
    fn format_from_real_tools() {
        assert_eq!(
            format_from_str("wally-v0.3.2-linux.zip"),
            Some(ArtifactFormat::Zip)
        );
        assert_eq!(
            format_from_str("lune-0.8.6-macos-aarch64.zip"),
            Some(ArtifactFormat::Zip)
        );
        assert_eq!(
            format_from_str("just-1.31.0-aarch64-apple-darwin.tar.gz"),
            Some(ArtifactFormat::TarGz)
        );
        assert_eq!(
            format_from_str("sentry-cli-linux-i686-2.32.1.tgz"),
            Some(ArtifactFormat::TarGz)
        );
        assert_eq!(
            format_from_str("lefthook_1.7.14_Windows_x86_64.gz"),
            Some(ArtifactFormat::Gz)
        );
    }

    #[test]
    fn format_case_sensitivity() {
        assert_eq!(format_from_str("file.ZIP"), Some(ArtifactFormat::Zip));
        assert_eq!(format_from_str("file.zip"), Some(ArtifactFormat::Zip));
        assert_eq!(format_from_str("file.Zip"), Some(ArtifactFormat::Zip));
        assert_eq!(format_from_str("file.tar"), Some(ArtifactFormat::Tar));
        assert_eq!(format_from_str("file.TAR"), Some(ArtifactFormat::Tar));
        assert_eq!(format_from_str("file.Tar"), Some(ArtifactFormat::Tar));
        assert_eq!(format_from_str("file.tar.gz"), Some(ArtifactFormat::TarGz));
        assert_eq!(format_from_str("file.TAR.GZ"), Some(ArtifactFormat::TarGz));
        assert_eq!(format_from_str("file.Tar.Gz"), Some(ArtifactFormat::TarGz));
        assert_eq!(format_from_str("file.tgz"), Some(ArtifactFormat::TarGz));
        assert_eq!(format_from_str("file.TGZ"), Some(ArtifactFormat::TarGz));
        assert_eq!(format_from_str("file.Tgz"), Some(ArtifactFormat::TarGz));
        assert_eq!(format_from_str("file.gz"), Some(ArtifactFormat::Gz));
        assert_eq!(format_from_str("file.GZ"), Some(ArtifactFormat::Gz));
        assert_eq!(format_from_str("file.Gz"), Some(ArtifactFormat::Gz));
    }

    #[test]
    fn test_ordering() {
        let artifact_names = [
            "tool-v1.0.0-x86_64-linux.zip",
            "tool-v1.0.0-x86_64-linux.tar",
            "tool-v1.0.0-x86_64-linux.tar.gz",
            "tool-v1.0.0-x86_64-linux.gz",
            "tool-v1.0.0-x86_64-linux",
            "tool-v1.0.0-x86_64-linux.elf",
        ];

        let mut artifact_formats: Vec<Option<ArtifactFormat>> = artifact_names
            .iter()
            .map(|name| format_from_str(name))
            .collect();

        artifact_formats.sort();

        assert_eq!(
            artifact_formats,
            vec![
                None,
                None,
                Some(ArtifactFormat::TarGz),
                Some(ArtifactFormat::Tar),
                Some(ArtifactFormat::Zip),
                Some(ArtifactFormat::Gz),
            ]
        );
    }
}
