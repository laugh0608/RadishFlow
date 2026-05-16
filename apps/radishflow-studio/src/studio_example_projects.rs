use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioExampleProjectModel {
    pub id: &'static str,
    pub title: &'static str,
    pub detail: &'static str,
    pub project_path: PathBuf,
    pub is_current: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StudioExampleProjectDefinition {
    id: &'static str,
    title: &'static str,
    detail: &'static str,
    file_name: &'static str,
}

const STUDIO_EXAMPLE_PROJECTS: &[StudioExampleProjectDefinition] = &[
    StudioExampleProjectDefinition {
        id: "feed-heater-flash",
        title: "Feed -> Heater -> Flash",
        detail: "Single feed, official binary hydrocarbon heater, flash drum",
        file_name: "feed-heater-flash-binary-hydrocarbon.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "feed-valve-flash",
        title: "Feed -> Valve -> Flash",
        detail: "Single feed, official binary hydrocarbon valve pressure drop, flash drum",
        file_name: "feed-valve-flash-binary-hydrocarbon.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "feed-cooler-flash",
        title: "Feed -> Cooler -> Flash",
        detail: "Single feed, official binary hydrocarbon cooler, flash drum",
        file_name: "feed-cooler-flash-binary-hydrocarbon.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "feed-mixer-flash",
        title: "Feed + Feed -> Mixer -> Flash",
        detail: "Two feeds, official binary hydrocarbon mixer, flash drum",
        file_name: "feed-mixer-flash-binary-hydrocarbon.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "feed-mixer-heater-flash",
        title: "Mixer -> Heater -> Flash",
        detail: "Two feeds, synthetic demo mixer, heater, flash drum",
        file_name: "feed-mixer-heater-flash-synthetic-demo.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "water-ethanol-heater-flash",
        title: "Water/ethanol heater flash",
        detail: "Water/ethanol PME validation sample",
        file_name: "feed-heater-flash-water-ethanol.rfproj.json",
    },
];

pub fn studio_example_project_models(
    current_project_path: Option<&Path>,
) -> Vec<StudioExampleProjectModel> {
    let examples_root = studio_examples_root();
    STUDIO_EXAMPLE_PROJECTS
        .iter()
        .map(|definition| {
            let project_path = examples_root.join(definition.file_name);
            let is_current = current_project_path
                .map(|current| path_eq(current, &project_path))
                .unwrap_or(false);
            StudioExampleProjectModel {
                id: definition.id,
                title: definition.title,
                detail: definition.detail,
                project_path,
                is_current,
            }
        })
        .collect()
}

fn studio_examples_root() -> PathBuf {
    let source_examples_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join("flowsheets");
    let exe_path = std::env::current_exe().ok();
    resolve_studio_examples_root(
        std::env::var_os("RADISHFLOW_EXAMPLES_DIR"),
        exe_path.as_deref(),
        source_examples_root,
    )
}

fn resolve_studio_examples_root(
    configured_examples_dir: Option<OsString>,
    exe_path: Option<&Path>,
    source_examples_root: PathBuf,
) -> PathBuf {
    if let Some(path) = configured_examples_dir {
        return PathBuf::from(path);
    }

    if let Some(exe_path) = exe_path {
        if let Some(exe_dir) = exe_path.parent() {
            let packaged_examples = exe_dir.join("examples").join("flowsheets");
            if packaged_examples.exists() {
                return packaged_examples;
            }
        }
    }

    source_examples_root
}

fn path_eq(left: &Path, right: &Path) -> bool {
    left == right
        || left
            .canonicalize()
            .ok()
            .zip(right.canonicalize().ok())
            .map(|(left, right)| left == right)
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs, path::PathBuf};

    use super::{resolve_studio_examples_root, studio_example_project_models};

    #[test]
    fn studio_example_project_models_mark_current_project() {
        let models = studio_example_project_models(None);
        let current = models
            .iter()
            .find(|model| model.id == "feed-valve-flash")
            .expect("expected feed valve example")
            .project_path
            .clone();

        let models = studio_example_project_models(Some(&current));

        assert_eq!(models.iter().filter(|model| model.is_current).count(), 1);
        assert_eq!(
            models
                .iter()
                .find(|model| model.is_current)
                .map(|model| model.id),
            Some("feed-valve-flash")
        );
    }

    #[test]
    fn studio_examples_root_prefers_configured_directory() {
        let configured = PathBuf::from("configured-examples");
        let source = PathBuf::from("source-examples");
        let resolved = resolve_studio_examples_root(
            Some(OsString::from(configured.as_os_str())),
            Some(PathBuf::from("package/radishflow-studio.exe").as_path()),
            source,
        );

        assert_eq!(resolved, configured);
    }

    #[test]
    fn studio_examples_root_uses_packaged_examples_when_present() {
        let package_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("target")
            .join("radishflow-studio-tests")
            .join(format!("packaged-examples-{}", std::process::id()));
        let packaged_examples = package_root.join("examples").join("flowsheets");
        fs::create_dir_all(&packaged_examples).expect("create packaged examples dir");

        let exe_path = package_root.join("radishflow-studio.exe");
        let resolved =
            resolve_studio_examples_root(None, Some(&exe_path), PathBuf::from("source-examples"));

        assert_eq!(resolved, packaged_examples);
    }

    #[test]
    fn studio_examples_root_falls_back_to_source_examples() {
        let source = PathBuf::from("source-examples");
        let resolved = resolve_studio_examples_root(
            None,
            Some(PathBuf::from("package/radishflow-studio.exe").as_path()),
            source.clone(),
        );

        assert_eq!(resolved, source);
    }
}
