use crate::core::Element;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::UNIX_EPOCH,
};

use anyhow::Result;
use jiff::Timestamp;
use rayon::Scope;

const DEPS: &[(&str, &str)] = &[("target", "Cargo.toml"), ("node_modules", "package.json")];

#[derive(Default)]
struct Tally {
    size: AtomicU64,
    last_modified: AtomicU64,
}

impl Tally {
    fn last_modified(&self) -> Timestamp {
        i64::try_from(self.last_modified.load(Ordering::Relaxed))
            .ok()
            .and_then(|s| Timestamp::from_second(s).ok())
            .unwrap_or(Timestamp::MAX)
    }
}

type Found = Mutex<Vec<(PathBuf, Arc<Tally>)>>;

pub fn scan(roots: &[PathBuf]) -> Result<Vec<Element>> {
    let found = Found::default();

    rayon::scope(|s| {
        for root in roots {
            let found = &found;
            s.spawn(move |s| discover(s, root.clone(), found));
        }
    });

    Ok(found
        .into_inner()?
        .into_iter()
        .map(|(path, t)| Element::new(path, t.size.load(Ordering::Relaxed), t.last_modified()))
        .collect())
}

fn discover<'s>(s: &Scope<'s>, dir: PathBuf, found: &'s Found) {
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };

    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }

        let path = entry.path();
        if is_artifact(&path) {
            let tally = Arc::new(Tally::default());

            if let Ok(meta) = entry.metadata() {
                tally
                    .last_modified
                    .store(unix_secs(&meta), Ordering::Relaxed);
            }

            found
                .lock()
                .unwrap()
                .push((path.clone(), Arc::clone(&tally)));
            s.spawn(move |s| measure(s, path, tally));
        } else {
            s.spawn(move |s| discover(s, path, found));
        }
    }
}

fn measure<'s>(s: &Scope<'s>, dir: PathBuf, tally: Arc<Tally>) {
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };

    let mut size = 0;
    let mut newest = 0;

    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        newest = newest.max(unix_secs(&meta));

        if meta.is_dir() {
            let (path, tally) = (entry.path(), Arc::clone(&tally));
            s.spawn(move |s| measure(s, path, tally));
        } else {
            size += meta.len();
        }
    }

    tally.size.fetch_add(size, Ordering::Relaxed);
    tally.last_modified.fetch_max(newest, Ordering::Relaxed);
}

fn is_artifact(dir: &Path) -> bool {
    let (Some(name), Some(parent)) = (dir.file_name(), dir.parent()) else {
        return false;
    };

    DEPS.iter()
        .any(|(dir_name, marker)| name == *dir_name && parent.join(marker).is_file())
}

fn unix_secs(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}
