pub mod policy;
pub mod scan;

use std::path::PathBuf;

use anyhow::Result;
use jiff::ZonedDateTime;

use crate::core::policy::Policy;

#[derive(Debug)]
pub struct Element {
    pub path: PathBuf,
    pub size: u64,
    pub modified: jiff::Timestamp,
}

impl Element {
    pub fn new(path: PathBuf, size: u64, modified: jiff::Timestamp) -> Self {
        Element {
            path,
            size,
            modified,
        }
    }

    pub fn deletable(&self, policy: &Policy, now: &ZonedDateTime) -> Result<bool> {
        policy.matches(self.size, self.modified, now)
    }

    pub fn delete() -> Result<()> {
        todo!()
    }
}
