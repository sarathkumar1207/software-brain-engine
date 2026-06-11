use sbe_common::FileEntry;
use std::path::{Component, Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Scanner {
    root: PathBuf,
    extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    pub files: Vec<FileEntry>,
    pub skipped_dirs: Vec<String>,
    pub warnings: Vec<String>,
    pub bytes_read: u64,
    pub elapsed_ms: u128,
}

impl Scanner {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            extensions: vec!["ts".into(), "tsx".into(), "py".into()],
        }
    }

    pub fn with_extensions(
        mut self,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.extensions = extensions.into_iter().map(Into::into).collect();
        self
    }

    pub fn scan(&self) -> anyhow::Result<Vec<FileEntry>> {
        Ok(self.scan_with_report()?.files)
    }

    pub fn scan_with_report(&self) -> anyhow::Result<ScanReport> {
        let started = Instant::now();
        let mut entries = Vec::new();
        let mut skipped_dirs = Vec::new();
        let mut warnings = Vec::new();
        let mut bytes_read = 0_u64;

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|entry| {
                let ignored = entry.path() != self.root && is_ignored_path(entry.path());
                if ignored && entry.file_type().is_dir() {
                    skipped_dirs.push(entry.path().to_string_lossy().to_string());
                }
                !ignored
            })
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| self.is_source(entry.path()))
        {
            let path = entry.path();
            let bytes = match std::fs::read(path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    warnings.push(format!("failed to read {}: {error}", path.display()));
                    continue;
                }
            };
            bytes_read += bytes.len() as u64;
            let hash = blake3::hash(&bytes).to_hex().to_string();
            let relative_path = path
                .strip_prefix(&self.root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default()
                .to_string();

            entries.push(FileEntry {
                id: stable_id(&relative_path),
                path: path.to_string_lossy().to_string(),
                relative_path,
                hash,
                extension,
            });
        }

        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        skipped_dirs.sort();
        skipped_dirs.dedup();
        Ok(ScanReport {
            files: entries,
            skipped_dirs,
            warnings,
            bytes_read,
            elapsed_ms: started.elapsed().as_millis(),
        })
    }

    fn is_source(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.extensions.iter().any(|allowed| allowed == ext))
            .unwrap_or(false)
    }
}

fn stable_id(value: &str) -> u64 {
    let hash = blake3::hash(value.as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[0..8]);
    u64::from_le_bytes(bytes)
}

fn is_ignored_path(path: &Path) -> bool {
    const IGNORED: &[&str] = &[
        ".git",
        ".sbe",
        "node_modules",
        "target",
        "dist",
        "build",
        "coverage",
        ".next",
        ".turbo",
    ];

    path.components().any(|component| match component {
        Component::Normal(name) => name
            .to_str()
            .map(|name| IGNORED.contains(&name))
            .unwrap_or(false),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_supported_languages_and_ignores_generated_folders() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::create_dir_all(temp.path().join("node_modules/pkg")).unwrap();
        std::fs::write(temp.path().join("src/app.ts"), "export function app() {}").unwrap();
        std::fs::write(
            temp.path().join("src/view.tsx"),
            "export const View = () => <div />",
        )
        .unwrap();
        std::fs::write(temp.path().join("src/service.py"), "def service(): pass").unwrap();
        std::fs::write(temp.path().join("src/readme.md"), "# no").unwrap();
        std::fs::write(temp.path().join("node_modules/pkg/index.ts"), "ignored").unwrap();

        let report = Scanner::new(temp.path()).scan_with_report().unwrap();

        assert_eq!(report.files.len(), 3);
        assert!(report
            .files
            .iter()
            .any(|file| file.relative_path == "src/app.ts"));
        assert!(report
            .files
            .iter()
            .any(|file| file.relative_path == "src/view.tsx"));
        assert!(report
            .files
            .iter()
            .any(|file| file.relative_path == "src/service.py"));
        assert!(report
            .skipped_dirs
            .iter()
            .any(|path| path.contains("node_modules")));
    }

    #[test]
    fn hash_changes_when_file_changes() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("a.ts");
        std::fs::write(&file, "one").unwrap();
        let first = Scanner::new(temp.path()).scan().unwrap().remove(0);
        std::fs::write(&file, "two").unwrap();
        let second = Scanner::new(temp.path()).scan().unwrap().remove(0);

        assert_eq!(first.id, second.id);
        assert_ne!(first.hash, second.hash);
    }
}
