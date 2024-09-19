use std::path;

use anyhow::{anyhow, Context, Result};

#[macro_export]
macro_rules! context {
    () => {
        concat!(file!(), ":", line!(), ":", column!())
    };
}

pub fn must_write<P: AsRef<path::Path>>(path: P, content: &str) -> Result<()> {
    let parent = path
        .as_ref()
        .parent()
        .ok_or_else(|| anyhow!(""))
        .with_context(|| context!())?;
    std::fs::create_dir_all(parent).with_context(|| context!())?;

    std::fs::write(path, content).with_context(|| context!())?;
    Ok(())
}
