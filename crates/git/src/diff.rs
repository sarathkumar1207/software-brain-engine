use crate::status::status_files;
pub use crate::status::{GitChangedFile, GitFileStatus};
use git2::{Delta, DiffOptions, Repository};
use std::path::PathBuf;

pub fn changed_files(repo_root: impl Into<PathBuf>) -> anyhow::Result<Vec<GitChangedFile>> {
    status_files(repo_root)
}

pub fn changed_files_since(
    repo_root: impl Into<PathBuf>,
    rev: &str,
) -> anyhow::Result<Vec<GitChangedFile>> {
    let repo = Repository::discover(repo_root.into())?;
    let object = repo.revparse_single(rev)?;
    let tree = object.peel_to_tree()?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("bare repositories are not supported"))?;
    let mut options = DiffOptions::new();
    let diff = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut options))?;
    let mut files = Vec::new();

    diff.foreach(
        &mut |delta, _| {
            let status = match delta.status() {
                Delta::Added | Delta::Untracked => GitFileStatus::Added,
                Delta::Deleted => GitFileStatus::Deleted,
                Delta::Modified | Delta::Renamed | Delta::Copied | Delta::Typechange => {
                    GitFileStatus::Modified
                }
                _ => return true,
            };
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|path| workdir.join(path));
            if let Some(path) = path {
                let relative = path
                    .strip_prefix(workdir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push(GitChangedFile {
                    path: relative,
                    status,
                });
            }
            true
        },
        None,
        None,
        None,
    )?;

    files.sort_by(|a, b| a.path.cmp(&b.path));
    files.dedup_by(|a, b| a.path == b.path);
    Ok(files)
}
