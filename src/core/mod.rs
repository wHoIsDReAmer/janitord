use std::path::PathBuf;

use anyhow::Result;

pub mod policy;

#[derive(Debug)]
pub struct Element {
    pub path: PathBuf,
    pub size: u64,
}

impl Element {
    pub fn new(path: PathBuf, size: u64) -> Self {
        Element { path, size }
    }

    pub fn deletable() -> bool {
        todo!()
    }

    pub fn delete() -> Result<()> {
        todo!()
    }
}
