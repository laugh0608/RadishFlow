use serde_json::Value;
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

const REQUIRED_FILES: &[&str] = &[
    ".editorconfig",
    ".gitattributes",
    ".gitignore",
    ".github/ISSUE_TEMPLATE/config.yml",
    ".github/PULL_REQUEST_TEMPLATE.md",
    ".github/rulesets/README.md",
    ".github/rulesets/master-protection.json",
    ".github/workflows/pr-check.yml",
    ".github/workflows/release-check.yml",
    "AGENTS.md",
    "CLAUDE.md",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "LICENSE",
    "README.md",
    "SECURITY.md",
    "docs/README.md",
    "docs/adr/0001-branch-and-pr-governance.md",
    "docs/status/current.md",
    "scripts/check-repo.ps1",
    "scripts/check-repo.sh",
    "xtask/src/repository_governance.rs",
];

const GOVERNANCE_CHECK_EXCLUDED_PREFIXES: &[&str] = &["adapters/reference/"];

pub(crate) fn check_repository_governance(
    repo_root: &Path,
    base_ref: Option<&str>,
) -> Result<(), io::Error> {
    let paths = repository_files(repo_root)?;
    let mut errors = Vec::new();

    check_required_files(repo_root, &mut errors);
    check_json_files(repo_root, &paths, &mut errors);
    check_markdown_links(repo_root, &paths, &mut errors);
    check_agent_files(repo_root, &mut errors);
    check_branch_sync_contract(repo_root, &mut errors);
    check_issue_template_contract(repo_root, &mut errors);
    check_ruleset_contract(repo_root, &mut errors);
    check_workflow_contract(repo_root, &mut errors);
    check_pull_request_template_contract(repo_root, &mut errors);
    check_diff(repo_root, base_ref, &mut errors)?;

    if errors.is_empty() {
        println!(
            "Repository governance checks passed ({} files inspected).",
            paths.len()
        );
        return Ok(());
    }

    for error in &errors {
        eprintln!("{error}");
    }

    Err(io::Error::other("repository governance checks failed"))
}

fn repository_files(repo_root: &Path) -> Result<Vec<PathBuf>, io::Error> {
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
        return Err(io::Error::other("failed to enumerate repository files"));
    }

    let mut paths = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|item| !item.is_empty())
        .map(|item| {
            let relative = String::from_utf8(item.to_vec())
                .map_err(|_| io::Error::other("git returned a non-UTF-8 file path"))?;
            Ok(PathBuf::from(relative))
        })
        .collect::<Result<Vec<_>, io::Error>>()?;
    paths.sort();
    Ok(paths)
}

fn check_required_files(repo_root: &Path, errors: &mut Vec<String>) {
    for relative_path in REQUIRED_FILES {
        if !repo_root.join(relative_path).is_file() {
            errors.push(format!("missing required governance file: {relative_path}"));
        }
    }
}

fn check_json_files(repo_root: &Path, paths: &[PathBuf], errors: &mut Vec<String>) {
    for relative_path in paths {
        if is_governance_check_excluded_path(relative_path)
            || relative_path.extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }

        let full_path = repo_root.join(relative_path);
        if !full_path.is_file() {
            continue;
        }

        let content = match fs::read_to_string(&full_path) {
            Ok(content) => content,
            Err(error) => {
                errors.push(format!(
                    "failed to read JSON file {}: {error}",
                    relative_path.display()
                ));
                continue;
            }
        };

        if let Err(error) = serde_json::from_str::<Value>(&content) {
            errors.push(format!("invalid JSON {}: {error}", relative_path.display()));
        }
    }
}

fn check_markdown_links(repo_root: &Path, paths: &[PathBuf], errors: &mut Vec<String>) {
    let canonical_root = match fs::canonicalize(repo_root) {
        Ok(path) => path,
        Err(error) => {
            errors.push(format!("failed to resolve repository root: {error}"));
            return;
        }
    };

    for relative_path in paths {
        if is_governance_check_excluded_path(relative_path)
            || relative_path.extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }

        let full_path = repo_root.join(relative_path);
        if !full_path.is_file() {
            continue;
        }

        let content = match fs::read_to_string(&full_path) {
            Ok(content) => content,
            Err(error) => {
                errors.push(format!(
                    "failed to read Markdown file {}: {error}",
                    relative_path.display()
                ));
                continue;
            }
        };

        for (line_number, target) in markdown_link_targets(&content) {
            let Some(target) = repository_relative_link_target(&target) else {
                continue;
            };

            let decoded_target = match percent_decode(&target) {
                Ok(value) => value,
                Err(error) => {
                    errors.push(format!(
                        "invalid percent encoding in Markdown link {}:{line_number}: {error}",
                        relative_path.display()
                    ));
                    continue;
                }
            };

            let resolved = full_path.parent().unwrap_or(repo_root).join(decoded_target);
            if !resolved.exists() {
                errors.push(format!(
                    "broken relative Markdown link {}:{line_number} -> {target}",
                    relative_path.display()
                ));
                continue;
            }

            let canonical_target = match fs::canonicalize(&resolved) {
                Ok(path) => path,
                Err(error) => {
                    errors.push(format!(
                        "failed to resolve Markdown link {}:{line_number} -> {target}: {error}",
                        relative_path.display()
                    ));
                    continue;
                }
            };

            if !canonical_target.starts_with(&canonical_root) {
                errors.push(format!(
                    "relative Markdown link escapes repository {}:{line_number} -> {target}",
                    relative_path.display()
                ));
            }
        }
    }
}

fn markdown_link_targets(content: &str) -> Vec<(usize, String)> {
    let mut targets = Vec::new();
    let mut fence_marker: Option<char> = None;

    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let line_fence = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };

        if let Some(marker) = line_fence {
            if fence_marker == Some(marker) {
                fence_marker = None;
            } else if fence_marker.is_none() {
                fence_marker = Some(marker);
            }
            continue;
        }

        if fence_marker.is_some() {
            continue;
        }

        let visible_line = without_inline_code(line);
        let mut remaining = visible_line.as_str();
        while let Some(link_start) = remaining.find("](") {
            let after_open = &remaining[link_start + 2..];
            let Some((target, consumed)) = parse_inline_link_target(after_open) else {
                break;
            };
            if !target.is_empty() {
                targets.push((line_index + 1, target));
            }
            remaining = &after_open[consumed..];
        }
    }

    targets
}

fn without_inline_code(line: &str) -> String {
    let mut visible = String::with_capacity(line.len());
    let mut in_code = false;

    for character in line.chars() {
        if character == '`' {
            in_code = !in_code;
            visible.push(' ');
        } else if in_code {
            visible.push(' ');
        } else {
            visible.push(character);
        }
    }

    visible
}

fn parse_inline_link_target(after_open: &str) -> Option<(String, usize)> {
    let trimmed = after_open.trim_start();
    let leading_whitespace = after_open.len() - trimmed.len();

    if let Some(angle_target) = trimmed.strip_prefix('<') {
        let end = angle_target.find('>')?;
        let closing_parenthesis = angle_target[end + 1..].find(')')?;
        let consumed = leading_whitespace + 1 + end + 1 + closing_parenthesis + 1;
        return Some((angle_target[..end].to_string(), consumed));
    }

    let end = trimmed.find(')')?;
    let target_with_title = &trimmed[..end];
    let target = target_with_title
        .split_ascii_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    Some((target, leading_whitespace + end + 1))
}

fn repository_relative_link_target(target: &str) -> Option<String> {
    let target = target.trim();
    if target.is_empty()
        || target.starts_with('#')
        || target.starts_with('/')
        || target.starts_with("mailto:")
        || target.starts_with("data:")
        || target.contains("://")
    {
        return None;
    }

    let end = target.find(['#', '?']).unwrap_or(target.len());
    let path = &target[..end];
    (!path.is_empty()).then(|| path.to_string())
}

fn percent_decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'%' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }

        if index + 2 >= bytes.len() {
            return Err("truncated percent escape".to_string());
        }
        let high =
            hex_value(bytes[index + 1]).ok_or_else(|| "invalid percent escape".to_string())?;
        let low =
            hex_value(bytes[index + 2]).ok_or_else(|| "invalid percent escape".to_string())?;
        decoded.push((high << 4) | low);
        index += 3;
    }

    String::from_utf8(decoded).map_err(|_| "decoded target is not UTF-8".to_string())
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn is_governance_check_excluded_path(relative_path: &Path) -> bool {
    let normalized = relative_path.to_string_lossy().replace('\\', "/");
    GOVERNANCE_CHECK_EXCLUDED_PREFIXES
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
}

fn check_agent_files(repo_root: &Path, errors: &mut Vec<String>) {
    let agents = repo_root.join("AGENTS.md");
    let claude = repo_root.join("CLAUDE.md");
    if agents.is_file() && claude.is_file() && fs::read(&agents).ok() != fs::read(&claude).ok() {
        errors.push("AGENTS.md and CLAUDE.md must remain identical".to_string());
    }
}

fn check_branch_sync_contract(repo_root: &Path, errors: &mut Vec<String>) {
    for relative_path in ["AGENTS.md", "CLAUDE.md"] {
        check_required_fragments(
            repo_root,
            relative_path,
            &[
                "任何 PR 合并到 `master` / `main` 后",
                "回灌并推送到 `dev`",
                "禁止使用 rebase、reset、force push",
            ],
            errors,
        );
    }

    check_required_fragments(
        repo_root,
        "docs/adr/0001-branch-and-pr-governance.md",
        &[
            "### `master` / `main` -> `dev` 合并后回灌",
            "回灌是稳定主线 PR 的必需收口动作",
            "可快进时，优先使用 fast-forward",
        ],
        errors,
    );
    check_required_fragments(
        repo_root,
        "CONTRIBUTING.md",
        &["开始下一轮开发前必须把最新 `origin/master` / `origin/main` 回灌并推送到 `dev`"],
        errors,
    );
    check_required_fragments(
        repo_root,
        ".github/PULL_REQUEST_TEMPLATE.md",
        &["已明确合并后立即把最新稳定主线回灌 `dev` 的执行人和时机"],
        errors,
    );
    check_required_fragments(
        repo_root,
        ".github/rulesets/README.md",
        &["合并到默认分支后，先把最新默认分支回灌并推送到 `dev`"],
        errors,
    );
}

fn check_issue_template_contract(repo_root: &Path, errors: &mut Vec<String>) {
    check_required_fragments(
        repo_root,
        ".github/ISSUE_TEMPLATE/config.yml",
        &[
            "blank_issues_enabled: false",
            "私下报告安全问题",
            "https://github.com/laugh0608/RadishFlow/security/advisories/new",
        ],
        errors,
    );
}

fn check_ruleset_contract(repo_root: &Path, errors: &mut Vec<String>) {
    let relative_path = ".github/rulesets/master-protection.json";
    let Ok(content) = fs::read_to_string(repo_root.join(relative_path)) else {
        return;
    };
    let Ok(ruleset) = serde_json::from_str::<Value>(&content) else {
        return;
    };

    if ruleset.get("target").and_then(Value::as_str) != Some("branch") {
        errors.push("master ruleset target must be branch".to_string());
    }
    if ruleset.get("enforcement").and_then(Value::as_str) != Some("active") {
        errors.push("master ruleset template must declare active enforcement".to_string());
    }

    let includes = ruleset
        .pointer("/conditions/ref_name/include")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    if includes != ["refs/heads/master", "refs/heads/main"] {
        errors.push("master ruleset must target master and main".to_string());
    }

    let rules = ruleset
        .get("rules")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let rule_types = rules
        .iter()
        .filter_map(|rule| rule.get("type").and_then(Value::as_str))
        .collect::<HashSet<_>>();
    for required_type in [
        "deletion",
        "non_fast_forward",
        "pull_request",
        "required_status_checks",
        "commit_message_pattern",
    ] {
        if !rule_types.contains(required_type) {
            errors.push(format!("master ruleset is missing rule: {required_type}"));
        }
    }

    let pull_request = find_rule(rules, "pull_request");
    let merge_methods = pull_request
        .and_then(|rule| rule.pointer("/parameters/allowed_merge_methods"))
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    if merge_methods != ["merge", "rebase"] {
        errors.push("master ruleset must allow merge and rebase, in that order".to_string());
    }
    if pull_request
        .and_then(|rule| rule.pointer("/parameters/required_review_thread_resolution"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        errors.push("master ruleset must require review thread resolution".to_string());
    }
    if pull_request
        .and_then(|rule| rule.pointer("/parameters/required_approving_review_count"))
        .and_then(Value::as_u64)
        != Some(0)
    {
        errors.push("single-maintainer baseline must require zero approvals".to_string());
    }

    let status_checks = find_rule(rules, "required_status_checks");
    let contexts = status_checks
        .and_then(|rule| rule.pointer("/parameters/required_status_checks"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("context").and_then(Value::as_str))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if contexts != ["Candidate Quality"] {
        errors.push("master ruleset must require only Candidate Quality".to_string());
    }
    if status_checks
        .and_then(|rule| rule.pointer("/parameters/strict_required_status_checks_policy"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        errors.push("master ruleset must require the branch to be up to date".to_string());
    }
}

fn find_rule<'a>(rules: &'a [Value], rule_type: &str) -> Option<&'a Value> {
    rules
        .iter()
        .find(|rule| rule.get("type").and_then(Value::as_str) == Some(rule_type))
}

fn check_workflow_contract(repo_root: &Path, errors: &mut Vec<String>) {
    check_required_fragments(
        repo_root,
        ".github/workflows/pr-check.yml",
        &[
            "pull_request:",
            "      - dev",
            "      - master",
            "      - main",
            "permissions:\n  contents: read",
            "name: Repo Hygiene",
            "name: Candidate Quality",
            "if: always()",
            "./scripts/check-repo.sh",
            "check-repository-governance",
        ],
        errors,
    );
    check_required_fragments(
        repo_root,
        ".github/workflows/release-check.yml",
        &[
            "workflow_dispatch:",
            "permissions:\n  contents: read",
            "name: Repo Hygiene",
            "check-repository-governance",
        ],
        errors,
    );

    for relative_path in [
        ".github/workflows/pr-check.yml",
        ".github/workflows/release-check.yml",
    ] {
        let Ok(content) = fs::read_to_string(repo_root.join(relative_path)) else {
            continue;
        };
        for forbidden_trigger in ["pull_request_target:", "workflow_run:"] {
            if content.contains(forbidden_trigger) {
                errors.push(format!(
                    "workflow {relative_path} must not use privileged trigger {forbidden_trigger}"
                ));
            }
        }
    }
}

fn check_pull_request_template_contract(repo_root: &Path, errors: &mut Vec<String>) {
    check_required_fragments(
        repo_root,
        ".github/PULL_REQUEST_TEMPLATE.md",
        &[
            "已停止公开业务功能维护和外部 PR 合并",
            "CONTRIBUTING.md",
            "SECURITY.md",
            "任职单位或其他第三方的保密材料",
            "未验证内容、回滚方式",
        ],
        errors,
    );
}

fn check_required_fragments(
    repo_root: &Path,
    relative_path: &str,
    fragments: &[&str],
    errors: &mut Vec<String>,
) {
    let Ok(content) = fs::read_to_string(repo_root.join(relative_path)) else {
        return;
    };
    for fragment in fragments {
        if !content.contains(fragment) {
            errors.push(format!(
                "{relative_path} is missing governance contract fragment: {fragment}"
            ));
        }
    }
}

fn check_diff(
    repo_root: &Path,
    base_ref: Option<&str>,
    errors: &mut Vec<String>,
) -> Result<(), io::Error> {
    let commands = if let Some(base_ref) = base_ref {
        let verification = git(repo_root, &["rev-parse", "--verify", base_ref])?;
        if !verification.status.success() {
            errors.push(format!("base ref does not resolve: {base_ref}"));
            return Ok(());
        }
        vec![vec![
            "diff".to_string(),
            "--check".to_string(),
            format!("{base_ref}...HEAD"),
        ]]
    } else {
        vec![
            vec!["diff".to_string(), "--check".to_string()],
            vec![
                "diff".to_string(),
                "--cached".to_string(),
                "--check".to_string(),
            ],
        ]
    };

    for command in commands {
        let args = command.iter().map(String::as_str).collect::<Vec<_>>();
        let result = git(repo_root, &args)?;
        if !result.status.success() {
            let detail = String::from_utf8_lossy(&result.stdout).trim().to_string()
                + String::from_utf8_lossy(&result.stderr).trim();
            errors.push(format!("git {} failed: {detail}", command.join(" ")));
        }
    }

    Ok(())
}

fn git(repo_root: &Path, args: &[&str]) -> Result<std::process::Output, io::Error> {
    Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
}

#[cfg(test)]
mod tests {
    use super::{
        markdown_link_targets, parse_inline_link_target, percent_decode,
        repository_relative_link_target,
    };

    #[test]
    fn markdown_link_targets_skip_code_and_fences() {
        let content = "[doc](docs/README.md) and `[ignored](missing.md)`\n\
                       ```text\n[fenced](missing.md)\n```\n\
                       [external](https://example.com)\n";
        assert_eq!(
            markdown_link_targets(content),
            vec![
                (1, "docs/README.md".to_string()),
                (5, "https://example.com".to_string()),
            ]
        );
    }

    #[test]
    fn markdown_link_targets_collect_multiple_links_on_one_line() {
        assert_eq!(
            markdown_link_targets("[one](README.md) and [two](docs/README.md)\n"),
            vec![
                (1, "README.md".to_string()),
                (1, "docs/README.md".to_string()),
            ]
        );
    }

    #[test]
    fn inline_link_target_supports_titles_and_angle_brackets() {
        assert_eq!(
            parse_inline_link_target("docs/README.md \"Docs\") tail"),
            Some(("docs/README.md".to_string(), 22))
        );
        assert_eq!(
            parse_inline_link_target("<docs/My File.md>) tail"),
            Some(("docs/My File.md".to_string(), 18))
        );
    }

    #[test]
    fn repository_relative_link_target_filters_non_file_targets() {
        assert_eq!(
            repository_relative_link_target("docs/README.md#governance"),
            Some("docs/README.md".to_string())
        );
        assert_eq!(repository_relative_link_target("#heading"), None);
        assert_eq!(repository_relative_link_target("https://example.com"), None);
        assert_eq!(
            repository_relative_link_target("mailto:test@example.com"),
            None
        );
    }

    #[test]
    fn percent_decode_accepts_utf8_paths_and_rejects_invalid_escapes() {
        assert_eq!(
            percent_decode("docs/My%20File.md"),
            Ok("docs/My File.md".to_string())
        );
        assert!(percent_decode("docs/%ZZ.md").is_err());
    }
}
