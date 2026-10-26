//! Conservative PR routing. Stable promotions always retain the complete gate.
use serde_json::Value;
use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
    process::Command,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PrChecks {
    rust: bool,
    dotnet: bool,
    package: bool,
}

impl PrChecks {
    const ALL: Self = Self {
        rust: true,
        dotnet: true,
        package: true,
    };

    fn for_changes(target: &str, paths: &[&str]) -> Self {
        if target != "dev" || paths.is_empty() {
            return Self::ALL;
        }
        let mut checks = Self::default();
        for path in paths {
            if path.starts_with("adapters/dotnet-capeopen/") {
                checks.dotnet = true;
            } else if path.starts_with("apps/radishflow-studio/")
                || path.starts_with("crates/rf-ui/")
                || path.starts_with("crates/rf-canvas/")
            {
                checks.rust = true;
                checks.package = true;
            } else if path.starts_with("tests/rust-integration/") {
                checks.rust = true;
            } else if !is_documentation(path) {
                // Core crates, build configuration, gate changes and unknown paths
                // may affect both consumers. No dependency graph is approximated here.
                return Self::ALL;
            }
        }
        checks
    }

    fn output(self) -> String {
        format!(
            "rust={}\ndotnet={}\npackage={}\n",
            self.rust, self.dotnet, self.package
        )
    }
}

fn is_documentation(path: &str) -> bool {
    (path.starts_with("docs/") && path.ends_with(".md"))
        || matches!(
            path,
            "README.md"
                | "AGENTS.md"
                | "CLAUDE.md"
                | "CONTRIBUTING.md"
                | "CODE_OF_CONDUCT.md"
                | "SECURITY.md"
        )
}

pub(crate) fn plan_pr_checks(
    repo_root: &Path,
    args: impl Iterator<Item = String>,
) -> Result<(), io::Error> {
    let mut args = args;
    let mut base_ref = None;
    let mut target = None;
    while let Some(arg) = args.next() {
        let field = match arg.as_str() {
            "--base-ref" => &mut base_ref,
            "--target-branch" => &mut target,
            _ => {
                return Err(crate::invalid_input(format!(
                    "unsupported argument `{arg}`"
                )));
            }
        };
        if field.is_some() {
            return Err(crate::invalid_input(format!("duplicate argument `{arg}`")));
        }
        *field = Some(
            args.next()
                .ok_or_else(|| crate::invalid_input(format!("{arg} requires a value")))?,
        );
    }
    let base_ref = base_ref.ok_or_else(|| crate::invalid_input("--base-ref is required"))?;
    let target = target.ok_or_else(|| crate::invalid_input("--target-branch is required"))?;
    if !matches!(target.as_str(), "dev" | "master" | "main") {
        return Err(crate::invalid_input("unsupported PR target branch"));
    }
    let resolved = git_output(
        repo_root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{base_ref}^{{commit}}"),
        ],
    )?;
    let range = format!("{}...HEAD", resolved.trim());
    let paths = git_output(
        repo_root,
        &["diff", "--name-only", "--no-renames", "-z", &range, "--"],
    )?;
    let paths: Vec<_> = paths.split('\0').filter(|path| !path.is_empty()).collect();
    let checks = PrChecks::for_changes(&target, &paths);
    let output = checks.output();
    print!("{output}");
    if let Some(path) = env::var_os("GITHUB_OUTPUT") {
        OpenOptions::new()
            .append(true)
            .open(path)?
            .write_all(output.as_bytes())?;
    }
    Ok(())
}

fn git_output(root: &Path, args: &[&str]) -> Result<String, io::Error> {
    let output = Command::new("git").current_dir(root).args(args).output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout).map_err(io::Error::other)
}

pub(crate) fn check_pr_results() -> Result<(), io::Error> {
    let needs = env::var("RADISHFLOW_CI_NEEDS").map_err(io::Error::other)?;
    let needs: Value = serde_json::from_str(&needs).map_err(io::Error::other)?;
    validate_results(&needs)?;
    println!(
        "All required PR quality components succeeded; only explicitly inapplicable checks may be skipped."
    );
    Ok(())
}

fn validate_results(needs: &Value) -> Result<(), io::Error> {
    if needs["repo-hygiene"]["result"].as_str() != Some("success") {
        return Err(io::Error::other(
            "Repo Hygiene (including PR plan) did not succeed",
        ));
    }
    for (flag, jobs) in [
        (
            "rust",
            &[
                "rust-baseline",
                "rust-baseline-macos",
                "rust-baseline-windows",
            ][..],
        ),
        ("dotnet", &["dotnet-adapter-baseline"][..]),
        ("package", &["windows-staging-package"][..]),
    ] {
        let required = match needs["repo-hygiene"]["outputs"][flag].as_str() {
            Some("true") => true,
            Some("false") => false,
            _ => {
                return Err(io::Error::other(format!(
                    "missing or invalid PR plan flag: {flag}"
                )));
            }
        };
        for job in jobs {
            let result = needs[*job]["result"].as_str();
            if result != Some("success") && (required || result != Some("skipped")) {
                return Err(io::Error::other(format!(
                    "{job}: required={required}, result={result:?}"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn routing_retains_consumers_and_treats_unknown_changes_conservatively() {
        assert_eq!(
            PrChecks::for_changes("dev", &["docs/status/current.md", "README.md"]),
            PrChecks::default()
        );
        assert_eq!(
            PrChecks::for_changes("dev", &["apps/radishflow-studio/src/main.rs"]),
            PrChecks {
                rust: true,
                dotnet: false,
                package: true
            }
        );
        assert_eq!(
            PrChecks::for_changes("dev", &["adapters/dotnet-capeopen/Project.cs"]),
            PrChecks {
                dotnet: true,
                ..Default::default()
            }
        );
        assert_eq!(
            PrChecks::for_changes("dev", &["tests/rust-integration/tests/example.rs"]),
            PrChecks {
                rust: true,
                ..Default::default()
            }
        );
        for path in [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "crates/rf-types/src/lib.rs",
            "scripts/check-repo.sh",
            "xtask/src/main.rs",
            ".github/workflows/pr-check.yml",
            "new-component/file",
            "docs/fixture.json",
            "LICENSE",
        ] {
            assert_eq!(
                PrChecks::for_changes("dev", &[path]),
                PrChecks::ALL,
                "{path}"
            );
        }
        // A rename is supplied as old + new paths; moving code into docs cannot exempt it.
        assert_eq!(
            PrChecks::for_changes("dev", &["crates/rf-types/src/lib.rs", "docs/old.md"]),
            PrChecks::ALL
        );
        assert_eq!(
            PrChecks::for_changes(
                "dev",
                &[
                    "apps/radishflow-studio/src/main.rs",
                    "adapters/dotnet-capeopen/Project.cs"
                ]
            ),
            PrChecks::ALL
        );
        for target in ["master", "main"] {
            assert_eq!(PrChecks::for_changes(target, &["README.md"]), PrChecks::ALL);
        }
        assert_eq!(PrChecks::for_changes("dev", &[]), PrChecks::ALL);
    }

    fn results(required: bool, result: &str) -> Value {
        let flag = required.to_string();
        let mut needs = json!({"repo-hygiene": {"result": "success", "outputs": {"rust": flag, "dotnet": flag, "package": flag}}});
        for job in [
            "rust-baseline",
            "rust-baseline-macos",
            "rust-baseline-windows",
            "dotnet-adapter-baseline",
            "windows-staging-package",
        ] {
            needs[job] = json!({"result": result});
        }
        needs
    }

    #[test]
    fn aggregate_rejects_failure_cancellation_missing_and_unplanned_skip() {
        assert!(validate_results(&results(true, "success")).is_ok());
        assert!(validate_results(&results(false, "skipped")).is_ok());
        for required in [true, false] {
            for bad in ["failure", "cancelled", "", "unknown"] {
                assert!(validate_results(&results(required, bad)).is_err());
            }
        }
        assert!(validate_results(&results(true, "skipped")).is_err());
        let mut needs = results(false, "skipped");
        needs["repo-hygiene"]["outputs"]["rust"] = Value::Null;
        assert!(validate_results(&needs).is_err());
        let mut needs = results(false, "skipped");
        needs["repo-hygiene"]["result"] = json!("failure");
        assert!(validate_results(&needs).is_err());
        let mut needs = results(true, "success");
        needs
            .as_object_mut()
            .unwrap()
            .remove("rust-baseline-windows");
        assert!(validate_results(&needs).is_err());
    }
}
