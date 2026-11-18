use crate::core::Element;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::Result;
use rayon::Scope;

const DEPS: &[(&str, &str)] = &[("target", "Cargo.toml"), ("node_modules", "package.json")];

type Found = Mutex<Vec<(PathBuf, Arc<AtomicU64>)>>;

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
        .map(|(path, size)| Element::new(path, size.load(Ordering::Relaxed)))
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
            let size = Arc::new(AtomicU64::new(0));
            found
                .lock()
                .unwrap()
                .push((path.clone(), Arc::clone(&size)));
            s.spawn(move |s| measure(s, path, size));
        } else {
            s.spawn(move |s| discover(s, path, found));
        }
    }
}

fn measure<'s>(s: &Scope<'s>, dir: PathBuf, size: Arc<AtomicU64>) {
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };

    let mut local = 0;

    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            let (path, size) = (entry.path(), Arc::clone(&size));
            s.spawn(move |s| measure(s, path, size));
        } else {
            local += meta.len();
        }
    }

    size.fetch_add(local, Ordering::Relaxed);
}

fn is_artifact(dir: &Path) -> bool {
    let (Some(name), Some(parent)) = (dir.file_name(), dir.parent()) else {
        return false;
    };

    DEPS.iter()
        .any(|(dir_name, marker)| name == *dir_name && parent.join(marker).is_file())
}
