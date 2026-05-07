use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

const TEXT_EXTENSIONS: &[&str] = &[
    "md", "toml", "rs", "ps1", "sh", "yml", "yaml", "json", "props", "targets", "sln", "txt", "cs",
    "csproj", "idl",
];

const TEXT_FILE_NAMES: &[&str] = &[
    ".dockerignore",
    ".editorconfig",
    ".gitattributes",
    ".gitignore",
    "AGENTS.md",
    "Cargo.lock",
    "Cargo.toml",
    "LICENSE",
    "README.md",
];

const TEXT_CHECK_EXCLUDED_PREFIXES: &[&str] = &[
    // External reference assets are kept for interoperability research and may
    // intentionally preserve upstream encodings, BOMs, and line endings.
    "adapters/reference/",
];

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = resolve_repo_root()?;
    let mut args = env::args().skip(1);

    let Some(command) = args.next() else {
        return Err(invalid_input(format!("missing command\n\n{}", usage())).into());
    };

    match command.as_str() {
        "check-text-files" => {
            ensure_no_extra_args(args)?;
            check_text_files(&repo_root)?;
        }
        "validate-workspace" => {
            let options = WorkspaceValidationOptions::parse(args)?;
            validate_workspace(&repo_root, options.skip_clippy)?;
        }
        "check-repo" => {
            let options = RepoCheckOptions::parse(args)?;

            if !options.skip_text_files {
                check_text_files(&repo_root)?;
            }

            validate_workspace(&repo_root, options.skip_clippy)?;
            println!("Repository checks passed.");
        }
        _ => {
            return Err(
                invalid_input(format!("unsupported command `{command}`\n\n{}", usage())).into(),
            );
        }
    }

    Ok(())
}

fn usage() -> &'static str {
    "Usage:
  cargo run -p xtask -- check-text-files
  cargo run -p xtask -- validate-workspace [--skip-clippy]
  cargo run -p xtask -- check-repo [--skip-clippy] [--skip-text-files]"
}

fn resolve_repo_root() -> Result<PathBuf, io::Error> {
    let current_dir = env::current_dir()?;

    for candidate in current_dir.ancestors() {
        if candidate.join("Cargo.toml").is_file() && candidate.join("AGENTS.md").is_file() {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(io::Error::other(
        "failed to locate repository root from current working directory",
    ))
}

fn ensure_no_extra_args(args: impl IntoIterator<Item = String>) -> Result<(), io::Error> {
    let unexpected = args.into_iter().collect::<Vec<_>>();
    if unexpected.is_empty() {
        return Ok(());
    }

    Err(invalid_input(format!(
        "unexpected arguments: {}",
        unexpected.join(" ")
    )))
}

fn check_text_files(repo_root: &Path) -> Result<(), io::Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("ls-files")
        .arg("--cached")
        .arg("--others")
        .arg("--exclude-standard")
        .arg("-z")
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other("failed to enumerate tracked files"));
    }

    let mut errors = Vec::new();

    for relative_path_bytes in output.stdout.split(|byte| *byte == 0) {
        if relative_path_bytes.is_empty() {
            continue;
        }

        let relative_path = String::from_utf8(relative_path_bytes.to_vec())
            .map_err(|_| io::Error::other("git returned a non-UTF-8 file path"))?;

        if relative_path.starts_with("target/")
            || is_text_check_excluded_path(&relative_path)
            || !should_check_text_path(&relative_path)
        {
            continue;
        }

        let full_path = repo_root.join(Path::new(&relative_path));
        if !full_path.is_file() {
            continue;
        }

        let bytes = fs::read(&full_path)?;
        if bytes.is_empty() {
            continue;
        }

        errors.extend(inspect_text_bytes(&relative_path, &bytes));
    }

    if errors.is_empty() {
        println!("Text file checks passed.");
        return Ok(());
    }

    for error in &errors {
        eprintln!("{error}");
    }

    Err(io::Error::other("text file checks failed"))
}

fn should_check_text_path(relative_path: &str) -> bool {
    if is_text_check_excluded_path(relative_path) {
        return false;
    }

    let path = Path::new(relative_path);

    let Some(file_name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };

    if TEXT_FILE_NAMES.contains(&file_name) {
        return true;
    }

    let Some(extension) = path.extension().and_then(OsStr::to_str) else {
        return false;
    };

    TEXT_EXTENSIONS
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
}

fn is_text_check_excluded_path(relative_path: &str) -> bool {
    TEXT_CHECK_EXCLUDED_PREFIXES
        .iter()
        .any(|prefix| relative_path.starts_with(prefix))
}

fn inspect_text_bytes(relative_path: &str, bytes: &[u8]) -> Vec<String> {
    let mut errors = Vec::new();

    if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
        errors.push(format!("{relative_path}: contains UTF-8 BOM"));
    }

    if bytes.contains(&0) {
        errors.push(format!("{relative_path}: contains NUL byte"));
        return errors;
    }

    let Ok(content) = std::str::from_utf8(bytes) else {
        errors.push(format!("{relative_path}: is not valid UTF-8"));
        return errors;
    };

    if content.contains('\r') {
        errors.push(format!(
            "{relative_path}: contains CRLF or CR line endings; expected LF"
        ));
    }

    if !content.ends_with('\n') {
        errors.push(format!("{relative_path}: missing trailing newline"));
    }

    errors
}

fn validate_workspace(repo_root: &Path, skip_clippy: bool) -> Result<(), io::Error> {
    run_checked_command(repo_root, "cargo", &["fmt", "--all", "--check"])?;
    run_checked_command(repo_root, "cargo", &["check", "--workspace"])?;
    run_checked_command(repo_root, "cargo", &["test", "--workspace"])?;

    if !skip_clippy {
        run_checked_command(
            repo_root,
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        )?;
    }

    Ok(())
}

fn run_checked_command(repo_root: &Path, file_path: &str, args: &[&str]) -> Result<(), io::Error> {
    println!("==> {file_path} {}", args.join(" "));

    let status = Command::new(file_path)
        .current_dir(repo_root)
        .args(args)
        .status()?;

    if status.success() {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "command failed: {file_path} {}",
        args.join(" ")
    )))
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

#[derive(Default)]
struct WorkspaceValidationOptions {
    skip_clippy: bool,
}

impl WorkspaceValidationOptions {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, io::Error> {
        let mut options = Self::default();

        for arg in args {
            match arg.as_str() {
                "--skip-clippy" => options.skip_clippy = true,
                _ => return Err(invalid_input(format!("unsupported argument `{arg}`"))),
            }
        }

        Ok(options)
    }
}

#[derive(Default)]
struct RepoCheckOptions {
    skip_clippy: bool,
    skip_text_files: bool,
}

impl RepoCheckOptions {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, io::Error> {
        let mut options = Self::default();

        for arg in args {
            match arg.as_str() {
                "--skip-clippy" => options.skip_clippy = true,
                "--skip-text-files" => options.skip_text_files = true,
                _ => return Err(invalid_input(format!("unsupported argument `{arg}`"))),
            }
        }

        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::{inspect_text_bytes, should_check_text_path};

    #[test]
    fn text_path_detection_covers_shell_and_workspace_files() {
        assert!(should_check_text_path("scripts/check-repo.sh"));
        assert!(should_check_text_path("README.md"));
        assert!(should_check_text_path("xtask/Cargo.toml"));
        assert!(should_check_text_path(
            "adapters/dotnet-capeopen/RadishFlow.CapeOpen.UnitOp.Mvp/typelib/RadishFlow.CapeOpen.UnitOp.Mvp.idl"
        ));
        assert!(should_check_text_path(".dockerignore"));
        assert!(!should_check_text_path("assets/logo.png"));
    }

    #[test]
    fn text_path_detection_skips_external_reference_assets() {
        assert!(!should_check_text_path(
            "adapters/reference/CapeOpenMixerExample_CSharp/CapeOpen/CapeOpen.cs"
        ));
    }

    #[test]
    fn inspect_text_bytes_accepts_utf8_lf_with_trailing_newline() {
        assert!(inspect_text_bytes("README.md", b"hello\nworld\n").is_empty());
    }

    #[test]
    fn inspect_text_bytes_reports_bom_and_crlf() {
        let errors = inspect_text_bytes("README.md", &[0xEF, 0xBB, 0xBF, b'a', b'\r', b'\n']);
        assert_eq!(
            errors,
            vec![
                "README.md: contains UTF-8 BOM".to_string(),
                "README.md: contains CRLF or CR line endings; expected LF".to_string(),
            ]
        );
    }

    #[test]
    fn inspect_text_bytes_reports_missing_trailing_newline() {
        let errors = inspect_text_bytes("README.md", b"hello");
        assert_eq!(
            errors,
            vec!["README.md: missing trailing newline".to_string()]
        );
    }

    #[test]
    fn inspect_text_bytes_reports_invalid_utf8() {
        let errors = inspect_text_bytes("README.md", &[0xFF]);
        assert_eq!(errors, vec!["README.md: is not valid UTF-8".to_string()]);
    }

    #[test]
    fn inspect_text_bytes_reports_nul_byte() {
        let errors = inspect_text_bytes("README.md", b"hello\0world");
        assert_eq!(errors, vec!["README.md: contains NUL byte".to_string()]);
    }
}
