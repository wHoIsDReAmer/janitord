use std::{
    fmt,
    fs::{self, File, TryLockError},
    io,
    path::{Path, PathBuf},
};

static KINDS: &[Kind] = &[
    Kind {
        name: "cargo",
        dir: "target",
        markers: &["Cargo.toml"],
        locks: &[Locks {
            base: Base::Artifact,
            dirs: &["*", "*/*"],
            files: LockFiles::Named(&[".cargo-lock", ".cargo-build-lock", ".cargo-artifact-lock"]),
        }],
    },
    Kind {
        name: "node",
        dir: "node_modules",
        markers: &["package.json"],
        locks: &[],
    },
    Kind {
        name: "gradle",
        dir: "build",
        markers: &["build.gradle", "build.gradle.kts"],
        locks: &[Locks {
            base: Base::RootWith(&["settings.gradle", "settings.gradle.kts"]),
            dirs: &[".gradle/*", ".gradle/*/*"],
            files: LockFiles::Extension("lock"),
        }],
    },
];

pub struct Kind {
    pub name: &'static str,
    dir: &'static str,
    markers: &'static [&'static str],
    locks: &'static [Locks],
}

struct Locks {
    base: Base,
    dirs: &'static [&'static str],
    files: LockFiles,
}

enum Base {
    Artifact,
    RootWith(&'static [&'static str]),
}

enum LockFiles {
    Named(&'static [&'static str]),
    Extension(&'static str),
}

impl Kind {
    pub fn detect(dir: &Path) -> Option<&'static Kind> {
        let (name, project) = (dir.file_name()?, dir.parent()?);

        KINDS
            .iter()
            .find(|kind| name == kind.dir && kind.markers.iter().any(|m| project.join(m).is_file()))
    }

    pub fn lock(&self, artifact: &Path) -> io::Result<Option<Vec<File>>> {
        let mut held = Vec::new();

        for locks in self.locks {
            for path in locks.find(artifact)? {
                let file = File::open(&path)?;
                match file.try_lock() {
                    Ok(()) => held.push(file),
                    Err(TryLockError::WouldBlock) => return Ok(None),
                    Err(TryLockError::Error(err)) => return Err(err),
                }
            }
        }

        Ok(Some(held))
    }
}

impl fmt::Debug for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.name)
    }
}

impl Locks {
    fn find(&self, artifact: &Path) -> io::Result<Vec<PathBuf>> {
        let base = self.base.resolve(artifact);
        let mut found = Vec::new();
        for pattern in self.dirs {
            for dir in expand(&base, pattern)? {
                self.files.find_in(&dir, &mut found)?;
            }
        }
        Ok(found)
    }
}

impl Base {
    fn resolve(&self, artifact: &Path) -> PathBuf {
        match self {
            Base::Artifact => artifact.to_path_buf(),
            Base::RootWith(markers) => {
                let project = artifact.parent().unwrap_or(artifact);
                project
                    .ancestors()
                    .find(|dir| markers.iter().any(|m| dir.join(m).is_file()))
                    .unwrap_or(project)
                    .to_path_buf()
            }
        }
    }
}

impl LockFiles {
    fn find_in(&self, dir: &Path, found: &mut Vec<PathBuf>) -> io::Result<()> {
        match self {
            LockFiles::Named(names) => {
                found.extend(
                    names
                        .iter()
                        .map(|name| dir.join(name))
                        .filter(|path| path.is_file()),
                );
            }
            LockFiles::Extension(ext) => {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if entry.file_type()?.is_file() && path.extension().is_some_and(|e| e == *ext) {
                        found.push(path);
                    }
                }
            }
        }
        Ok(())
    }
}

fn expand(base: &Path, pattern: &str) -> io::Result<Vec<PathBuf>> {
    let mut dirs = vec![base.to_path_buf()];
    for segment in pattern.split('/') {
        let mut next = Vec::new();
        for dir in &dirs {
            if segment == "*" {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        next.push(entry.path());
                    }
                }
            } else {
                let path = dir.join(segment);
                if fs::symlink_metadata(&path).is_ok_and(|meta| meta.is_dir()) {
                    next.push(path);
                }
            }
        }
        dirs = next;
    }
    Ok(dirs)
}
