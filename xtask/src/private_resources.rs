//! Guard the public Git index without opening research documents.
use std::{fs, io, path::Path, process::Command};

const PRIVATE_DIRECTORY: &str = "private-resources";

pub(crate) fn check_private_resources(repo_root: &Path) -> io::Result<()> {
    let canonical_private = fs::canonicalize(repo_root)?.join(PRIVATE_DIRECTORY);
    let tracked = git(repo_root, &["ls-files", "--stage", "-z"])?;
    for record in tracked.split(|byte| *byte == 0).filter(|v| !v.is_empty()) {
        let record = std::str::from_utf8(record)
            .map_err(|_| io::Error::other("Git index contains a non-UTF-8 path"))?;
        let (metadata, path) = record
            .split_once('\t')
            .ok_or_else(|| io::Error::other("invalid Git index record"))?;
        if contains_private_component(path) {
            return Err(io::Error::other(
                "private-resources must not be tracked in the public repository (including gitlinks)",
            ));
        }
        if metadata.starts_with("120000 ") {
            let object = metadata
                .split_whitespace()
                .nth(1)
                .ok_or_else(|| io::Error::other("missing symlink object"))?;
            let target = git(repo_root, &["cat-file", "blob", object])?;
            let target = std::str::from_utf8(&target)
                .map_err(|_| io::Error::other("non-UTF-8 symlink target"))?;
            let resolved = repo_root.join(path).parent().unwrap().join(target);
            if contains_private_component(target)
                || fs::canonicalize(&resolved).is_ok_and(|p| p.starts_with(&canonical_private))
            {
                return Err(io::Error::other(
                    "public index contains a symlink into private resources",
                ));
            }
        }
    }

    // A nonexistent probe also works in a public clone with no research checkout.
    let ignored = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args([
            "check-ignore",
            "--quiet",
            "--no-index",
            "private-resources/catalog/privacy-probe",
        ])
        .status()?;
    if !ignored.success() {
        return Err(io::Error::other(
            "private-resources must be ignored by the public repository",
        ));
    }
    println!("Private resource boundary checks passed.");
    Ok(())
}

fn contains_private_component(path: &str) -> bool {
    path.split(['/', '\\'])
        .any(|part| part.eq_ignore_ascii_case(PRIVATE_DIRECTORY))
}

fn git(repo_root: &Path, args: &[&str]) -> io::Result<Vec<u8>> {
    let result = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()?;
    if !result.status.success() {
        return Err(io::Error::other("private resource Git check failed"));
    }
    Ok(result.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    struct Repository(PathBuf);

    impl Repository {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "radishflow-private-boundary-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            git(&path, &["init", "--quiet"]).unwrap();
            fs::write(path.join(".gitignore"), "/private-resources/\n").unwrap();
            git(&path, &["add", ".gitignore"]).unwrap();
            Self(path)
        }
    }

    impl Drop for Repository {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn allows_public_clone_and_ignored_nested_repository() {
        let repo = Repository::new();
        check_private_resources(&repo.0).unwrap();
        let private = repo.0.join(PRIVATE_DIRECTORY);
        fs::create_dir(&private).unwrap();
        git(&private, &["init", "--quiet"]).unwrap();
        fs::write(private.join("source.pdf"), b"private fixture").unwrap();
        check_private_resources(&repo.0).unwrap();
    }

    #[test]
    fn rejects_forced_private_file_and_gitlink_even_without_worktree_file() {
        let repo = Repository::new();
        let private = repo.0.join(PRIVATE_DIRECTORY);
        fs::create_dir(&private).unwrap();
        let file = private.join("source.pdf");
        fs::write(&file, b"private fixture").unwrap();
        git(&repo.0, &["add", "-f", "private-resources/source.pdf"]).unwrap();
        fs::remove_file(file).unwrap();
        assert!(check_private_resources(&repo.0).is_err());
        git(
            &repo.0,
            &[
                "update-index",
                "--force-remove",
                "private-resources/source.pdf",
            ],
        )
        .unwrap();
        git(
            &repo.0,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                "160000,1111111111111111111111111111111111111111,private-resources",
            ],
        )
        .unwrap();
        assert!(check_private_resources(&repo.0).is_err());
    }

    #[test]
    fn rejects_missing_ignore_rule() {
        let repo = Repository::new();
        fs::write(repo.0.join(".gitignore"), "").unwrap();
        assert!(check_private_resources(&repo.0).is_err());
    }

    #[test]
    fn source_archive_excludes_even_forced_private_files() {
        let repo = Repository::new();
        fs::write(
            repo.0.join(".gitattributes"),
            include_str!("../../.gitattributes"),
        )
        .unwrap();
        fs::write(repo.0.join("README.md"), "PUBLIC_ARCHIVE_SENTINEL").unwrap();
        fs::create_dir(repo.0.join(PRIVATE_DIRECTORY)).unwrap();
        fs::write(
            repo.0.join("private-resources/source.pdf"),
            "PRIVATE_ARCHIVE_SENTINEL",
        )
        .unwrap();
        git(&repo.0, &["add", ".gitattributes", "README.md"]).unwrap();
        git(&repo.0, &["add", "-f", "private-resources/source.pdf"]).unwrap();
        let tree = git(&repo.0, &["write-tree"]).unwrap();
        let tree = std::str::from_utf8(&tree).unwrap().trim();
        let archive = git(&repo.0, &["archive", "--format=tar", tree]).unwrap();
        let archive = String::from_utf8_lossy(&archive);
        assert!(archive.contains("PUBLIC_ARCHIVE_SENTINEL"));
        assert!(!archive.contains("PRIVATE_ARCHIVE_SENTINEL"));
        assert!(!archive.contains("private-resources/source.pdf"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_staged_symlink_to_private_file() {
        let repo = Repository::new();
        std::os::unix::fs::symlink("private-resources/source.pdf", repo.0.join("public.pdf"))
            .unwrap();
        git(&repo.0, &["add", "public.pdf"]).unwrap();
        fs::remove_file(repo.0.join("public.pdf")).unwrap();
        assert!(check_private_resources(&repo.0).is_err());
    }
}
