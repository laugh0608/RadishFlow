use std::{
    fs,
    path::Path,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn plan(root: &Path, base: &str, target: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(root)
        .env_remove("GITHUB_OUTPUT")
        .args([
            "plan-pr-checks",
            "--base-ref",
            base,
            "--target-branch",
            target,
        ])
        .output()
        .unwrap()
}

#[test]
fn pr_plan_uses_entire_merge_base_diff_including_renames_and_deletions() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("rf-pr-plan-{}-{unique}", std::process::id()));
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("Cargo.toml"), "").unwrap();
    fs::write(root.join("AGENTS.md"), "").unwrap();
    fs::create_dir_all(root.join("crates/rf-types/src")).unwrap();
    fs::write(root.join("crates/rf-types/src/lib.rs"), "// initial core\n").unwrap();
    git(&root, &["init", "--quiet"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "--quiet", "-m", "base"]);
    git(&root, &["tag", "base"]);
    fs::write(root.join("docs/guide.md"), "guide\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "--quiet", "-m", "docs"]);
    let output = plan(&root, "base", "dev");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "rust=false\ndotnet=false\npackage=false\n"
    );
    assert!(
        String::from_utf8(plan(&root, "base", "main").stdout)
            .unwrap()
            .contains("package=true")
    );
    assert!(!plan(&root, "missing-ref", "dev").status.success());
    // Core changes remain in the PR range even when the latest commit is documentation.
    fs::create_dir_all(root.join("crates/rf-types/src")).unwrap();
    fs::write(root.join("crates/rf-types/src/lib.rs"), "// core\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "--quiet", "-m", "core"]);
    git(&root, &["tag", "before-move"]);
    git(&root, &["mv", "crates/rf-types/src/lib.rs", "docs/old.md"]);
    git(&root, &["commit", "--quiet", "-m", "move"]);
    for base in ["base", "before-move"] {
        let output = plan(&root, base, "dev");
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "rust=true\ndotnet=true\npackage=true\n"
        );
    }
    fs::remove_dir_all(root).unwrap();
}
