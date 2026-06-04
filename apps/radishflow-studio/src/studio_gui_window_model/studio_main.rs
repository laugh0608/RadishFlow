use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiWindowHomeCaseTileSource {
    Recent,
    Example,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiWindowHomeCaseTileStatus {
    Ready,
    Current,
    MissingFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowThumbnailFlowModel {
    pub nodes: Vec<String>,
    pub edges: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowHomeCaseTileModel {
    pub source: StudioGuiWindowHomeCaseTileSource,
    pub source_id: String,
    pub title: String,
    pub detail: String,
    pub path_text: String,
    pub package_summary: String,
    pub component_summary: String,
    pub status: StudioGuiWindowHomeCaseTileStatus,
    pub status_label: &'static str,
    pub thumbnail: StudioGuiWindowThumbnailFlowModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowHomeModel {
    pub title: &'static str,
    pub recent_case_tiles: Vec<StudioGuiWindowHomeCaseTileModel>,
    pub example_case_tiles: Vec<StudioGuiWindowHomeCaseTileModel>,
    pub message_tags: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowPropertyPackageModel {
    pub package_id: String,
    pub label: String,
    pub detail: String,
    pub component_summary: String,
    pub command_id: String,
    pub selected: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowPropertyComponentModel {
    pub component_id: String,
    pub name: String,
    pub formula: Option<String>,
    pub selected: bool,
    pub select_command_id: String,
    pub remove_command_id: String,
    pub remove_enabled: bool,
    pub remove_detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowPropertyMetricModel {
    pub label: &'static str,
    pub value: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowPropertyPageModel {
    pub title: &'static str,
    pub selected_package_id: Option<String>,
    pub package_status_label: &'static str,
    pub selected_component_count: usize,
    pub package_choices: Vec<StudioGuiWindowPropertyPackageModel>,
    pub component_choices: Vec<StudioGuiWindowPropertyComponentModel>,
    pub metrics: Vec<StudioGuiWindowPropertyMetricModel>,
    pub future_sections: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowStatusSummaryMetricModel {
    pub label: &'static str,
    pub value: String,
    pub status_label: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowStatusSummaryModel {
    pub title: &'static str,
    pub metrics: Vec<StudioGuiWindowStatusSummaryMetricModel>,
    pub snapshot_consistency_label: &'static str,
    pub snapshot_consistency_detail: String,
}

impl StudioGuiWindowHomeModel {
    pub fn from_runtime(runtime: &StudioGuiWindowRuntimeAreaModel) -> Self {
        Self {
            title: "Home",
            recent_case_tiles: Vec::new(),
            example_case_tiles: runtime
                .example_projects
                .iter()
                .map(example_case_tile_from_model)
                .collect(),
            message_tags: vec!["Auth", "Examples", "Cache"],
        }
    }
}

impl StudioGuiWindowPropertyPageModel {
    pub fn from_runtime(runtime: &StudioGuiWindowRuntimeAreaModel) -> Self {
        let document = &runtime.workspace_document;
        let selected_component_count = document
            .project_component_choices
            .iter()
            .filter(|component| component.selected)
            .count();
        let selected_package = document.property_package_id.clone();
        let selected_package_component_summary = document
            .property_package_choices
            .iter()
            .find(|choice| choice.selected)
            .map(|choice| choice.component_summary.clone())
            .unwrap_or_else(|| "Unselected".to_string());

        Self {
            title: "Property",
            selected_package_id: selected_package.clone(),
            package_status_label: if selected_package.is_some() {
                "Selected"
            } else {
                "Unselected"
            },
            selected_component_count,
            package_choices: document
                .property_package_choices
                .iter()
                .map(|choice| StudioGuiWindowPropertyPackageModel {
                    package_id: choice.package_id.clone(),
                    label: choice.label.clone(),
                    detail: choice.detail.clone(),
                    component_summary: choice.component_summary.clone(),
                    command_id: choice.command_id.clone(),
                    selected: choice.selected,
                    enabled: choice.enabled,
                })
                .collect(),
            component_choices: document
                .project_component_choices
                .iter()
                .map(|component| StudioGuiWindowPropertyComponentModel {
                    component_id: component.component_id.clone(),
                    name: component.name.clone(),
                    formula: component.formula.clone(),
                    selected: component.selected,
                    select_command_id: component.select_command_id.clone(),
                    remove_command_id: component.remove_command_id.clone(),
                    remove_enabled: component.remove_enabled,
                    remove_detail: component.remove_detail.clone(),
                })
                .collect(),
            metrics: vec![
                StudioGuiWindowPropertyMetricModel {
                    label: "Package",
                    value: selected_package.unwrap_or_else(|| "Unselected".to_string()),
                    detail: selected_package_component_summary,
                },
                StudioGuiWindowPropertyMetricModel {
                    label: "Components",
                    value: selected_component_count.to_string(),
                    detail: "Project component selection stored in the flowsheet document."
                        .to_string(),
                },
                StudioGuiWindowPropertyMetricModel {
                    label: "Source",
                    value: "Built-in".to_string(),
                    detail: "MVP scope only exposes controlled built-in property assets."
                        .to_string(),
                },
            ],
            future_sections: vec!["Parameters", "Analysis", "Sources"],
        }
    }
}

impl StudioGuiWindowStatusSummaryModel {
    pub fn from_runtime(runtime: &StudioGuiWindowRuntimeAreaModel) -> Self {
        let document = &runtime.workspace_document;
        let run_panel_view = runtime.run_panel.view();
        let case_value = if document.has_unsaved_changes {
            "Modified"
        } else {
            "Saved"
        };
        let convergence_value = runtime
            .latest_solve_snapshot
            .as_ref()
            .map(|snapshot| snapshot.status_label.to_string())
            .or_else(|| {
                runtime
                    .latest_failure
                    .as_ref()
                    .map(|failure| failure.status_label.to_string())
            })
            .or_else(|| {
                runtime
                    .stale_solve_snapshot
                    .as_ref()
                    .map(|_| "Stale".to_string())
            })
            .unwrap_or_else(|| "N/A".to_string());
        let step_value = runtime
            .latest_solve_snapshot
            .as_ref()
            .map(|snapshot| snapshot.step_count.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let diagnostic_value = runtime
            .latest_solve_snapshot
            .as_ref()
            .map(|snapshot| snapshot.diagnostic_count)
            .or_else(|| {
                runtime
                    .latest_failure
                    .as_ref()
                    .and_then(|failure| failure.diagnostic_detail.as_ref())
                    .map(|detail| detail.diagnostic_count)
            })
            .unwrap_or(0)
            .to_string();
        let (snapshot_label, snapshot_detail) = snapshot_consistency(runtime, document.revision);

        Self {
            title: "Status Summary",
            metrics: vec![
                StudioGuiWindowStatusSummaryMetricModel {
                    label: "Case",
                    value: case_value.to_string(),
                    status_label: case_value.to_string(),
                    detail: format!(
                        "document rev {}{}",
                        document.revision,
                        document
                            .last_saved_revision
                            .map(|revision| format!(", saved rev {revision}"))
                            .unwrap_or_else(|| ", never saved".to_string())
                    ),
                },
                StudioGuiWindowStatusSummaryMetricModel {
                    label: "Run",
                    value: run_panel_view.status_label.to_string(),
                    status_label: run_panel_view.status_label.to_string(),
                    detail: run_panel_view
                        .pending_label
                        .map(|label| format!("pending: {label}"))
                        .unwrap_or_else(|| "no pending run blocker in run panel".to_string()),
                },
                StudioGuiWindowStatusSummaryMetricModel {
                    label: "Convergence",
                    value: convergence_value.clone(),
                    status_label: convergence_value,
                    detail: "Current revision latest result, formal failure, or stale notice."
                        .to_string(),
                },
                StudioGuiWindowStatusSummaryMetricModel {
                    label: "Steps",
                    value: step_value,
                    status_label: "Sequential steps".to_string(),
                    detail: "Sequential modular solve steps; no fabricated iteration count."
                        .to_string(),
                },
                StudioGuiWindowStatusSummaryMetricModel {
                    label: "Diagnostics",
                    value: diagnostic_value,
                    status_label: "Diagnostics".to_string(),
                    detail: "Diagnostics from the current result or formal run failure."
                        .to_string(),
                },
            ],
            snapshot_consistency_label: snapshot_label,
            snapshot_consistency_detail: snapshot_detail,
        }
    }
}

fn example_case_tile_from_model(
    example: &StudioExampleProjectModel,
) -> StudioGuiWindowHomeCaseTileModel {
    let status = if !example.project_path.exists() {
        StudioGuiWindowHomeCaseTileStatus::MissingFile
    } else if example.is_current {
        StudioGuiWindowHomeCaseTileStatus::Current
    } else {
        StudioGuiWindowHomeCaseTileStatus::Ready
    };

    StudioGuiWindowHomeCaseTileModel {
        source: StudioGuiWindowHomeCaseTileSource::Example,
        source_id: example.id.to_string(),
        title: example.title.to_string(),
        detail: example.detail.to_string(),
        path_text: example.project_path.display().to_string(),
        package_summary: "binary-hydrocarbon-lite-v1".to_string(),
        component_summary: "Methane, Ethane".to_string(),
        status,
        status_label: home_case_tile_status_label(status),
        thumbnail: thumbnail_from_case_title(example.title),
    }
}

fn thumbnail_from_case_title(title: &str) -> StudioGuiWindowThumbnailFlowModel {
    let stages = title
        .split("->")
        .map(|stage| {
            stage
                .split('+')
                .map(str::trim)
                .filter(|node| !node.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|stage| !stage.is_empty())
        .collect::<Vec<_>>();
    let mut nodes = Vec::new();
    let mut stage_indices: Vec<Vec<usize>> = Vec::new();
    for stage in stages {
        let indices = stage
            .into_iter()
            .map(|node| {
                let index = nodes.len();
                nodes.push(node);
                index
            })
            .collect::<Vec<_>>();
        stage_indices.push(indices);
    }

    let mut edges = Vec::new();
    for pair in stage_indices.windows(2) {
        for from in &pair[0] {
            for to in &pair[1] {
                edges.push((*from, *to));
            }
        }
    }

    StudioGuiWindowThumbnailFlowModel { nodes, edges }
}

fn home_case_tile_status_label(status: StudioGuiWindowHomeCaseTileStatus) -> &'static str {
    match status {
        StudioGuiWindowHomeCaseTileStatus::Ready => "Ready",
        StudioGuiWindowHomeCaseTileStatus::Current => "Current",
        StudioGuiWindowHomeCaseTileStatus::MissingFile => "Missing file",
    }
}

fn snapshot_consistency(
    runtime: &StudioGuiWindowRuntimeAreaModel,
    current_revision: u64,
) -> (&'static str, String) {
    if let Some(snapshot) = runtime.latest_solve_snapshot.as_ref() {
        return (
            "Current",
            format!(
                "snapshot {} sequence {} matches document rev {}",
                snapshot.snapshot_id, snapshot.sequence, snapshot.document_revision
            ),
        );
    }

    if let Some(stale) = runtime.stale_solve_snapshot.as_ref() {
        return (
            "Stale",
            format!(
                "snapshot {} was produced at rev {}, current document rev {}",
                stale.snapshot_id,
                stale.snapshot_document_revision,
                stale.current_document_revision
            ),
        );
    }

    (
        "None",
        format!("no solve snapshot is current for document rev {current_revision}"),
    )
}
