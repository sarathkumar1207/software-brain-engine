use anyhow::Context;
use sbe_common::{IndexSnapshot, STORAGE_VERSION};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const INDEX_FILE: &str = "index.bin";
const JSON_EXPORT_FILE: &str = "snapshots/current.json";

#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageManifest {
    pub name: String,
    pub storage_version: u32,
    pub index_file: String,
    pub format: String,
}

impl Store {
    pub fn open(repo_root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = repo_root.as_ref().join(".sbe");
        std::fs::create_dir_all(root.join("snapshots"))?;
        std::fs::create_dir_all(root.join("objects"))?;
        std::fs::create_dir_all(root.join("reports"))?;
        let store = Self { root };
        store.write_manifest()?;
        Ok(store)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn index_path(&self) -> PathBuf {
        self.root.join(INDEX_FILE)
    }

    pub fn index_size(&self) -> anyhow::Result<u64> {
        Ok(std::fs::metadata(self.index_path())?.len())
    }

    pub fn has_index(&self) -> bool {
        self.index_path().exists()
    }

    pub fn write_snapshot(&self, snapshot: &IndexSnapshot) -> anyhow::Result<()> {
        let path = self.index_path();
        let temp_path = self.root.join("index.bin.tmp");
        let bytes = bincode::serde::encode_to_vec(snapshot, bincode::config::standard())?;
        std::fs::write(&temp_path, bytes)
            .with_context(|| format!("failed to write {}", temp_path.display()))?;
        std::fs::rename(&temp_path, &path).with_context(|| {
            format!(
                "failed to atomically replace {} with {}",
                temp_path.display(),
                path.display()
            )
        })
    }

    pub fn read_snapshot(&self) -> anyhow::Result<IndexSnapshot> {
        let path = self.index_path();
        let bytes =
            std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let (snapshot, _): (IndexSnapshot, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                .with_context(|| format!("failed to decode {}", path.display()))?;
        if snapshot.storage_version != STORAGE_VERSION {
            anyhow::bail!(
                "unsupported .sbe storage version {} (expected {}). Run `sbe scan` with a compatible binary or remove .sbe to rebuild the index.",
                snapshot.storage_version,
                STORAGE_VERSION
            );
        }
        Ok(snapshot)
    }

    pub fn read_snapshot_or_empty(
        &self,
        repo_root: impl AsRef<Path>,
    ) -> anyhow::Result<IndexSnapshot> {
        if self.index_path().exists() {
            self.read_snapshot()
        } else {
            Ok(IndexSnapshot::empty(
                repo_root.as_ref().to_string_lossy().to_string(),
            ))
        }
    }

    pub fn export_snapshot_json(&self) -> anyhow::Result<PathBuf> {
        let snapshot = self.read_snapshot()?;
        let path = self.root.join(JSON_EXPORT_FILE);
        let bytes = serde_json::to_vec_pretty(&snapshot)?;
        std::fs::write(&path, bytes)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(path)
    }

    pub fn write_report_json<T: Serialize>(
        &self,
        name: &str,
        report: &T,
    ) -> anyhow::Result<PathBuf> {
        let path = self.root.join("reports").join(name);
        let bytes = serde_json::to_vec_pretty(report)?;
        std::fs::write(&path, bytes)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(path)
    }

    fn write_manifest(&self) -> anyhow::Result<()> {
        let path = self.root.join("manifest.json");
        let manifest = StorageManifest {
            name: "Software Brain Engine".into(),
            storage_version: STORAGE_VERSION,
            index_file: INDEX_FILE.into(),
            format: "bincode-serde-v2".into(),
        };
        std::fs::write(&path, serde_json::to_vec_pretty(&manifest)?)
            .with_context(|| format!("failed to write {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::{FileEntry, IndexSnapshot};

    #[test]
    fn writes_and_reads_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let store = Store::open(temp.path()).unwrap();
        let mut snapshot = IndexSnapshot::empty(temp.path().to_string_lossy());
        snapshot.files.push(FileEntry {
            id: 1,
            path: "src/a.ts".into(),
            relative_path: "src/a.ts".into(),
            hash: "hash".into(),
            extension: "ts".into(),
        });

        store.write_snapshot(&snapshot).unwrap();
        let read = store.read_snapshot().unwrap();

        assert_eq!(read.files.len(), 1);
        assert_eq!(read.storage_version, STORAGE_VERSION);
        assert!(store.index_path().exists());
        assert!(!store.root().join("index.bin.tmp").exists());
    }

    #[test]
    fn exports_debug_json_from_binary_index() {
        let temp = tempfile::tempdir().unwrap();
        let store = Store::open(temp.path()).unwrap();
        store
            .write_snapshot(&IndexSnapshot::empty(temp.path().to_string_lossy()))
            .unwrap();

        let json_path = store.export_snapshot_json().unwrap();

        assert!(json_path.exists());
        assert!(std::fs::read_to_string(json_path)
            .unwrap()
            .contains("storage_version"));
    }

    #[test]
    fn rejects_unsupported_storage_version() {
        let temp = tempfile::tempdir().unwrap();
        let store = Store::open(temp.path()).unwrap();
        let mut snapshot = IndexSnapshot::empty(temp.path().to_string_lossy());
        snapshot.storage_version = STORAGE_VERSION + 1;
        let bytes = bincode::serde::encode_to_vec(&snapshot, bincode::config::standard()).unwrap();
        std::fs::write(store.index_path(), bytes).unwrap();

        let error = store.read_snapshot().unwrap_err().to_string();

        assert!(error.contains("unsupported .sbe storage version"));
    }
}
