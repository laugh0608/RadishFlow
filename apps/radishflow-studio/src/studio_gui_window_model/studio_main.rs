use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiWindowHomeCaseTileSource {
    Current,
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
    pub flowsheet_modeling_enabled: bool,
    pub flowsheet_modeling_status_label: &'static str,
    pub flowsheet_modeling_detail: String,
    pub package_choices: Vec<StudioGuiWindowPropertyPackageModel>,
    pub component_choices: Vec<StudioGuiWindowPropertyComponentModel>,
    pub metrics: Vec<StudioGuiWindowPropertyMetricModel>,
    pub future_sections: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiWindowModuleSettingsState {
    NoUnitSelected,
    UnitDetailUnavailable,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowModuleSettingsParameterSummaryModel {
    pub title: &'static str,
    pub status_label: &'static str,
    pub detail: String,
    pub total_field_count: usize,
    pub dirty_field_count: usize,
    pub issue_count: usize,
    pub notice_count: usize,
    pub batch_commit_available: bool,
    pub batch_discard_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowModuleSettingsModel {
    pub title: &'static str,
    pub state: StudioGuiWindowModuleSettingsState,
    pub state_label: &'static str,
    pub detail: String,
    pub selected_unit: Option<StudioGuiWindowInspectorTargetModel>,
    pub summary_rows: Vec<StudioGuiWindowInspectorTargetSummaryRowModel>,
    pub parameter_summary: Option<StudioGuiWindowModuleSettingsParameterSummaryModel>,
    pub parameter_fields: Vec<StudioGuiWindowInspectorTargetFieldModel>,
    pub parameter_notices: Vec<StudioGuiWindowInspectorPropertyNoticeModel>,
    pub parameter_batch_commit_command_id: Option<String>,
    pub parameter_batch_discard_command_id: Option<String>,
    pub connection_actions: Vec<StudioGuiWindowInspectorConnectionActionModel>,
    pub ports: Vec<StudioGuiWindowInspectorTargetPortModel>,
    pub related_diagnostics: Vec<StudioGuiWindowDiagnosticModel>,
    pub diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
    pub help_actions: Vec<StudioGuiWindowCommandActionModel>,
    pub help_detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiWindowModuleResultsState {
    NoUnitSelected,
    NoCurrentResult,
    Stale,
    NoUnitResult,
    Current,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowModuleResultsModel {
    pub title: &'static str,
    pub state: StudioGuiWindowModuleResultsState,
    pub state_label: &'static str,
    pub detail: String,
    pub selected_unit: Option<StudioGuiWindowInspectorTargetModel>,
    pub snapshot_id: Option<String>,
    pub stale_snapshot: Option<StudioGuiWindowStaleSolveSnapshotModel>,
    pub selected_unit_result: Option<StudioGuiWindowUnitExecutionResultModel>,
    pub consumed_stream_chips: Vec<StudioGuiWindowStreamResultReferenceModel>,
    pub produced_stream_chips: Vec<StudioGuiWindowStreamResultReferenceModel>,
    pub related_steps: Vec<StudioGuiWindowSolveStepModel>,
    pub related_diagnostics: Vec<StudioGuiWindowDiagnosticModel>,
    pub diagnostic_actions: Vec<StudioGuiWindowDiagnosticTargetActionModel>,
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
        let flowsheet_modeling_enabled = selected_package.is_some() && selected_component_count > 0;

        Self {
            title: "Property",
            selected_package_id: selected_package.clone(),
            package_status_label: if selected_package.is_some() {
                "Selected"
            } else {
                "Unselected"
            },
            selected_component_count,
            flowsheet_modeling_enabled,
            flowsheet_modeling_status_label: if flowsheet_modeling_enabled {
                "Ready"
            } else {
                "Incomplete"
            },
            flowsheet_modeling_detail: if flowsheet_modeling_enabled {
                "Property package and project components are selected; continue to flowsheet modeling."
                    .to_string()
            } else {
                "Select a property package and at least one project component before entering flowsheet modeling."
                    .to_string()
            },
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

impl StudioGuiWindowModuleSettingsModel {
    pub fn from_runtime(runtime: &StudioGuiWindowRuntimeAreaModel) -> Self {
        let Some(selected_unit) = active_unit_target(runtime) else {
            return module_settings_empty(
                StudioGuiWindowModuleSettingsState::NoUnitSelected,
                "Select a unit before editing module settings.",
                "No formal help command is available without a selected unit.",
            );
        };

        let Some(detail) = runtime.active_inspector_detail.as_ref().filter(|detail| {
            detail.target.kind_label == "Unit" && detail.target.target_id == selected_unit.target_id
        }) else {
            return StudioGuiWindowModuleSettingsModel {
                title: "Module Settings",
                state: StudioGuiWindowModuleSettingsState::UnitDetailUnavailable,
                state_label: module_settings_state_label(
                    StudioGuiWindowModuleSettingsState::UnitDetailUnavailable,
                ),
                detail: format!(
                    "Inspector detail for {} is not available yet.",
                    selected_unit.target_id
                ),
                selected_unit: Some(selected_unit.clone()),
                summary_rows: Vec::new(),
                parameter_summary: None,
                parameter_fields: Vec::new(),
                parameter_notices: Vec::new(),
                parameter_batch_commit_command_id: None,
                parameter_batch_discard_command_id: None,
                connection_actions: Vec::new(),
                ports: Vec::new(),
                related_diagnostics: Vec::new(),
                diagnostic_actions: Vec::new(),
                help_actions: Vec::new(),
                help_detail: module_settings_help_detail(&selected_unit),
            };
        };

        StudioGuiWindowModuleSettingsModel {
            title: "Module Settings",
            state: StudioGuiWindowModuleSettingsState::Ready,
            state_label: module_settings_state_label(StudioGuiWindowModuleSettingsState::Ready),
            detail: format!(
                "Formal parameter drafts, ports, diagnostics, and commands for {}.",
                detail.target.target_id
            ),
            selected_unit: Some(detail.target.clone()),
            summary_rows: detail.summary_rows.clone(),
            parameter_summary: module_settings_parameter_summary(
                &detail.property_fields,
                &detail.property_notices,
                detail.property_batch_commit_command_id.as_deref(),
                detail.property_batch_discard_command_id.as_deref(),
            ),
            parameter_fields: detail.property_fields.clone(),
            parameter_notices: detail.property_notices.clone(),
            parameter_batch_commit_command_id: detail.property_batch_commit_command_id.clone(),
            parameter_batch_discard_command_id: detail.property_batch_discard_command_id.clone(),
            connection_actions: detail.connection_actions.clone(),
            ports: detail.unit_ports.clone(),
            related_diagnostics: detail.related_diagnostics.clone(),
            diagnostic_actions: detail.diagnostic_actions.clone(),
            help_actions: Vec::new(),
            help_detail: module_settings_help_detail(&detail.target),
        }
    }
}

impl StudioGuiWindowModuleResultsModel {
    pub fn from_runtime(runtime: &StudioGuiWindowRuntimeAreaModel) -> Self {
        let Some(selected_unit) = active_unit_target(runtime) else {
            return module_results_empty(
                StudioGuiWindowModuleResultsState::NoUnitSelected,
                "No unit is selected for module results.",
            );
        };

        if let Some(stale_snapshot) = runtime.stale_solve_snapshot.as_ref() {
            return StudioGuiWindowModuleResultsModel {
                title: "Module Results",
                state: StudioGuiWindowModuleResultsState::Stale,
                state_label: module_results_state_label(StudioGuiWindowModuleResultsState::Stale),
                detail: stale_snapshot.detail.clone(),
                selected_unit: Some(selected_unit),
                snapshot_id: None,
                stale_snapshot: Some(stale_snapshot.clone()),
                selected_unit_result: None,
                consumed_stream_chips: Vec::new(),
                produced_stream_chips: Vec::new(),
                related_steps: Vec::new(),
                related_diagnostics: Vec::new(),
                diagnostic_actions: Vec::new(),
            };
        }

        let Some(snapshot) = runtime.latest_solve_snapshot.as_ref() else {
            return StudioGuiWindowModuleResultsModel {
                title: "Module Results",
                state: StudioGuiWindowModuleResultsState::NoCurrentResult,
                state_label: module_results_state_label(
                    StudioGuiWindowModuleResultsState::NoCurrentResult,
                ),
                detail: format!(
                    "Run the current document before reviewing module results for {}.",
                    selected_unit.target_id
                ),
                selected_unit: Some(selected_unit),
                snapshot_id: None,
                stale_snapshot: None,
                selected_unit_result: None,
                consumed_stream_chips: Vec::new(),
                produced_stream_chips: Vec::new(),
                related_steps: Vec::new(),
                related_diagnostics: Vec::new(),
                diagnostic_actions: Vec::new(),
            };
        };

        let target =
            rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(selected_unit.target_id.as_str()));
        let selected_unit_result = latest_unit_result_for_target(snapshot, &target);
        let related_steps = related_steps_for_target(snapshot, &target);
        let related_diagnostics = related_diagnostics_for_target(snapshot, &target);
        let diagnostic_actions = inspector_detail_diagnostic_actions(
            &selected_unit,
            selected_unit_result.as_ref(),
            &related_steps,
            &related_diagnostics,
        );

        let Some(selected_unit_result) = selected_unit_result else {
            return StudioGuiWindowModuleResultsModel {
                title: "Module Results",
                state: StudioGuiWindowModuleResultsState::NoUnitResult,
                state_label: module_results_state_label(
                    StudioGuiWindowModuleResultsState::NoUnitResult,
                ),
                detail: format!(
                    "Snapshot {} has no solve step for {}.",
                    snapshot.snapshot_id, selected_unit.target_id
                ),
                selected_unit: Some(selected_unit),
                snapshot_id: Some(snapshot.snapshot_id.clone()),
                stale_snapshot: None,
                selected_unit_result: None,
                consumed_stream_chips: Vec::new(),
                produced_stream_chips: Vec::new(),
                related_steps,
                related_diagnostics,
                diagnostic_actions,
            };
        };

        StudioGuiWindowModuleResultsModel {
            title: "Module Results",
            state: StudioGuiWindowModuleResultsState::Current,
            state_label: module_results_state_label(StudioGuiWindowModuleResultsState::Current),
            detail: format!(
                "Snapshot {} step #{} for {}.",
                snapshot.snapshot_id, selected_unit_result.step_index, selected_unit.target_id
            ),
            selected_unit: Some(selected_unit),
            snapshot_id: Some(snapshot.snapshot_id.clone()),
            stale_snapshot: None,
            consumed_stream_chips: selected_unit_result.consumed_stream_results.clone(),
            produced_stream_chips: selected_unit_result.produced_stream_results.clone(),
            selected_unit_result: Some(selected_unit_result),
            related_steps,
            related_diagnostics,
            diagnostic_actions,
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
        let (unit_value, unit_status_label) =
            if let Some(snapshot) = runtime.latest_solve_snapshot.as_ref() {
                (
                    snapshot.review_summary.unit_results.len().to_string(),
                    "Current".to_string(),
                )
            } else {
                ("N/A".to_string(), "N/A".to_string())
            };
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
                    label: "Units",
                    value: unit_value,
                    status_label: unit_status_label,
                    detail: "Unit result count from the current SolveSnapshot review summary."
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

fn active_unit_target(
    runtime: &StudioGuiWindowRuntimeAreaModel,
) -> Option<StudioGuiWindowInspectorTargetModel> {
    runtime
        .active_inspector_detail
        .as_ref()
        .map(|detail| detail.target.clone())
        .or_else(|| runtime.active_inspector_target.clone())
        .filter(|target| target.kind_label == "Unit")
}

fn module_settings_empty(
    state: StudioGuiWindowModuleSettingsState,
    detail: &str,
    help_detail: &str,
) -> StudioGuiWindowModuleSettingsModel {
    StudioGuiWindowModuleSettingsModel {
        title: "Module Settings",
        state,
        state_label: module_settings_state_label(state),
        detail: detail.to_string(),
        selected_unit: None,
        summary_rows: Vec::new(),
        parameter_summary: None,
        parameter_fields: Vec::new(),
        parameter_notices: Vec::new(),
        parameter_batch_commit_command_id: None,
        parameter_batch_discard_command_id: None,
        connection_actions: Vec::new(),
        ports: Vec::new(),
        related_diagnostics: Vec::new(),
        diagnostic_actions: Vec::new(),
        help_actions: Vec::new(),
        help_detail: help_detail.to_string(),
    }
}

fn module_settings_parameter_summary(
    fields: &[StudioGuiWindowInspectorTargetFieldModel],
    notices: &[StudioGuiWindowInspectorPropertyNoticeModel],
    batch_commit_command_id: Option<&str>,
    batch_discard_command_id: Option<&str>,
) -> Option<StudioGuiWindowModuleSettingsParameterSummaryModel> {
    if fields.is_empty() {
        return None;
    }

    let total_field_count = fields.len();
    let dirty_field_count = fields.iter().filter(|field| field.is_dirty).count();
    let invalid_field_count = fields
        .iter()
        .filter(|field| field.status_label == "Invalid")
        .count();
    let invalid_notice_count = notices
        .iter()
        .filter(|notice| notice.status_label == "Invalid")
        .count();
    let issue_count = invalid_field_count + invalid_notice_count;
    let status_label = if issue_count > 0 {
        "Invalid"
    } else if dirty_field_count > 0 {
        "Draft"
    } else {
        "Synced"
    };
    let batch_commit_available =
        batch_commit_command_id.is_some() && dirty_field_count > 0 && issue_count == 0;
    let batch_discard_available = batch_discard_command_id.is_some() && dirty_field_count > 0;
    let detail = if issue_count > 0 {
        format!(
            "{total_field_count} parameter field(s), {dirty_field_count} draft(s), {issue_count} issue(s); fix invalid values before committing."
        )
    } else if dirty_field_count > 0 {
        format!(
            "{total_field_count} parameter field(s), {dirty_field_count} draft(s), ready for batch commit."
        )
    } else {
        format!("{total_field_count} parameter field(s), all synced with the current document.")
    };

    Some(StudioGuiWindowModuleSettingsParameterSummaryModel {
        title: "Parameter Summary",
        status_label,
        detail,
        total_field_count,
        dirty_field_count,
        issue_count,
        notice_count: notices.len(),
        batch_commit_available,
        batch_discard_available,
    })
}

fn module_settings_state_label(state: StudioGuiWindowModuleSettingsState) -> &'static str {
    match state {
        StudioGuiWindowModuleSettingsState::NoUnitSelected => "No unit selected",
        StudioGuiWindowModuleSettingsState::UnitDetailUnavailable => "Unit detail unavailable",
        StudioGuiWindowModuleSettingsState::Ready => "Ready",
    }
}

fn module_settings_help_detail(unit: &StudioGuiWindowInspectorTargetModel) -> String {
    format!(
        "No formal module help command is registered for {} yet.",
        unit.target_id
    )
}

fn module_results_empty(
    state: StudioGuiWindowModuleResultsState,
    detail: &str,
) -> StudioGuiWindowModuleResultsModel {
    StudioGuiWindowModuleResultsModel {
        title: "Module Results",
        state,
        state_label: module_results_state_label(state),
        detail: detail.to_string(),
        selected_unit: None,
        snapshot_id: None,
        stale_snapshot: None,
        selected_unit_result: None,
        consumed_stream_chips: Vec::new(),
        produced_stream_chips: Vec::new(),
        related_steps: Vec::new(),
        related_diagnostics: Vec::new(),
        diagnostic_actions: Vec::new(),
    }
}

fn module_results_state_label(state: StudioGuiWindowModuleResultsState) -> &'static str {
    match state {
        StudioGuiWindowModuleResultsState::NoUnitSelected => "No unit selected",
        StudioGuiWindowModuleResultsState::NoCurrentResult => "No current result",
        StudioGuiWindowModuleResultsState::Stale => "Stale",
        StudioGuiWindowModuleResultsState::NoUnitResult => "No unit result",
        StudioGuiWindowModuleResultsState::Current => "Current",
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
