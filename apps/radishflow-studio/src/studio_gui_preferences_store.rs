use std::path::{Path, PathBuf};

use rf_store::{
    STORED_STUDIO_PREFERENCES_FILE_NAME, StoredStudioPreferencesFile, read_studio_preferences_file,
    write_studio_preferences_file,
};
use rf_types::RfResult;

pub fn default_studio_preferences_path() -> PathBuf {
    if let Some(path) = std::env::var_os("RADISHFLOW_STUDIO_PREFERENCES_PATH") {
        return PathBuf::from(path);
    }

    platform_config_root()
        .join("RadishFlow")
        .join("Studio")
        .join(STORED_STUDIO_PREFERENCES_FILE_NAME)
}

pub fn load_recent_project_paths(preferences_path: &Path) -> RfResult<Vec<PathBuf>> {
    if !preferences_path.exists() {
        return Ok(Vec::new());
    }

    let preferences = read_studio_preferences_file(preferences_path)?;
    Ok(preferences
        .recent_project_paths
        .into_iter()
        .map(PathBuf::from)
        .collect())
}

pub fn save_recent_project_paths(
    preferences_path: &Path,
    recent_projects: &[PathBuf],
) -> RfResult<()> {
    let preferences = StoredStudioPreferencesFile::new(
        recent_projects
            .iter()
            .map(|project_path| project_path.display().to_string())
            .collect(),
    );
    write_studio_preferences_file(preferences_path, &preferences)
}

#[cfg(target_os = "windows")]
fn platform_config_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir)
}

#[cfg(not(target_os = "windows"))]
fn platform_config_root() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(std::env::temp_dir)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{load_recent_project_paths, save_recent_project_paths};

    #[test]
    fn preferences_store_round_trips_recent_project_paths() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("radishflow-preferences-store-{unique}"));
        let preferences_path = root
            .join("RadishFlow")
            .join("Studio")
            .join("preferences.rfstudio-preferences.json");
        let recent_projects = vec![
            root.join("demo-a.rfproj.json"),
            root.join("demo-b.rfproj.json"),
        ];

        save_recent_project_paths(&preferences_path, &recent_projects)
            .expect("expected preferences save");
        let loaded =
            load_recent_project_paths(&preferences_path).expect("expected preferences load");

        assert_eq!(loaded, recent_projects);
        fs::remove_dir_all(root).expect("expected temp cleanup");
    }
}
