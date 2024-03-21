#![allow(unused)]

use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use anyhow::bail;

pub fn write_only_new(path: &Path, contents: &str) -> anyhow::Result<()> {
    let mut file = match OpenOptions::new().create_new(true).write(true).open(path) {
        Ok(file) => BufWriter::new(file),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                bail!("File {} already exists.", path.display());
            }

            bail!(err);
        }
    };

    file.write_all(contents.as_bytes())?;
    file.into_inner()?;

    Ok(())
}

pub fn write_if_not_exists(path: &Path, contents: &str) -> anyhow::Result<()> {
    let mut file = match OpenOptions::new().create_new(true).write(true).open(path) {
        Ok(file) => BufWriter::new(file),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                return Ok(());
            }

            bail!(err);
        }
    };

    file.write_all(contents.as_bytes())?;
    file.into_inner()?;

    Ok(())
}
