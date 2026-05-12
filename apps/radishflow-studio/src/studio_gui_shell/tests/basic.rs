use super::*;
use radishflow_studio::test_support::{
    apply_official_binary_hydrocarbon_near_boundary_consumer_scenario,
    build_official_binary_hydrocarbon_provider,
    official_binary_hydrocarbon_near_boundary_consumer_scenarios,
    solve_snapshot_model_from_project_with_provider_and_edit,
};

#[test]
fn insert_neighbors_from_area_ids_returns_previous_and_next_for_middle_target() {
    let area_ids = [
        StudioGuiWindowAreaId::Commands,
        StudioGuiWindowAreaId::Canvas,
        StudioGuiWindowAreaId::Runtime,
    ];

    let (previous, next) = insert_neighbors_from_area_ids(&area_ids, 1);

    assert_eq!(previous, Some(StudioGuiWindowAreaId::Commands));
    assert_eq!(next, Some(StudioGuiWindowAreaId::Runtime));
}

#[test]
fn insert_neighbors_from_area_ids_clamps_to_stack_end() {
    let area_ids = [
        StudioGuiWindowAreaId::Commands,
        StudioGuiWindowAreaId::Canvas,
    ];

    let (previous, next) = insert_neighbors_from_area_ids(&area_ids, 8);

    assert_eq!(previous, Some(StudioGuiWindowAreaId::Commands));
    assert_eq!(next, None);
}

#[test]
fn clamp_overlay_pos_to_rect_keeps_overlay_inside_screen_padding() {
    let screen = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(200.0, 120.0));
    let size = egui::vec2(80.0, 40.0);

    let clamped = clamp_overlay_pos_to_rect(screen, egui::pos2(180.0, 110.0), size);

    assert_eq!(clamped, egui::pos2(112.0, 72.0));
}

#[test]
fn shell_locale_defaults_to_chinese_and_can_translate_runtime_labels() {
    let locale = StudioShellLocale::default();

    assert_eq!(locale, StudioShellLocale::ZhCn);
    assert_eq!(locale.text(ShellText::Runtime), "运行");
    assert_eq!(locale.runtime_label("Converged").as_ref(), "已收敛");
    assert_eq!(
        locale.workspace_counts("Demo", 2, 3, 1),
        "Demo | 2 个单元 | 3 股流股 | 1 个快照"
    );
    assert_eq!(
        locale.solve_snapshot_counts(3, 4, 1),
        "3 股流股，4 个步骤，1 条诊断"
    );
    assert_eq!(
        locale.snapshot_identity("snapshot-a", 7),
        "快照 snapshot-a，序号 7"
    );
    assert_eq!(locale.text(ShellText::ResultInspector), "结果检查器");
    assert_eq!(locale.text(ShellText::StreamComparison), "流股对比");
    assert_eq!(locale.text(ShellText::Delta), "差值");
    assert_eq!(locale.text(ShellText::DiagnosticTargets), "诊断目标");
    assert_eq!(
        locale.text(ShellText::StaleStreamSelection),
        "已选流股不在最新快照中。"
    );
    assert_eq!(locale.text(ShellText::LastRunFailed), "最近一次运行失败");
    assert_eq!(locale.text(ShellText::SuggestedRecovery), "建议修复");
    assert_eq!(locale.text(ShellText::ActiveInspectorTarget), "检查器目标");
    assert_eq!(locale.text(ShellText::InspectorProperties), "属性");
    assert_eq!(locale.text(ShellText::BubbleDewWindow), "泡点/露点窗口");
    assert_eq!(locale.runtime_label("Number").as_ref(), "数值");
    assert_eq!(locale.runtime_label("Synced").as_ref(), "已同步");
    assert_eq!(locale.runtime_label("Temperature").as_ref(), "温度");
    assert_eq!(locale.runtime_label("Pressure").as_ref(), "压力");
    assert_eq!(locale.runtime_label("Molar flow").as_ref(), "摩尔流量");
    assert_eq!(locale.runtime_label("Molar enthalpy").as_ref(), "摩尔焓");
    assert_eq!(locale.runtime_label("Phase region").as_ref(), "相区");
    assert_eq!(locale.runtime_label("Bubble pressure").as_ref(), "泡点压力");
    assert_eq!(locale.runtime_label("Dew pressure").as_ref(), "露点压力");
    assert_eq!(
        locale.runtime_label("Bubble temperature").as_ref(),
        "泡点温度"
    );
    assert_eq!(locale.runtime_label("Dew temperature").as_ref(), "露点温度");
    assert_eq!(locale.runtime_label("two_phase").as_ref(), "两相");
    assert_eq!(locale.text(ShellText::InspectorPorts), "端口");
    assert_eq!(locale.text(ShellText::InspectorConsumedStreams), "消费流股");
    assert_eq!(locale.text(ShellText::InspectorProducedStreams), "产出流股");
    assert_eq!(locale.runtime_label("Unit").as_ref(), "单元");
    assert_eq!(locale.runtime_label("Stream").as_ref(), "流股");
    assert_eq!(
        StudioShellLocale::En.runtime_label("Converged").as_ref(),
        "Converged"
    );
}

#[test]
fn result_inspector_state_tracks_selected_stream_per_snapshot() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    let snapshot = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .expect("expected solve snapshot");

    let default_selected = app
        .result_inspector
        .selected_stream_id_for_snapshot(&snapshot);
    assert_eq!(
        default_selected.as_deref(),
        snapshot
            .streams
            .first()
            .map(|stream| stream.stream_id.as_str())
    );

    app.result_inspector
        .select_stream(&snapshot.snapshot_id, "stream-heated");
    app.result_inspector
        .select_comparison_stream(&snapshot.snapshot_id, "stream-feed");
    assert_eq!(
        app.result_inspector
            .selected_stream_id_for_snapshot(&snapshot)
            .as_deref(),
        Some("stream-heated")
    );
    assert_eq!(
        app.result_inspector.comparison_stream_id.as_deref(),
        Some("stream-feed")
    );

    let mut next_snapshot = snapshot.clone();
    next_snapshot.snapshot_id = "snapshot-next".to_string();
    assert_eq!(
        app.result_inspector
            .selected_stream_id_for_snapshot(&next_snapshot)
            .as_deref(),
        next_snapshot
            .streams
            .first()
            .map(|stream| stream.stream_id.as_str())
    );
    assert_eq!(app.result_inspector.comparison_stream_id, None);
}

#[test]
fn result_inspector_state_tracks_selected_unit_per_snapshot() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    let snapshot = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .expect("expected solve snapshot");

    let default_unit = app
        .result_inspector
        .selected_unit_id_for_snapshot(&snapshot);
    assert_eq!(
        default_unit.as_deref(),
        snapshot.steps.first().map(|step| step.unit_id.as_str())
    );

    app.result_inspector
        .select_unit(&snapshot.snapshot_id, "heater-1");
    assert_eq!(
        app.result_inspector
            .selected_unit_id_for_snapshot(&snapshot)
            .as_deref(),
        Some("heater-1")
    );

    let inspector = snapshot.result_inspector_with_unit(
        snapshot
            .streams
            .first()
            .map(|stream| stream.stream_id.as_str()),
        None,
        Some("heater-1"),
    );
    assert_eq!(inspector.selected_unit_id.as_deref(), Some("heater-1"));
    assert!(
        inspector
            .unit_options
            .iter()
            .any(|option| option.unit_id == "heater-1" && option.is_selected),
        "expected heater-1 unit option to be marked selected"
    );

    // missing unit on the same snapshot falls back to first unit and the
    // shell-side state is reset to that fallback.
    app.result_inspector
        .select_unit(&snapshot.snapshot_id, "missing-unit");
    let fallback_unit = app
        .result_inspector
        .selected_unit_id_for_snapshot(&snapshot);
    assert_eq!(
        fallback_unit.as_deref(),
        snapshot.steps.first().map(|step| step.unit_id.as_str())
    );

    let mut next_snapshot = snapshot.clone();
    next_snapshot.snapshot_id = "snapshot-next-unit".to_string();
    let next_default_unit = app
        .result_inspector
        .selected_unit_id_for_snapshot(&next_snapshot);
    assert_eq!(
        next_default_unit.as_deref(),
        next_snapshot
            .steps
            .first()
            .map(|step| step.unit_id.as_str()),
        "expected unit selection to reset on snapshot identity change"
    );
}

#[test]
fn result_inspector_state_tracks_official_near_boundary_flash_selector_transitions() {
    let mut app = ready_app_state(&synced_workspace_config());
    let provider = build_official_binary_hydrocarbon_provider();

    for scenario in official_binary_hydrocarbon_near_boundary_consumer_scenarios() {
        let snapshot = solve_snapshot_model_from_project_with_provider_and_edit(
            scenario.project_json,
            &provider,
            |project| {
                apply_official_binary_hydrocarbon_near_boundary_consumer_scenario(
                    project, &scenario,
                );
            },
        );

        app.result_inspector
            .select_stream(&snapshot.snapshot_id, "stream-liquid");
        app.result_inspector
            .select_comparison_stream(&snapshot.snapshot_id, "stream-vapor");
        app.result_inspector
            .select_unit(&snapshot.snapshot_id, "flash-1");

        let selected_stream_id = app
            .result_inspector
            .selected_stream_id_for_snapshot(&snapshot);
        let selected_unit_id = app
            .result_inspector
            .selected_unit_id_for_snapshot(&snapshot);
        let comparison_stream_id = app.result_inspector.comparison_stream_id.clone();
        let inspector = snapshot.result_inspector_with_unit(
            selected_stream_id.as_deref(),
            comparison_stream_id.as_deref(),
            selected_unit_id.as_deref(),
        );

        assert_eq!(
            inspector.selected_stream_id.as_deref(),
            Some("stream-liquid"),
            "{}",
            scenario.case.label
        );
        assert_eq!(
            inspector.comparison_stream_id.as_deref(),
            Some("stream-vapor"),
            "{}",
            scenario.case.label
        );
        assert_eq!(
            inspector.selected_unit_id.as_deref(),
            Some("flash-1"),
            "{}",
            scenario.case.label
        );
        assert!(!inspector.has_stale_selection, "{}", scenario.case.label);
        assert!(!inspector.has_stale_comparison, "{}", scenario.case.label);
        assert!(
            !inspector.has_stale_unit_selection,
            "{}",
            scenario.case.label
        );
        assert!(
            inspector.comparison.as_ref().is_some_and(|comparison| {
                comparison.base_stream_id == "stream-liquid"
                    && comparison.compared_stream_id == "stream-vapor"
            }),
            "{}",
            scenario.case.label
        );
        assert!(
            inspector
                .comparison_options
                .iter()
                .any(|option| option.stream_id == "stream-vapor" && option.is_selected),
            "{}",
            scenario.case.label
        );
        assert!(
            inspector
                .unit_options
                .iter()
                .any(|option| option.unit_id == "flash-1" && option.is_selected),
            "{}",
            scenario.case.label
        );

        app.result_inspector
            .select_stream(&snapshot.snapshot_id, "stream-vapor");
        let switched_stream_id = app
            .result_inspector
            .selected_stream_id_for_snapshot(&snapshot);
        let switched_unit_id = app
            .result_inspector
            .selected_unit_id_for_snapshot(&snapshot);
        let switched_comparison_stream_id = app.result_inspector.comparison_stream_id.clone();
        let switched_inspector = snapshot.result_inspector_with_unit(
            switched_stream_id.as_deref(),
            switched_comparison_stream_id.as_deref(),
            switched_unit_id.as_deref(),
        );

        assert_eq!(
            switched_inspector.selected_stream_id.as_deref(),
            Some("stream-vapor"),
            "{}",
            scenario.case.label
        );
        assert_eq!(
            switched_inspector.comparison_stream_id, None,
            "{}",
            scenario.case.label
        );
        assert_eq!(
            switched_inspector.selected_unit_id.as_deref(),
            Some("flash-1"),
            "{}",
            scenario.case.label
        );
        assert_eq!(
            switched_inspector.comparison, None,
            "{}",
            scenario.case.label
        );
        assert!(
            !switched_inspector.has_stale_comparison,
            "{}",
            scenario.case.label
        );
        assert!(
            switched_inspector
                .unit_options
                .iter()
                .any(|option| option.unit_id == "flash-1" && option.is_selected),
            "{}",
            scenario.case.label
        );

        app.result_inspector
            .select_comparison_stream(&snapshot.snapshot_id, "stream-liquid");
        let rearmed_stream_id = app
            .result_inspector
            .selected_stream_id_for_snapshot(&snapshot);
        let rearmed_unit_id = app
            .result_inspector
            .selected_unit_id_for_snapshot(&snapshot);
        let rearmed_comparison_stream_id = app.result_inspector.comparison_stream_id.clone();
        let rearmed_inspector = snapshot.result_inspector_with_unit(
            rearmed_stream_id.as_deref(),
            rearmed_comparison_stream_id.as_deref(),
            rearmed_unit_id.as_deref(),
        );

        assert_eq!(
            rearmed_inspector.selected_stream_id.as_deref(),
            Some("stream-vapor"),
            "{}",
            scenario.case.label
        );
        assert_eq!(
            rearmed_inspector.comparison_stream_id.as_deref(),
            Some("stream-liquid"),
            "{}",
            scenario.case.label
        );
        assert_eq!(
            rearmed_inspector.selected_unit_id.as_deref(),
            Some("flash-1"),
            "{}",
            scenario.case.label
        );
        assert!(
            rearmed_inspector
                .comparison
                .as_ref()
                .is_some_and(|comparison| {
                    comparison.base_stream_id == "stream-vapor"
                        && comparison.compared_stream_id == "stream-liquid"
                }),
            "{}",
            scenario.case.label
        );
        assert!(
            rearmed_inspector
                .comparison_options
                .iter()
                .any(|option| option.stream_id == "stream-liquid" && option.is_selected),
            "{}",
            scenario.case.label
        );
    }
}

#[test]
fn canvas_object_list_filter_matches_expected_object_groups() {
    let unit = radishflow_studio::StudioGuiCanvasObjectListItemViewModel {
        kind_label: "Unit",
        target_id: "flash-1".to_string(),
        label: "Flash Drum".to_string(),
        detail: "flash_drum | ports 1/3".to_string(),
        attention_summary: Some(
            "attention: 1 error(s); ports flash-1:inlet; codes solver.step.execution".to_string(),
        ),
        viewport_anchor_label: "unit-slot-1".to_string(),
        command_id: "inspector.focus_unit:flash-1".to_string(),
        related_stream_ids: Vec::new(),
        status_badges: vec![radishflow_studio::StudioGuiCanvasStatusBadgeViewModel {
            severity_label: "Error",
            short_label: "E1".to_string(),
            detail: "solver.step.execution: unit failed".to_string(),
        }],
        is_active: false,
    };
    let stream = radishflow_studio::StudioGuiCanvasObjectListItemViewModel {
        kind_label: "Stream",
        target_id: "stream-feed".to_string(),
        label: "Feed".to_string(),
        detail: "feed-1:outlet -> flash-1:inlet".to_string(),
        attention_summary: None,
        viewport_anchor_label: "stream-feed:0".to_string(),
        command_id: "inspector.focus_stream:stream-feed".to_string(),
        related_stream_ids: vec!["stream-feed".to_string()],
        status_badges: Vec::new(),
        is_active: false,
    };

    assert_eq!(
        CanvasObjectListFilter::from_filter_id("attention"),
        Some(CanvasObjectListFilter::Attention)
    );
    assert_eq!(CanvasObjectListFilter::Units.filter_id(), "units");
    assert!(CanvasObjectListFilter::All.matches(&unit));
    assert!(CanvasObjectListFilter::Attention.matches(&unit));
    assert!(!CanvasObjectListFilter::Attention.matches(&stream));
    assert!(CanvasObjectListFilter::Units.matches(&unit));
    assert!(!CanvasObjectListFilter::Units.matches(&stream));
    assert!(CanvasObjectListFilter::Streams.matches(&stream));
    assert!(!CanvasObjectListFilter::Streams.matches(&unit));
}
