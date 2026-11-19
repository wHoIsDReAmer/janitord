use crate::core::{Element, kind::Kind};

use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::UNIX_EPOCH,
};

use anyhow::Result;
use jiff::Timestamp;
use rayon::Scope;

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

struct Found {
    path: PathBuf,
    kind: &'static Kind,
    tally: Arc<Tally>,
}

impl Found {
    fn into_element(self) -> Element {
        Element::new(
            self.path,
            self.kind,
            self.tally.size.load(Ordering::Relaxed),
            self.tally.last_modified(),
        )
    }
}

#[derive(Default)]
struct Findings(Mutex<Vec<Found>>);

impl Findings {
    fn push(&self, found: Found) {
        self.0.lock().unwrap().push(found);
    }

    fn into_elements(self) -> Result<Vec<Element>> {
        let found = self.0.into_inner()?;
        Ok(found.into_iter().map(Found::into_element).collect())
    }
}

pub fn scan(roots: &[PathBuf]) -> Result<Vec<Element>> {
    let findings = Findings::default();

    rayon::scope(|s| {
        for root in roots {
            let findings = &findings;
            s.spawn(move |s| discover(s, root.clone(), findings));
        }
    });

    findings.into_elements()
}

fn discover<'s>(s: &Scope<'s>, dir: PathBuf, findings: &'s Findings) {
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };

    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }

        let path = entry.path();
        if let Some(kind) = Kind::detect(&path) {
            let tally = Arc::new(Tally::default());

            if let Ok(meta) = entry.metadata() {
                tally
                    .last_modified
                    .store(unix_secs(&meta), Ordering::Relaxed);
            }

            findings.push(Found {
                path: path.clone(),
                kind,
                tally: Arc::clone(&tally),
            });
            s.spawn(move |s| measure(s, path, tally));
        } else {
            s.spawn(move |s| discover(s, path, findings));
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

fn unix_secs(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}
