pub mod policy;
pub mod scan;

use std::path::PathBuf;

use anyhow::Result;

#[derive(Debug)]
pub struct Element {
    pub path: PathBuf,
    pub size: u64,
    pub last_modified: jiff::Timestamp,
}

impl Element {
    pub fn new(path: PathBuf, size: u64, last_modified: jiff::Timestamp) -> Self {
        Element {
            path,
            size,
            last_modified,
        }
    }

    pub fn deletable() -> bool {
        todo!()
    }

    pub fn delete() -> Result<()> {
        todo!()
    }
}
