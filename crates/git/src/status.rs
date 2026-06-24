use git2::{Repository, Status};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitFileStatus {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitChangedFile {
    pub path: String,
    pub status: GitFileStatus,
}

pub fn status_files(repo_root: impl Into<PathBuf>) -> anyhow::Result<Vec<GitChangedFile>> {
    let repo = Repository::discover(repo_root.into())?;
    let statuses = repo.statuses(None)?;
    let mut files = Vec::new();

    for entry in statuses.iter() {
        let Some(path) = entry.path() else {
            continue;
        };
        let status = entry.status();
        let file_status = if status.intersects(Status::WT_DELETED | Status::INDEX_DELETED) {
            GitFileStatus::Deleted
        } else if status.intersects(Status::WT_NEW | Status::INDEX_NEW) {
            GitFileStatus::Added
        } else if status.intersects(Status::WT_MODIFIED | Status::INDEX_MODIFIED) {
            GitFileStatus::Modified
        } else {
            continue;
        };
        files.push(GitChangedFile {
            path: path.replace('\\', "/"),
            status: file_status,
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    files.dedup_by(|a, b| a.path == b.path);
    Ok(files)
}
