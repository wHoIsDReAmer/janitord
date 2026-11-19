pub mod kind;
pub mod policy;
pub mod scan;

use std::{fs, path::PathBuf};

use anyhow::Result;
use jiff::ZonedDateTime;

use crate::core::{kind::Kind, policy::Policy};

#[derive(Debug)]
pub struct Element {
    pub path: PathBuf,
    pub kind: &'static Kind,
    pub size: u64,
    pub modified: jiff::Timestamp,
}

impl Element {
    pub fn new(path: PathBuf, kind: &'static Kind, size: u64, modified: jiff::Timestamp) -> Self {
        Element {
            path,
            kind,
            size,
            modified,
        }
    }

    pub fn deletable(&self, policy: &Policy, now: &ZonedDateTime) -> Result<bool> {
        policy.matches(self.size, self.modified, now)
    }

    pub fn delete(&self) -> Result<bool> {
        if !fs::symlink_metadata(&self.path)?.is_dir() {
            return Ok(false);
        }

        let Some(locks) = self.kind.lock(&self.path)? else {
            return Ok(false);
        };

        let trash = self.trash_path();
        fs::rename(&self.path, &trash)?;
        drop(locks);

        fs::remove_dir_all(&trash)?;
        Ok(true)
    }

    fn trash_path(&self) -> PathBuf {
        let mut name = self.path.file_name().unwrap_or_default().to_os_string();
        name.push(format!(".janitord-deleting-{}", std::process::id()));
        self.path.with_file_name(name)
    }
}
