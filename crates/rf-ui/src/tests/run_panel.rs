use super::*;

#[test]
fn run_panel_view_model_consumes_primary_and_secondary_actions() {
    let app_state = AppState::new(sample_document());

    let view = RunPanelViewModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(view.mode_label, "Hold");
    assert_eq!(view.status_label, "Idle");
    assert_eq!(view.pending_label, Some("Snapshot missing"));
    assert_eq!(view.primary_action.id, RunPanelActionId::Resume);
    assert_eq!(view.primary_action.label, "Resume");
    assert_eq!(
        view.primary_action.detail,
        "Resume pending work while the workspace stays in Hold mode"
    );
    assert_eq!(
        view.primary_action.prominence,
        RunPanelActionProminence::Primary
    );
    assert_eq!(view.secondary_actions.len(), 3);
    assert_eq!(view.secondary_actions[0].id, RunPanelActionId::RunManual);
    assert_eq!(view.secondary_actions[1].id, RunPanelActionId::SetHold);
    assert_eq!(view.secondary_actions[2].id, RunPanelActionId::SetActive);
}

#[test]
fn run_panel_text_view_renders_primary_and_secondary_actions() {
    let app_state = AppState::new(sample_document());
    let view = RunPanelViewModel::from_state(&app_state.workspace.run_panel);

    let text = RunPanelTextView::from_view_model(&view);

    assert_eq!(text.title, "Run panel");
    assert_eq!(text.lines[0], "Mode: Hold");
    assert_eq!(text.lines[1], "Status: Idle");
    assert!(
        text.lines
            .iter()
            .any(|line| line == "Pending: Snapshot missing")
    );
    assert!(
        text.lines
            .iter()
            .any(|line| line == "Primary action: Resume [enabled]")
    );
    assert!(text.lines.iter().any(|line| {
        line == "Primary detail: Resume pending work while the workspace stays in Hold mode"
    }));
    assert!(
        text.lines
            .iter()
            .any(|line| { line == "  - Run [enabled] | Run the current workspace once" })
    );
}

#[test]
fn run_panel_text_view_renders_notice_when_present() {
    let mut state = AppState::new(sample_document()).workspace.run_panel;
    state.notice = Some(
        crate::RunPanelNotice::new(
            crate::RunPanelNoticeLevel::Warning,
            "Run blocked",
            "explicit package selection is required",
        )
        .with_recovery_action(crate::RunPanelRecoveryAction::new(
            crate::RunPanelRecoveryActionKind::InspectFailureDetails,
            "Choose package",
            "选择明确的 property package 后再运行。",
        )),
    );

    let view = RunPanelViewModel::from_state(&state);
    let text = RunPanelTextView::from_view_model(&view);

    assert!(
        text.lines
            .iter()
            .any(|line| line == "Notice: Run blocked [warning]")
    );
    assert!(
        text.lines
            .iter()
            .any(|line| line == "Notice detail: explicit package selection is required")
    );
    assert!(
        text.lines
            .iter()
            .any(|line| line == "Suggested action: Choose package")
    );
    assert!(
        text.lines
            .iter()
            .any(|line| line == "Suggested detail: 选择明确的 property package 后再运行。")
    );
    assert!(
        !text
            .lines
            .iter()
            .any(|line| line.starts_with("Suggested target: unit "))
    );
}

#[test]
fn run_panel_view_model_returns_dispatchable_intents_for_enabled_actions() {
    let app_state = AppState::new(sample_document());
    let view = RunPanelViewModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        view.dispatchable_primary_intent(),
        Some(crate::RunPanelIntent::resume(
            crate::RunPanelPackageSelection::preferred()
        ))
    );
    assert_eq!(
        view.dispatchable_intent(RunPanelActionId::RunManual),
        Some(crate::RunPanelIntent::run_manual(
            crate::RunPanelPackageSelection::preferred()
        ))
    );
    assert_eq!(view.dispatchable_intent(RunPanelActionId::SetHold), None);
}

#[test]
fn run_panel_presentation_combines_view_text_and_dispatchable_intents() {
    let app_state = AppState::new(sample_document());

    let presentation = RunPanelPresentation::from_state(&app_state.workspace.run_panel);

    assert_eq!(presentation.view.primary_action.label, "Resume");
    assert_eq!(presentation.text.title, "Run panel");
    assert_eq!(
        presentation.dispatchable_primary_intent(),
        Some(crate::RunPanelIntent::resume(
            crate::RunPanelPackageSelection::preferred()
        ))
    );
}

#[test]
fn run_panel_widget_dispatches_primary_action_and_blocks_disabled_actions() {
    let app_state = AppState::new(sample_document());

    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        widget.activate_primary(),
        RunPanelWidgetEvent::Dispatched {
            action_id: RunPanelActionId::Resume,
            intent: crate::RunPanelIntent::resume(crate::RunPanelPackageSelection::preferred()),
        }
    );
    assert_eq!(
        widget.activate(RunPanelActionId::SetHold),
        RunPanelWidgetEvent::Disabled {
            action_id: RunPanelActionId::SetHold,
            detail: "Workspace is already in Hold mode",
        }
    );
}

#[test]
fn run_panel_widget_exposes_recovery_action_when_solver_failure_targets_unit() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.step.inlet: solver step 1 inlet resolution failed for unit `heater-1`",
    )
    .with_primary_code("solver.step.inlet")
    .with_related_unit_ids(vec![UnitId::new("heater-1")]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        widget.activate_recovery_action(),
        RunPanelRecoveryWidgetEvent::Requested {
            action: crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::InspectInletPath,
                "Inspect inlet path",
                "检查入口连接是否完整，以及上游流股是否应先于该单元求解。",
            )
            .with_target_unit(UnitId::new("heater-1")),
        }
    );
    assert!(
        widget
            .text()
            .lines
            .iter()
            .any(|line| line == "Suggested target: unit heater-1")
    );
}

#[test]
fn run_panel_widget_exposes_recovery_action_when_connection_failure_targets_disconnectable_port() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.duplicate_downstream_sink: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.duplicate_downstream_sink")
    .with_related_unit_ids(vec![UnitId::new("flash-1"), UnitId::new("mixer-1")])
    .with_related_stream_ids(vec![rf_types::StreamId::new("shared-stream")])
    .with_related_port_targets(vec![
        rf_types::DiagnosticPortTarget::new("flash-1", "inlet"),
        rf_types::DiagnosticPortTarget::new("mixer-1", "inlet_a"),
    ]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        widget.activate_recovery_action(),
        RunPanelRecoveryWidgetEvent::Requested {
            action: crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::FixConnections,
                "Disconnect conflicting sink",
                "断开冲突的 inlet sink 端口，让该流股只保留一个下游去向后再继续修复。",
            )
            .with_disconnect_port(UnitId::new("mixer-1"), "inlet_a"),
        }
    );
    assert!(
        widget
            .text()
            .lines
            .iter()
            .any(|line| line == "Suggested target: unit mixer-1 port inlet_a")
    );
}

#[test]
fn recording_duplicate_upstream_source_prefers_last_conflicting_port_for_disconnect() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.duplicate_upstream_source: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.duplicate_upstream_source")
    .with_related_unit_ids(vec![UnitId::new("feed-1"), UnitId::new("heater-1")])
    .with_related_stream_ids(vec![rf_types::StreamId::new("shared-stream")])
    .with_related_port_targets(vec![
        rf_types::DiagnosticPortTarget::new("feed-1", "outlet"),
        rf_types::DiagnosticPortTarget::new("heater-1", "outlet"),
    ]);

    app_state.record_failure(0, RunStatus::Error, summary);

    assert_eq!(
        app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref()),
        Some(
            &crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::FixConnections,
                "Disconnect conflicting source",
                "断开冲突的 outlet source 端口，让该流股只保留一个上游来源后再继续修复。",
            )
            .with_disconnect_port(UnitId::new("heater-1"), "outlet")
        )
    );
}

#[test]
fn run_panel_widget_exposes_recovery_action_when_orphan_stream_targets_deletable_stream() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.orphan_stream: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.orphan_stream")
    .with_related_stream_ids(vec![rf_types::StreamId::new("stream-orphan")]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        widget.activate_recovery_action(),
        RunPanelRecoveryWidgetEvent::Requested {
            action: crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::FixConnections,
                "Delete orphan stream",
                "删除当前未连接到任何单元端口的孤立流股，避免它继续阻塞连接校验。",
            )
            .with_delete_stream(rf_types::StreamId::new("stream-orphan")),
        }
    );
    assert!(
        widget
            .text()
            .lines
            .iter()
            .any(|line| line == "Suggested target: stream stream-orphan")
    );
}

#[test]
fn run_panel_widget_exposes_recovery_action_when_unbound_outlet_targets_stream_creation() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.unbound_outlet_port: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.unbound_outlet_port")
    .with_related_unit_ids(vec![UnitId::new("feed-1")])
    .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
        "feed-1", "outlet",
    )]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        widget.activate_recovery_action(),
        RunPanelRecoveryWidgetEvent::Requested {
            action: crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::FixConnections,
                "Create outlet stream",
                "为当前未绑定 stream 的 outlet 端口创建一条占位流股，并立即写回连接。",
            )
            .with_create_and_bind_outlet_stream(UnitId::new("feed-1"), "outlet"),
        }
    );
    assert!(
        widget
            .text()
            .lines
            .iter()
            .any(|line| line == "Suggested target: unit feed-1 port outlet")
    );
}

#[test]
fn run_panel_widget_exposes_recovery_action_when_self_loop_targets_disconnectable_port() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.topological_ordering.self_loop_cycle: solver topological ordering failed",
    )
    .with_primary_code("solver.topological_ordering.self_loop_cycle")
    .with_related_unit_ids(vec![UnitId::new("flash-1")])
    .with_related_stream_ids(vec![rf_types::StreamId::new("stream-loop")])
    .with_related_port_targets(vec![
        rf_types::DiagnosticPortTarget::new("flash-1", "inlet"),
        rf_types::DiagnosticPortTarget::new("flash-1", "liquid"),
    ]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        widget.activate_recovery_action(),
        RunPanelRecoveryWidgetEvent::Requested {
            action: crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::BreakCycle,
                "Disconnect self-loop inlet",
                "断开当前单元引用自身 outlet stream 的 inlet 端口，先消除自环依赖，再继续检查剩余连接问题。",
            )
            .with_disconnect_port(UnitId::new("flash-1"), "inlet"),
        }
    );
    assert!(
        widget
            .text()
            .lines
            .iter()
            .any(|line| line == "Suggested target: unit flash-1 port inlet")
    );
}

#[test]
fn run_panel_widget_exposes_recovery_action_when_two_unit_cycle_targets_disconnectable_port() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.topological_ordering.two_unit_cycle: solver topological ordering failed",
    )
    .with_primary_code("solver.topological_ordering.two_unit_cycle")
    .with_related_unit_ids(vec![UnitId::new("heater-1"), UnitId::new("valve-1")])
    .with_related_stream_ids(vec![
        rf_types::StreamId::new("stream-a"),
        rf_types::StreamId::new("stream-b"),
    ])
    .with_related_port_targets(vec![
        rf_types::DiagnosticPortTarget::new("valve-1", "inlet"),
        rf_types::DiagnosticPortTarget::new("heater-1", "inlet"),
    ]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        widget.activate_recovery_action(),
        RunPanelRecoveryWidgetEvent::Requested {
            action: crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::BreakCycle,
                "Disconnect cycle inlet",
                "断开当前双单元回路中的一个 inlet 端口，先打破互相依赖，再继续检查剩余连接问题。",
            )
            .with_disconnect_port(UnitId::new("heater-1"), "inlet"),
        }
    );
    assert!(
        widget
            .text()
            .lines
            .iter()
            .any(|line| line == "Suggested target: unit heater-1 port inlet")
    );
}

#[test]
fn run_panel_widget_exposes_recovery_action_when_connection_failure_targets_port() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.missing_upstream_source: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.missing_upstream_source")
    .with_related_unit_ids(vec![UnitId::new("mixer-1")])
    .with_related_stream_ids(vec![rf_types::StreamId::new("stream-feed-a")])
    .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
        "mixer-1", "inlet_a",
    )]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

    assert_eq!(
        widget.activate_recovery_action(),
        RunPanelRecoveryWidgetEvent::Requested {
            action: crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::FixConnections,
                "Remove dangling inlet stream",
                "断开当前缺少上游 source 的 inlet 绑定，并删除对应孤立流股，先消除悬空入口连接。",
            )
            .with_disconnect_port_and_delete_stream(
                UnitId::new("mixer-1"),
                "inlet_a",
                rf_types::StreamId::new("stream-feed-a"),
            ),
        }
    );
    assert!(
        widget
            .text()
            .lines
            .iter()
            .any(|line| line == "Suggested target: unit mixer-1 port inlet_a")
    );
}

#[test]
fn storing_snapshot_updates_run_panel_summary() {
    let mut app_state = AppState::new(sample_document());
    let snapshot = SolveSnapshot::new(
        "snapshot-ui-1",
        0,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(0, DiagnosticSeverity::Info, "snapshot ok"),
    );

    app_state.store_snapshot(snapshot);

    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("snapshot-ui-1")
    );
    assert_eq!(
        app_state
            .workspace
            .run_panel
            .latest_snapshot_summary
            .as_deref(),
        Some("snapshot ok")
    );
    assert_eq!(
        app_state.workspace.run_panel.run_status,
        RunStatus::Converged
    );
}

#[test]
fn recording_failure_updates_run_panel_status_and_log_message() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(0, DiagnosticSeverity::Error, "solve failed");

    app_state.push_log(crate::AppLogLevel::Error, "solver failed");
    app_state.record_failure(0, RunStatus::Error, summary);

    assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Error);
    assert_eq!(
        app_state.workspace.run_panel.latest_log_message.as_deref(),
        Some("solver failed")
    );
    assert_eq!(
        app_state.workspace.run_panel.commands.primary_action,
        RunPanelActionId::RunManual
    );
    assert_eq!(
        app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Run failed")
    );
    assert!(
        !app_state
            .workspace
            .run_panel
            .commands
            .action(RunPanelActionId::Resume)
            .expect("expected resume action")
            .enabled
    );
}

#[test]
fn recording_failure_clears_current_snapshot_for_same_revision() {
    let mut app_state = AppState::new(sample_document());
    app_state.store_snapshot(SolveSnapshot::new(
        "snapshot-success-1",
        0,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(0, DiagnosticSeverity::Info, "snapshot ok"),
    ));

    app_state.record_failure(
        0,
        RunStatus::Error,
        DiagnosticSummary::new(0, DiagnosticSeverity::Error, "solve failed again"),
    );

    assert_eq!(app_state.workspace.solve_session.latest_snapshot, None);
    assert!(latest_snapshot(&app_state.workspace).is_none());
    assert_eq!(app_state.workspace.run_panel.latest_snapshot_id, None);
    assert_eq!(app_state.workspace.run_panel.latest_snapshot_summary, None);
    assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Error);
    assert_eq!(
        app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Run failed")
    );
}

#[test]
fn recording_solver_failure_uses_primary_code_for_run_panel_notice_title() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.step.execution: solver step 2 unit execution failed",
    )
    .with_primary_code("solver.step.execution")
    .with_related_unit_ids(vec![UnitId::new("heater-1")]);

    app_state.record_failure(0, RunStatus::Error, summary);

    assert_eq!(
        app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Unit execution failed")
    );
    assert_eq!(
        app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .map(|action| {
                (
                    action.title,
                    action.detail,
                    action
                        .target_unit_id
                        .as_ref()
                        .map(|unit_id| unit_id.as_str()),
                )
            }),
        Some((
            "Inspect unit inputs",
            "检查单元规格、物性条件和入口状态是否满足执行前提。",
            Some("heater-1"),
        ))
    );
}

#[test]
fn recording_connection_validation_subcode_refines_run_panel_notice_and_recovery() {
    let mut app_state = AppState::new(sample_document());
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.invalid_port_signature: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.invalid_port_signature")
    .with_related_unit_ids(vec![UnitId::new("feed-1")]);

    app_state.record_failure(0, RunStatus::Error, summary);

    assert_eq!(
        app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Invalid port signature")
    );
    assert_eq!(
        app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .map(|action| {
                (
                    action.title,
                    action.detail,
                    action
                        .target_unit_id
                        .as_ref()
                        .map(|unit_id| unit_id.as_str()),
                    action.mutation.clone(),
                )
            }),
        Some((
            "Restore canonical ports",
            "按当前内建 unit kind 的 canonical spec 重建端口签名，并尽量保留可匹配的现有 stream 绑定。",
            Some("feed-1"),
            Some(
                crate::RunPanelRecoveryMutation::RestoreCanonicalPortSignature {
                    unit_id: UnitId::new("feed-1"),
                }
            ),
        ))
    );
}

#[test]
fn applying_run_panel_recovery_action_restores_canonical_ports_and_preserves_stream_binding() {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/failures/invalid-port-signature.rfproj.json"
    ))
    .expect("expected project parse");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(
            "doc-invalid-port-signature-recovery",
            "Invalid Port Signature Recovery Demo",
            timestamp(39),
        ),
    ));
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.connection_validation.invalid_port_signature: solver connection validation failed",
    )
    .with_primary_code("solver.connection_validation.invalid_port_signature")
    .with_related_unit_ids(vec![UnitId::new("feed-1")]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    assert_eq!(
        action.mutation,
        Some(
            crate::RunPanelRecoveryMutation::RestoreCanonicalPortSignature {
                unit_id: UnitId::new("feed-1"),
            }
        )
    );

    let applied_target = app_state.apply_run_panel_recovery_action(&action);

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::RestoreCanonicalUnitPorts {
            unit_id: UnitId::new("feed-1"),
        })
    );
    let feed = app_state
        .workspace
        .document
        .flowsheet
        .units
        .get(&UnitId::new("feed-1"))
        .expect("expected feed unit");
    assert_eq!(feed.ports.len(), 1);
    assert_eq!(feed.ports[0].name, "outlet");
    assert_eq!(
        feed.ports[0]
            .stream_id
            .as_ref()
            .map(|stream_id| stream_id.as_str()),
        Some("stream-feed")
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
}
