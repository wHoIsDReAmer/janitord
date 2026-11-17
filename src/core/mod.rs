use std::path::Path;

use anyhow::Result;

pub mod condition;

pub struct Element<'s> {
    pub path: &'s Path,
    pub size: u64,
}

impl Element<'_> {
    pub fn deletable() -> bool {
        todo!()
    }

    pub fn delete() -> Result<()> {
        todo!()
    }
}
