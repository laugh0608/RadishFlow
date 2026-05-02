use std::path::{Path, PathBuf};

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
        detail: "Single feed, heater/cooler, flash drum",
        file_name: "feed-heater-flash.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "feed-valve-flash",
        title: "Feed -> Valve -> Flash",
        detail: "Single feed, valve pressure drop, flash drum",
        file_name: "feed-valve-flash.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "feed-cooler-flash",
        title: "Feed -> Cooler -> Flash",
        detail: "Single feed, cooler, flash drum",
        file_name: "feed-cooler-flash.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "feed-mixer-flash",
        title: "Feed + Feed -> Mixer -> Flash",
        detail: "Two feeds, mixer, flash drum",
        file_name: "feed-mixer-flash.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "feed-mixer-heater-flash",
        title: "Mixer -> Heater -> Flash",
        detail: "Two feeds, mixer, heater, flash drum",
        file_name: "feed-mixer-heater-flash.rfproj.json",
    },
    StudioExampleProjectDefinition {
        id: "binary-hydrocarbon-heater-flash",
        title: "Hydrocarbon heater flash",
        detail: "Binary hydrocarbon package-facing sample",
        file_name: "feed-heater-flash-binary-hydrocarbon.rfproj.json",
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
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join("flowsheets")
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
    use super::studio_example_project_models;

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
}
