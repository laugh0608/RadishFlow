use super::*;
use crate::studio_gui_shell::home_dashboard::{HomeCaseRowAction, home_case_row_action};
use radishflow_studio::test_support::{
    apply_official_binary_hydrocarbon_near_boundary_consumer_scenario,
    build_official_binary_hydrocarbon_provider,
    official_binary_hydrocarbon_near_boundary_consumer_scenarios,
    solve_snapshot_model_from_project_with_provider_and_edit,
};

fn render_top_bar_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let windows = snapshot.app_host_state.windows.clone();
    let ctx = egui::Context::default();
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            let mut hovered_drop_target = false;
            app.render_top_bar(ctx, &windows, &window, &mut hovered_drop_target);
        },
    );
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            let mut hovered_drop_target = false;
            app.render_top_bar(ctx, &windows, &window, &mut hovered_drop_target);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_tools_menu_content_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let windows = snapshot.app_host_state.windows.clone();
    let current_window_id = window.layout_state.scope.window_id;
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 360.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_tools_top_menu_content(ui, &windows, current_window_id, &window);
            });
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_settings_menu_content_texts(app: &mut ReadyAppState) -> Vec<String> {
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 240.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_settings_top_menu_content(ui);
            });
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_alpha_workbench_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 860.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            let mut hovered_drop_target = false;
            app.render_left_sidebar(ctx, &window, &mut hovered_drop_target);
            app.render_right_sidebar(ctx, &window, &mut hovered_drop_target);
            app.render_bottom_status_bar(ctx, &window);
            app.render_bottom_drawer(ctx, &window);
            app.render_center_stage(ctx, &window, &mut hovered_drop_target);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_center_stage_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(860.0, 620.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            let mut hovered_drop_target = false;
            app.render_center_stage(ctx, &window, &mut hovered_drop_target);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_right_sidebar_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 860.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            let mut hovered_drop_target = false;
            app.render_right_sidebar(ctx, &window, &mut hovered_drop_target);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_left_sidebar_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(360.0, 860.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            let mut hovered_drop_target = false;
            app.render_left_sidebar(ctx, &window, &mut hovered_drop_target);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_property_page_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 860.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            app.render_property_page(ctx, &window);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn active_inspector_field_commit_command(app: &ReadyAppState, field_key: &str) -> String {
    app.platform_host
        .snapshot()
        .window_model()
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected active inspector detail")
        .property_fields
        .iter()
        .find(|field| field.key == field_key)
        .unwrap_or_else(|| panic!("expected active inspector field `{field_key}`"))
        .commit_command_id
        .clone()
        .unwrap_or_else(|| {
            panic!("expected active inspector field `{field_key}` to be committable")
        })
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "expected {actual} to equal {expected}"
    );
}

fn commit_displayed_unit_parameter(
    app: &mut ReadyAppState,
    unit_id: &str,
    field_key: &str,
    expected_value: f64,
) {
    app.dispatch_ui_command(format!("inspector.focus_unit:{unit_id}"));
    let before_window = app.platform_host.snapshot().window_model();
    let before_field = before_window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected active unit inspector detail")
        .property_fields
        .iter()
        .find(|field| field.key == field_key)
        .unwrap_or_else(|| panic!("expected unit inspector field `{field_key}`"));
    assert_eq!(
        before_field.status_label, "Draft",
        "displayed fallback/default `{field_key}` should be presented as a committable draft"
    );
    assert!(
        before_field.is_dirty,
        "displayed fallback/default `{field_key}` should require explicit commit"
    );
    assert_close(
        before_field
            .current_value
            .parse::<f64>()
            .unwrap_or_else(|_| panic!("expected numeric field value for `{field_key}`")),
        expected_value,
    );
    let commit_command = before_field
        .commit_command_id
        .clone()
        .unwrap_or_else(|| panic!("expected commit command for `{field_key}`"));

    app.dispatch_inspector_field_draft_commit(commit_command);

    let unit = &app.platform_host.document().flowsheet.units[&UnitId::new(unit_id)];
    let committed = if field_key.ends_with(":outlet_temperature_k") {
        unit.parameters.outlet_temperature_k
    } else if field_key.ends_with(":outlet_pressure_pa") {
        unit.parameters.outlet_pressure_pa
    } else {
        panic!("unsupported unit parameter field `{field_key}`");
    };
    assert_eq!(
        committed,
        Some(expected_value),
        "expected `{field_key}` to be written into UnitOperationParameters"
    );

    let after_window = app.platform_host.snapshot().window_model();
    let after_field = after_window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected active unit inspector detail after commit")
        .property_fields
        .iter()
        .find(|field| field.key == field_key)
        .unwrap_or_else(|| panic!("expected committed unit inspector field `{field_key}`"));
    assert_eq!(after_field.status_label, "Synced");
    assert!(after_field.commit_command_id.is_none());
}

fn render_bottom_drawer_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 360.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            app.render_bottom_drawer(ctx, &window);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_bottom_results_table_direct_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 640.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_bottom_results_table(ui, &window);
            });
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_home_dashboard_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = app.window_model_with_shell_home(&snapshot);
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 860.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            app.render_home_dashboard(ctx, &window);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn collect_shape_texts(shape: &egui::epaint::Shape, texts: &mut Vec<String>) {
    match shape {
        egui::epaint::Shape::Text(text) => texts.push(text.galley.job.text.clone()),
        egui::epaint::Shape::Vec(shapes) => {
            for shape in shapes {
                collect_shape_texts(shape, texts);
            }
        }
        _ => {}
    }
}

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
    assert_eq!(locale.text(ShellText::File), "文件");
    assert_eq!(locale.text(ShellText::Tools), "工具");
    assert_eq!(locale.text(ShellText::Settings), "设置");
    assert_eq!(locale.text(ShellText::ViewOptions), "视图");
    assert_eq!(locale.text(ShellText::Commands), "命令");
    assert_eq!(locale.text(ShellText::Language), "语言");
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
    assert_eq!(locale.text(ShellText::Project), "项目");
    assert_eq!(locale.text(ShellText::Palette), "模块");
    assert_eq!(locale.text(ShellText::ResultsTable), "结果表");
    assert_eq!(locale.text(ShellText::UnitsSi), "单位: SI");
    assert_eq!(
        locale.text(ShellText::SolverSequentialModular),
        "求解器: 顺序模块法"
    );
    assert_eq!(locale.runtime_label("Unit").as_ref(), "单元");
    assert_eq!(locale.runtime_label("Stream").as_ref(), "流股");
    assert_eq!(locale.runtime_label("Idle").as_ref(), "空闲");
    assert_eq!(locale.runtime_label("Status Summary").as_ref(), "状态汇总");
    assert_eq!(locale.runtime_label("Case").as_ref(), "案例");
    assert_eq!(locale.runtime_label("Saved").as_ref(), "已保存");
    assert_eq!(locale.runtime_label("Unselected").as_ref(), "未选择");
    assert_eq!(locale.runtime_label("SnapshotMissing").as_ref(), "缺少快照");
    assert_eq!(
        locale.runtime_label("Result Context").as_ref(),
        "结果工具栏"
    );
    assert_eq!(locale.runtime_label("Focus").as_ref(), "聚焦");
    assert_eq!(locale.runtime_label("Place Feed").as_ref(), "放置进料");
    assert_eq!(locale.runtime_label("Place unit").as_ref(), "放置单元");
    assert_eq!(
        StudioShellLocale::En.runtime_label("Converged").as_ref(),
        "Converged"
    );
}

#[test]
fn native_options_start_with_room_for_alpha_workspace() {
    let options = studio_native_options();

    assert_eq!(options.viewport.inner_size, Some(egui::vec2(1280.0, 860.0)));
    assert_eq!(
        options.viewport.min_inner_size,
        Some(egui::vec2(1024.0, 720.0))
    );
}

#[cfg(target_os = "macos")]
#[test]
fn native_options_use_metal_only_on_macos_to_avoid_opengl_loader_noise() {
    let options = studio_native_options();

    assert_eq!(options.renderer, eframe::Renderer::Wgpu);
    match options.wgpu_options.wgpu_setup {
        eframe::egui_wgpu::WgpuSetup::CreateNew(setup) => {
            assert_eq!(
                setup.instance_descriptor.backends,
                eframe::wgpu::Backends::METAL
            );
        }
        eframe::egui_wgpu::WgpuSetup::Existing(_) => {
            panic!("expected Studio native options to create a Metal-only wgpu instance");
        }
    }
}

#[test]
fn top_bar_aligns_primary_navigation_and_command_buckets() {
    let mut app = ready_app_state(&synced_workspace_config());
    let texts = render_top_bar_texts(&mut app);

    let primary_entries = [
        "文件",
        "主页",
        "物性",
        "流程图",
        "运行",
        "结果",
        "工具",
        "设置",
    ];
    for expected in primary_entries {
        assert_eq!(
            texts
                .iter()
                .filter(|text| text.as_str() == expected)
                .count(),
            1,
            "expected top bar primary navigation to render `{expected}` exactly once, rendered texts: {:?}",
            texts
        );
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected top bar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    assert_eq!(
        primary_entries
            .iter()
            .map(|entry| texts.iter().filter(|text| text.as_str() == *entry).count())
            .sum::<usize>(),
        8,
        "expected top bar to expose exactly eight primary navigation entries, rendered texts: {:?}",
        texts
    );
    for hidden in [
        "快速操作",
        "打开示例",
        "新建空白",
        "打开项目...",
        "保存",
        "另存为...",
        "视图",
        "新建逻辑窗口",
        "English",
        "命令面板 (Ctrl+K)",
        ".rfproj.json",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "expected top bar to hide `{hidden}` behind the view menu, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn flowsheet_context_toolbar_renders_existing_canvas_run_result_state() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.screen = StudioShellScreen::Workbench;
    let texts = render_top_bar_texts(&mut app);

    for expected in [
        "流程图工具栏",
        "画布",
        "放置进料",
        "放置闪蒸罐",
        "运行当前流程",
        "审阅",
        "模块结果",
        "结果表",
        "快照",
        "缺少快照",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected flowsheet context toolbar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for hidden in ["帮助", "完整报表", "自动布线", "自由连线", "完整参数表"] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "flowsheet context toolbar must not expose out-of-scope `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn property_context_toolbar_renders_existing_package_component_commands_and_state() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.create_blank_project();
    app.screen = StudioShellScreen::Property;

    let texts = render_top_bar_texts(&mut app);

    for expected in [
        "物性工具栏",
        "物性包",
        "二元烃 Lite",
        "组分",
        "选择 Methane",
        "选择 Ethane",
        "源: 内置",
        "未选择",
        "可用",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected property context toolbar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for hidden in [
        "第三方物性包",
        "完整组分数据库",
        "Thermodynamics PMC",
        "完整参数表",
        "分析",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "property context toolbar must not expose out-of-scope `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn run_context_toolbar_renders_existing_run_commands_and_state() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.screen = StudioShellScreen::Run;

    let texts = render_top_bar_texts(&mut app);

    for expected in [
        "运行工具栏",
        "控制",
        "运行当前流程",
        "监控",
        "运行日志",
        "收敛",
        "建议",
        "诊断",
        "模式",
        "快照",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected run context toolbar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for hidden in [
        "完整运行控制台",
        "完整日志系统",
        "完整报表",
        "批量运行",
        "自动调度",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "run context toolbar must not expose out-of-scope `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn result_context_toolbar_renders_existing_result_commands_and_state() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.screen = StudioShellScreen::Results;

    let texts = render_top_bar_texts(&mut app);

    for expected in [
        "结果工具栏",
        "审阅",
        "模块结果",
        "结果表",
        "聚焦",
        "Feed",
        "Heated Outlet",
        "Liquid Outlet",
        "Vapor Outlet",
        "快照",
        "流股",
        "单元",
        "诊断",
        "当前",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected result context toolbar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for hidden in [
        "完整报表",
        "跨快照报表",
        "报表模板",
        "打印",
        "批量导出",
        "第三方报表",
        "自动布线",
        "自由连线",
        "完整参数表",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "result context toolbar must not expose out-of-scope `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn tools_menu_content_uses_existing_command_and_window_state() {
    let mut app = ready_app_state(&synced_workspace_config());
    let texts = render_tools_menu_content_texts(&mut app);

    for expected in [
        "命令",
        "命令面板 (Ctrl+K)",
        "显示命令",
        "逻辑窗口",
        "新建逻辑窗口",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected tools menu content to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for hidden in [
        "帮助",
        "插件",
        "扩展",
        "单位集",
        "偏好",
        "主题",
        "发布",
        "完整报表",
        "完整参数表",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "tools menu content must not expose out-of-scope `{hidden}`, rendered texts: {:?}",
            texts
        );
    }

    app.command_palette.open();
    let open_palette_texts = render_tools_menu_content_texts(&mut app);
    assert!(
        open_palette_texts
            .iter()
            .any(|text| text.contains("隐藏命令面板")),
        "expected tools menu content to reflect command palette shell state, rendered texts: {:?}",
        open_palette_texts
    );
}

#[test]
fn settings_menu_content_uses_existing_language_state() {
    let mut app = ready_app_state(&synced_workspace_config());
    let texts = render_settings_menu_content_texts(&mut app);

    for expected in ["语言", "中文", "English"] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected settings menu content to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for hidden in [
        "单位集",
        "偏好",
        "插件",
        "主题",
        "账号",
        "授权",
        "服务器",
        "完整设置",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "settings menu content must not expose out-of-scope `{hidden}`, rendered texts: {:?}",
            texts
        );
    }

    app.locale = StudioShellLocale::En;
    let english_texts = render_settings_menu_content_texts(&mut app);
    assert!(
        english_texts.iter().any(|text| text.contains("Language")),
        "expected settings menu content to reflect current locale, rendered texts: {:?}",
        english_texts
    );
}

#[test]
fn shell_defaults_to_alpha_workbench_layout_regions() {
    let mut app = ready_app_state(&synced_workspace_config());
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Project);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    assert_eq!(app.bottom_drawer_tab, StudioShellBottomDrawerTab::Messages);

    let texts = render_alpha_workbench_texts(&mut app);
    for expected in [
        "项目",
        "模块",
        "示例项目",
        "物性包",
        "检查器",
        "模块设置",
        "模块结果",
        "结果",
        "运行",
        "消息",
        "结果表",
        "还没有求解快照。",
        "状态汇总",
        "案例: 已保存",
        "收敛: 无",
        "单位: SI",
        "求解器: 顺序模块法",
        "流程图模式",
        "物料线",
        "画布",
        "画布状态",
        "项目组分",
        "Methane",
        "Ethane",
        "已选择",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected alpha workbench to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for hidden in [
        "pending reason",
        "diagnostics=0",
        "info diagnostics stay out",
        "green markers",
        "arrows indicate",
        "还没有可显示的求解结果。",
        "Canvas",
        "suggestions",
        "actions enabled",
        "选择画布工具",
        "使用放置单元操作开始画布编辑",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "expected alpha workbench to hide developer canvas detail `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn canvas_stage_keeps_object_tree_in_project_sidebar() {
    let mut app = ready_app_state(&synced_workspace_config());

    let center_texts = render_center_stage_texts(&mut app);
    for expected in ["画布", "画布状态", "选择", "视口", "物料线"] {
        assert!(
            center_texts.iter().any(|text| text.contains(expected)),
            "expected center stage to render `{expected}`, rendered texts: {:?}",
            center_texts
        );
    }
    for hidden in [
        "项目输入",
        "示例入口",
        "对象树",
        "审阅状态",
        "完整项目浏览器",
        "自由连线",
        "自动布线",
    ] {
        assert!(
            !center_texts.iter().any(|text| text.contains(hidden)),
            "center stage must not render project sidebar role `{hidden}`, rendered texts: {:?}",
            center_texts
        );
    }

    let left_texts = render_left_sidebar_texts(&mut app);
    assert!(
        left_texts.iter().any(|text| text == "对象树"),
        "expected project sidebar to keep object tree ownership, rendered texts: {:?}",
        left_texts
    );
}

#[test]
fn left_sidebar_top_tabs_align_with_module_project_roles() {
    let mut app = ready_app_state(&synced_workspace_config());

    let texts = render_left_sidebar_texts(&mut app);

    for expected in ["模块", "项目", "示例项目", "物性包", "项目组分"] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected left sidebar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for retired in ["放置", "运行", "检查器", "模块设置", "模块结果"] {
        assert!(
            !texts.iter().any(|text| text == retired),
            "left sidebar top roles must not expose `{retired}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn project_sidebar_separates_inputs_objects_examples_and_review_roles() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.left_sidebar_tab = StudioShellLeftSidebarTab::Project;

    let texts = render_left_sidebar_texts(&mut app);

    for expected in [
        "项目输入",
        "示例入口",
        "对象树",
        "审阅状态",
        "物性包",
        "项目组分",
        "示例项目",
        "流股",
        "单元",
        "结果",
        "诊断",
    ] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected project sidebar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }

    for hidden in [
        "放置单元",
        "流股源",
        "完整项目浏览器",
        "模块库",
        "自由连线",
        "自动布线",
        "完整报表",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "project sidebar must not expose out-of-scope `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn module_sidebar_groups_supported_palette_by_modeling_roles() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;

    let texts = render_left_sidebar_texts(&mut app);

    for expected in [
        "模块",
        "项目",
        "放置单元",
        "使用当前受控模块集搭建小流程。",
        "流股源",
        "调节单元",
        "汇合与分离",
        "放置进料",
        "放置加热器",
        "放置冷却器",
        "放置阀门",
        "放置混合器",
        "放置闪蒸罐",
    ] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected module sidebar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }

    for hidden in ["完整模块库", "自由连线", "自动布线", "完整拖拽布局"] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "module sidebar must not expose out-of-scope `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn module_sidebar_filter_limits_palette_to_matching_supported_units() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;
    app.module_palette_filter = "flash".to_string();

    let texts = render_left_sidebar_texts(&mut app);

    for expected in ["汇合与分离", "放置闪蒸罐"] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected filtered module sidebar to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }

    for hidden in [
        "流股源",
        "调节单元",
        "放置进料",
        "放置加热器",
        "放置冷却器",
        "放置阀门",
        "放置混合器",
    ] {
        assert!(
            !texts.iter().any(|text| text == hidden),
            "filtered module sidebar should hide `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn top_bar_exposes_home_property_and_flowsheet_navigation() {
    let mut app = ready_app_state(&synced_workspace_config());
    let texts = render_top_bar_texts(&mut app);

    for expected in ["主页", "物性", "流程图", "运行"] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected top bar navigation to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn right_sidebar_main_tabs_align_with_module_workbench_roles() {
    let mut app = ready_app_state(&synced_workspace_config());

    let texts = render_right_sidebar_texts(&mut app);

    for expected in ["检查器", "模块设置", "模块结果"] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected right sidebar tab `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for retired in ["运行", "物性包"] {
        assert!(
            !texts.iter().any(|text| text == retired),
            "expected retired right sidebar tab `{retired}` to be absent, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn right_sidebar_tabs_share_canvas_selection_context_for_active_unit() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_unit:heater-1");

    for tab in [
        StudioShellRightSidebarTab::Inspector,
        StudioShellRightSidebarTab::ModuleSettings,
        StudioShellRightSidebarTab::ModuleResults,
    ] {
        app.right_sidebar_tab = tab;
        let texts = render_right_sidebar_texts(&mut app);

        assert!(
            texts.iter().any(|text| text == "画布选择")
                && texts.iter().any(|text| text == "单元")
                && texts.iter().any(|text| text == "heater-1")
                && texts.iter().any(|text| text.contains("都跟随这个已选单元")),
            "expected right sidebar tab `{tab:?}` to share the active canvas unit selection context, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn module_tabs_keep_unit_only_content_when_canvas_selection_is_stream() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");

    for tab in [
        StudioShellRightSidebarTab::ModuleSettings,
        StudioShellRightSidebarTab::ModuleResults,
    ] {
        app.right_sidebar_tab = tab;
        let texts = render_right_sidebar_texts(&mut app);

        assert!(
            texts.iter().any(|text| text == "画布选择")
                && texts.iter().any(|text| text == "流股")
                && texts.iter().any(|text| text == "stream-feed")
                && texts.iter().any(|text| text == "未选择单元")
                && texts.iter().any(|text| text.contains("等待单元选择")),
            "expected right sidebar tab `{tab:?}` to keep module content unit-only while exposing the stream canvas selection, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn bottom_drawer_tabs_align_with_run_information_roles() {
    let mut app = ready_app_state(&synced_workspace_config());

    let texts = render_bottom_drawer_texts(&mut app);

    for expected in ["消息", "运行日志", "收敛", "建议", "诊断", "结果表"] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected bottom drawer tab `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for right_sidebar_only in ["检查器", "模块设置", "模块结果"] {
        assert!(
            !texts.iter().any(|text| text == right_sidebar_only),
            "bottom drawer must not render right sidebar tab `{right_sidebar_only}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn bottom_workbench_renders_status_summary_as_right_split_role() {
    let mut app = ready_app_state(&synced_workspace_config());

    let texts = render_bottom_drawer_texts(&mut app);

    for expected in [
        "状态汇总",
        "案例",
        "运行",
        "收敛",
        "步骤",
        "诊断",
        "快照",
        "已保存",
        "顺序步骤",
        "无",
    ] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected bottom status summary split to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for hidden in [
        "检查器",
        "模块设置",
        "模块结果",
        "完整报表",
        "跨快照报表",
        "收敛曲线",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "bottom status summary split must not expose `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn bottom_status_summary_split_tracks_current_snapshot_after_run() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");

    let texts = render_bottom_drawer_texts(&mut app);

    for expected in ["状态汇总", "当前", "已收敛", "步骤", "诊断"] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected bottom status summary split to render current run `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    assert!(
        !texts.iter().any(|text| text.contains("迭代")),
        "bottom status summary must not fabricate iteration data, rendered texts: {:?}",
        texts
    );
}

#[test]
fn property_screen_renders_independent_property_page_from_window_model() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.screen = StudioShellScreen::Property;
    let texts = render_property_page_texts(&mut app);

    for expected in [
        "物性",
        "物性工作区",
        "物性包",
        "二元烃 Lite",
        "项目组分",
        "Methane",
        "Ethane",
        "摘要",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected property page to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn project_navigator_uses_row_click_without_repeated_inspect_buttons() {
    let mut app = ready_app_state(&synced_workspace_config());

    let texts = render_alpha_workbench_texts(&mut app);

    assert!(
        !texts.iter().any(|text| text == "检查"),
        "expected project navigator rows to avoid repeated inspect buttons, rendered texts: {:?}",
        texts
    );
}

#[test]
fn shell_starts_on_home_dashboard_with_start_environment_and_messages() {
    let mut app = ready_app_state(&synced_workspace_config());

    assert_eq!(app.screen, StudioShellScreen::Home);

    let texts = render_home_dashboard_texts(&mut app);
    for expected in [
        "RadishFlow Studio",
        "稳态流程模拟",
        "开始",
        "新建项目",
        "创建 Mixer-Flash 小案例",
        "创建 Heater-Flash 小案例",
        "打开项目",
        "打开示例项目",
        "最近项目",
        "示例项目",
        "环境",
        "客户端",
        "服务端",
        "设备",
        "消息",
        "本地就绪",
        "服务端离线",
        "尚未登录",
        "内置示例已在本地可用。",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected home dashboard to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    assert!(
        !texts.iter().any(|text| text.contains("继续上次项目")),
        "expected home dashboard to hide redundant continue action, rendered texts: {:?}",
        texts
    );
    assert!(
        !texts
            .iter()
            .any(|text| text.contains("v26.5.1-dev internal")),
        "expected home dashboard to avoid stale staging version chips, rendered texts: {:?}",
        texts
    );
    assert_eq!(
        texts
            .iter()
            .filter(|text| text.contains("打开示例项目"))
            .count(),
        1,
        "expected home dashboard to render only the left-side example open action, rendered texts: {:?}",
        texts
    );
    for hidden in [
        "Start",
        "New Blank Case",
        "打开 Case",
        "打开示例 Case",
        "最近 Case",
        "示例 Case",
        "PME 样例",
        "Synthetic",
        "Recent Cases",
        "Example Cases",
        "Environment",
        "Messages",
        "Local ready",
        "Server offline",
        "Signed out",
        "You are not signed in",
    ] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "expected home dashboard to hide English `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn home_window_model_maps_recent_projects_to_case_tiles() {
    let mut app = ready_app_state(&synced_workspace_config());
    let snapshot = app.platform_host.snapshot();
    let current_project = PathBuf::from(
        snapshot
            .runtime
            .workspace_document
            .project_path
            .as_ref()
            .expect("expected synced workspace project path"),
    );
    let missing_project = std::env::temp_dir().join(format!(
        "radishflow-home-missing-recent-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    app.project_open.recent_projects = vec![missing_project.clone(), current_project.clone()];

    let window = app.window_model_with_shell_home(&snapshot);

    assert_eq!(window.home.recent_case_tiles.len(), 2);
    let missing_tile = window
        .home
        .recent_case_tiles
        .iter()
        .find(|tile| tile.path_text == missing_project.display().to_string())
        .expect("expected missing recent tile");
    assert_eq!(
        missing_tile.source,
        radishflow_studio::StudioGuiWindowHomeCaseTileSource::Recent
    );
    assert_eq!(
        missing_tile.status,
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::MissingFile
    );
    assert_eq!(missing_tile.status_label, "Missing file");

    let current_tile = window
        .home
        .recent_case_tiles
        .iter()
        .find(|tile| tile.path_text == current_project.display().to_string())
        .expect("expected current recent tile");
    assert_eq!(
        current_tile.status,
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current
    );
    assert_eq!(
        current_tile.title,
        "Feed Heater Flash Binary Hydrocarbon Example"
    );
    assert_eq!(current_tile.package_summary, "Unselected");
    assert_eq!(current_tile.component_summary, "Ethane, Methane");
    assert!(
        current_tile
            .thumbnail
            .nodes
            .iter()
            .any(|node| node == "Heater"),
        "expected current recent tile thumbnail to come from stored flowsheet topology"
    );
}

#[test]
fn home_dashboard_renders_recent_case_tiles_from_window_model() {
    let mut app = ready_app_state(&synced_workspace_config());
    let current_project = PathBuf::from(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .as_ref()
            .expect("expected synced workspace project path"),
    );
    app.project_open.recent_projects = vec![current_project];

    let texts = render_home_dashboard_texts(&mut app);

    for expected in [
        "Feed Heater",
        "当前",
        "feed-heater-flash-binary-hydrocarbon",
        "Feed",
        "Heater",
        "Flash Drum",
        "Ethane, Methane",
        "未选择",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected home dashboard recent tile text `{expected}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn home_mixer_flash_authoring_entry_creates_blank_project_and_opens_palette() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.start_mixer_flash_authoring_case();

    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    assert_eq!(app.bottom_drawer_tab, StudioShellBottomDrawerTab::Messages);
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .expect("expected case authoring notice")
            .title,
        "已开始 Mixer-Flash 小案例"
    );

    let texts = render_alpha_workbench_texts(&mut app);
    for expected in ["Mixer-Flash 小案例", "放置两个 Feed", "放置 Mixer", "待做"] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected authoring checklist to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn home_heater_flash_authoring_entry_creates_blank_project_and_opens_palette() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.start_heater_flash_authoring_case();

    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    assert_eq!(app.bottom_drawer_tab, StudioShellBottomDrawerTab::Messages);
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .expect("expected heater case authoring notice")
            .title,
        "已开始 Heater-Flash 小案例"
    );

    let texts = render_alpha_workbench_texts(&mut app);
    for expected in [
        "Heater-Flash 小案例",
        "放置一个 Feed",
        "放置 Heater",
        "待做",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected heater authoring checklist to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    assert!(
        !texts.iter().any(|text| text.contains("放置 Mixer")),
        "expected selected heater authoring path to hide the mixer checklist, rendered texts: {:?}",
        texts
    );
}

#[test]
fn mixer_flash_authoring_checklist_reflects_feed_outlet_progress() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.start_mixer_flash_authoring_case();
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    let initial_texts = render_alpha_workbench_texts(&mut app);
    assert!(
        !initial_texts.iter().any(|text| text == "完成"),
        "expected blank authoring checklist to start with no completed tasks, rendered texts: {:?}",
        initial_texts
    );

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 140.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-2");

    let texts = render_alpha_workbench_texts(&mut app);
    assert!(
        texts.iter().filter(|text| *text == "完成").count() >= 2,
        "expected checklist to mark feed placement and outlet tasks complete, rendered texts: {:?}",
        texts
    );
    assert!(
        texts.iter().any(|text| text.contains("放置 Mixer")),
        "expected later authoring tasks to remain visible, rendered texts: {:?}",
        texts
    );
}

#[test]
fn heater_flash_authoring_checklist_reflects_heater_outlet_progress() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.start_heater_flash_authoring_case();
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    let initial_texts = render_alpha_workbench_texts(&mut app);
    assert!(
        !initial_texts.iter().any(|text| text == "完成"),
        "expected blank heater authoring checklist to start with no completed tasks, rendered texts: {:?}",
        initial_texts
    );

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");

    app.dispatch_ui_command("canvas.begin_place_unit.heater");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(180.0, 40.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.heater.connect_inlet.heater-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.heater.create_outlet.heater-1");

    let texts = render_alpha_workbench_texts(&mut app);
    assert!(
        texts.iter().filter(|text| *text == "完成").count() >= 5,
        "expected checklist to mark feed and heater path tasks complete, rendered texts: {:?}",
        texts
    );
    assert!(
        texts.iter().any(|text| text.contains("放置 Flash Drum")),
        "expected later heater authoring tasks to remain visible, rendered texts: {:?}",
        texts
    );
}

#[test]
fn blank_palette_does_not_auto_match_authoring_case_after_mixer_topology() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    app.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 140.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-2");
    app.dispatch_ui_command("canvas.begin_place_unit.mixer");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(210.0, 90.0));

    let texts = render_alpha_workbench_texts(&mut app);
    assert!(
        !texts.iter().any(|text| text.contains("Mixer-Flash 小案例")),
        "blank project must not infer mixer authoring checklist, rendered texts: {:?}",
        texts
    );
    assert!(
        !texts
            .iter()
            .any(|text| text.contains("Heater-Flash 小案例")),
        "blank project must not infer heater authoring checklist, rendered texts: {:?}",
        texts
    );
}

#[test]
fn blank_project_empty_run_uses_generic_modeling_readiness() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    app.dispatch_ui_command("run_panel.run_manual");

    let window = app.platform_host.snapshot().window_model();
    assert!(
        window.runtime.latest_failure.is_none(),
        "empty blank project should stop before solver failure"
    );
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    let notice = app
        .project_open
        .notice
        .as_ref()
        .expect("expected modeling readiness notice");
    assert_eq!(notice.title, "模型输入未完成");
    assert!(
        notice.detail.contains("至少一个单元"),
        "expected first-unit readiness detail, got {notice:?}"
    );
}

#[test]
fn blank_project_empty_resume_uses_generic_modeling_readiness() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    app.dispatch_ui_command("run_panel.resume_workspace");

    let window = app.platform_host.snapshot().window_model();
    assert!(
        window.runtime.latest_failure.is_none(),
        "empty blank project resume should stop before solver failure"
    );
    assert_eq!(
        window.runtime.control_state.pending_reason,
        Some(rf_ui::SolvePendingReason::SnapshotMissing)
    );
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    assert_eq!(app.bottom_drawer_tab, StudioShellBottomDrawerTab::Messages);
    let notice = app
        .project_open
        .notice
        .as_ref()
        .expect("expected modeling readiness notice");
    assert_eq!(notice.title, "模型输入未完成");
    assert!(
        notice.detail.contains("至少一个单元"),
        "expected first-unit readiness detail, got {notice:?}"
    );
}

#[test]
fn blank_project_feed_outlet_run_requires_project_components_before_composition() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");

    app.dispatch_ui_command("run_panel.run_manual");

    let window = app.platform_host.snapshot().window_model();
    assert!(
        window.runtime.latest_failure.is_none(),
        "missing project components should stop before solver failure"
    );
    assert_eq!(app.screen, StudioShellScreen::Property);
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Project);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    let notice = app
        .project_open
        .notice
        .as_ref()
        .expect("expected modeling readiness notice");
    assert_eq!(notice.title, "模型输入未完成");
    assert!(
        notice.detail.contains("项目组分"),
        "expected project component readiness detail, got {notice:?}"
    );
}

#[test]
fn blank_project_mixer_topology_run_uses_generic_modeling_readiness() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 140.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-2");
    select_builtin_binary_hydrocarbon_basis(&mut app);
    app.dispatch_ui_command("canvas.begin_place_unit.mixer");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(210.0, 90.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.mixer.connect_inlet_a.mixer-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.mixer.connect_inlet_b.mixer-1.stream-feed-2-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.mixer.create_outlet.mixer-1");
    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(360.0, 90.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-1.stream-mixer-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.liquid");
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.vapor");

    app.dispatch_ui_command("run_panel.run_manual");

    let window = app.platform_host.snapshot().window_model();
    assert!(
        window.runtime.latest_failure.is_none(),
        "generic modeling readiness should stop before solver failure"
    );
    let detail = window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected missing feed composition stream to be focused");
    assert_eq!(detail.target.kind_label, "Stream");
    assert_eq!(detail.target.target_id, "stream-feed-1-outlet");
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("模型输入未完成")
    );
    assert_ne!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("小案例输入未完成")
    );
}

#[test]
fn blank_project_feed_source_run_requires_positive_molar_flow() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    select_builtin_binary_hydrocarbon_basis(&mut app);
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");
    app.dispatch_ui_command("inspector.focus_stream:stream-feed-1-outlet");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(
            "stream:stream-feed-1-outlet:total_molar_flow_mol_s",
        ),
        "0",
    );
    app.dispatch_inspector_field_draft_commit(
        radishflow_studio::inspector_draft_commit_command_id(
            "stream:stream-feed-1-outlet:total_molar_flow_mol_s",
        ),
    );

    app.dispatch_ui_command("run_panel.run_manual");

    let window = app.platform_host.snapshot().window_model();
    assert!(
        window.runtime.latest_failure.is_none(),
        "invalid feed source flow should stop before solver failure"
    );
    let detail = window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected feed source stream to be focused");
    assert_eq!(detail.target.kind_label, "Stream");
    assert_eq!(detail.target.target_id, "stream-feed-1-outlet");
    let notice = app
        .project_open
        .notice
        .as_ref()
        .expect("expected modeling readiness notice");
    assert_eq!(notice.title, "模型输入未完成");
    assert!(
        notice.detail.contains("摩尔流量"),
        "expected feed source molar-flow readiness detail, got {notice:?}"
    );
}

#[test]
fn blank_project_unit_default_parameter_fields_are_directly_committable() {
    let mut feed_app = ready_app_state(&synced_workspace_config());
    feed_app.create_blank_project();
    feed_app.dispatch_ui_command("canvas.begin_place_unit.feed");
    feed_app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut feed_app, "local.feed.create_outlet.feed-1");

    commit_displayed_unit_parameter(
        &mut feed_app,
        "feed-1",
        "unit:feed-1:outlet_temperature_k",
        298.15,
    );
    commit_displayed_unit_parameter(
        &mut feed_app,
        "feed-1",
        "unit:feed-1:outlet_pressure_pa",
        101_325.0,
    );

    for (begin_command, connect_suggestion, outlet_suggestion, unit_id, fields) in [
        (
            "canvas.begin_place_unit.cooler",
            "local.cooler.connect_inlet.cooler-1.stream-feed-1-outlet",
            "local.cooler.create_outlet.cooler-1",
            "cooler-1",
            vec![
                ("unit:cooler-1:outlet_temperature_k", 285.0),
                ("unit:cooler-1:outlet_pressure_pa", 101_325.0),
            ],
        ),
        (
            "canvas.begin_place_unit.valve",
            "local.valve.connect_inlet.valve-1.stream-feed-1-outlet",
            "local.valve.create_outlet.valve-1",
            "valve-1",
            vec![("unit:valve-1:outlet_pressure_pa", 90_000.0)],
        ),
    ] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.create_blank_project();
        app.dispatch_ui_command("canvas.begin_place_unit.feed");
        app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
        accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");
        app.dispatch_ui_command(begin_command);
        app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(180.0, 40.0));
        accept_canvas_suggestion_by_id(&mut app, connect_suggestion);
        accept_canvas_suggestion_by_id(&mut app, outlet_suggestion);

        for (field_key, expected_value) in fields {
            commit_displayed_unit_parameter(&mut app, unit_id, field_key, expected_value);
        }
    }

    let mut mixer_app = ready_app_state(&synced_workspace_config());
    mixer_app.create_blank_project();
    mixer_app.dispatch_ui_command("canvas.begin_place_unit.feed");
    mixer_app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut mixer_app, "local.feed.create_outlet.feed-1");
    mixer_app.dispatch_ui_command("canvas.begin_place_unit.feed");
    mixer_app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 140.0));
    accept_canvas_suggestion_by_id(&mut mixer_app, "local.feed.create_outlet.feed-2");
    mixer_app.dispatch_ui_command("canvas.begin_place_unit.mixer");
    mixer_app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(210.0, 90.0));
    accept_canvas_suggestion_by_id(
        &mut mixer_app,
        "local.mixer.connect_inlet_a.mixer-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(
        &mut mixer_app,
        "local.mixer.connect_inlet_b.mixer-1.stream-feed-2-outlet",
    );
    accept_canvas_suggestion_by_id(&mut mixer_app, "local.mixer.create_outlet.mixer-1");

    commit_displayed_unit_parameter(
        &mut mixer_app,
        "mixer-1",
        "unit:mixer-1:outlet_pressure_pa",
        101_325.0,
    );
}

#[test]
fn blank_project_feed_flash_run_requires_flash_parameters_after_feed_inputs() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    select_builtin_binary_hydrocarbon_basis(&mut app);
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");
    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.liquid");
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.vapor");

    app.dispatch_ui_command("run_panel.run_manual");

    let window = app.platform_host.snapshot().window_model();
    assert!(
        window.runtime.latest_failure.is_none(),
        "missing flash parameters should stop before solver failure"
    );
    let detail = window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected flash unit to be focused");
    assert_eq!(detail.target.kind_label, "Unit");
    assert_eq!(detail.target.target_id, "flash-1");
    let notice = app
        .project_open
        .notice
        .as_ref()
        .expect("expected modeling readiness notice");
    assert_eq!(notice.title, "模型输入未完成");
    assert!(
        notice.detail.contains("出口温度"),
        "expected flash parameter readiness detail, got {notice:?}"
    );

    let flash_temperature_commit =
        active_inspector_field_commit_command(&app, "unit:flash-1:outlet_temperature_k");
    app.dispatch_inspector_field_draft_commit(flash_temperature_commit);

    let notice = app
        .project_open
        .notice
        .as_ref()
        .expect("expected next flash parameter readiness notice");
    assert_eq!(notice.title, "模型输入未完成");
    assert!(
        notice.detail.contains("出口压力"),
        "expected readiness notice to advance to flash pressure, got {notice:?}"
    );

    let flash_pressure_commit =
        active_inspector_field_commit_command(&app, "unit:flash-1:outlet_pressure_pa");
    app.dispatch_inspector_field_draft_commit(flash_pressure_commit);

    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        None,
        "modeling readiness notice should clear after displayed flash defaults are committed"
    );
}

#[test]
fn modeling_readiness_notice_refreshes_after_committing_displayed_unit_defaults() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    select_builtin_binary_hydrocarbon_basis(&mut app);
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");
    app.dispatch_ui_command("canvas.begin_place_unit.heater");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(180.0, 40.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.heater.connect_inlet.heater-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.heater.create_outlet.heater-1");

    app.dispatch_ui_command("run_panel.run_manual");

    let notice = app
        .project_open
        .notice
        .as_ref()
        .expect("expected heater parameter readiness notice");
    assert_eq!(notice.title, "模型输入未完成");
    assert!(
        notice.detail.contains("出口温度"),
        "expected missing heater temperature notice, got {notice:?}"
    );

    let heater_temperature_commit =
        active_inspector_field_commit_command(&app, "unit:heater-1:outlet_temperature_k");
    app.dispatch_inspector_field_draft_commit(heater_temperature_commit);

    let notice = app
        .project_open
        .notice
        .as_ref()
        .expect("expected next heater parameter readiness notice");
    assert_eq!(notice.title, "模型输入未完成");
    assert!(
        notice.detail.contains("出口压力"),
        "expected readiness notice to advance to heater pressure, got {notice:?}"
    );

    let heater_pressure_commit =
        active_inspector_field_commit_command(&app, "unit:heater-1:outlet_pressure_pa");
    app.dispatch_inspector_field_draft_commit(heater_pressure_commit);

    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        None,
        "modeling readiness notice should clear after displayed defaults are committed"
    );
}

#[test]
fn blank_project_feed_port_exposes_stream_inspector_action() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");
    app.right_sidebar_tab = StudioShellRightSidebarTab::ModuleSettings;
    app.dispatch_ui_command("inspector.focus_unit:feed-1");

    let texts = render_alpha_workbench_texts(&mut app);
    assert!(
        texts.iter().any(|text| text == "stream-feed-1-outlet"),
        "expected feed outlet stream id in unit port list, rendered texts: {:?}",
        texts
    );
    assert!(
        texts.iter().any(|text| text == "打开流股"),
        "expected explicit stream inspector action in unit port list, rendered texts: {:?}",
        texts
    );

    app.dispatch_ui_command("inspector.focus_stream:stream-feed-1-outlet");
    let window = app.platform_host.snapshot().window_model();
    let detail = window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected stream inspector detail");
    assert_eq!(detail.target.kind_label, "Stream");
    assert_eq!(detail.target.target_id, "stream-feed-1-outlet");
}

#[test]
fn heater_flash_authoring_checklist_keeps_input_tasks_pending_after_topology_only() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.start_heater_flash_authoring_case();
    select_builtin_binary_hydrocarbon_basis(&mut app);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");

    app.dispatch_ui_command("canvas.begin_place_unit.heater");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(180.0, 40.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.heater.connect_inlet.heater-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.heater.create_outlet.heater-1");

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(320.0, 40.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-1.stream-heater-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.liquid");
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.vapor");

    let texts = render_alpha_workbench_texts(&mut app);
    for expected in [
        "提交 Feed 组成",
        "提交 Feed 温度和压力",
        "提交 Heater 出口温度和压力",
        "提交 Flash Drum 温度和压力",
        "运行案例并检查结果",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected authoring checklist to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    assert!(
        texts.iter().filter(|text| *text == "待做").count() >= 4,
        "expected topology-only heater case to keep input and run tasks pending, rendered texts: {:?}",
        texts
    );
}

#[test]
fn mixer_flash_authoring_run_focuses_missing_feed_composition_before_solving() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.start_mixer_flash_authoring_case();
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 140.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-2");
    select_builtin_binary_hydrocarbon_basis(&mut app);
    app.dispatch_ui_command("canvas.begin_place_unit.mixer");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(210.0, 90.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.mixer.connect_inlet_a.mixer-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.mixer.connect_inlet_b.mixer-1.stream-feed-2-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.mixer.create_outlet.mixer-1");
    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(360.0, 90.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-1.stream-mixer-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.liquid");
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.vapor");

    app.dispatch_ui_command("run_panel.run_manual");

    let window = app.platform_host.snapshot().window_model();
    assert!(
        window.runtime.latest_failure.is_none(),
        "authoring run should stop before solver failure"
    );
    let detail = window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected missing feed composition stream to be focused");
    assert_eq!(detail.target.kind_label, "Stream");
    assert_eq!(detail.target.target_id, "stream-feed-1-outlet");
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    assert_eq!(app.bottom_drawer_tab, StudioShellBottomDrawerTab::Messages);
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("模型输入未完成")
    );
}

#[test]
fn home_case_row_action_opens_on_double_click() {
    assert_eq!(home_case_row_action(false, false), HomeCaseRowAction::None);
    assert_eq!(home_case_row_action(true, false), HomeCaseRowAction::Select);
    assert_eq!(home_case_row_action(true, true), HomeCaseRowAction::Open);
    assert_eq!(home_case_row_action(false, true), HomeCaseRowAction::Open);
}

#[test]
fn home_open_project_uses_selected_recent_project() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let first_project = std::env::temp_dir().join(format!(
        "radishflow-home-selected-recent-first-{timestamp}.rfproj.json"
    ));
    let second_project = std::env::temp_dir().join(format!(
        "radishflow-home-selected-recent-second-{timestamp}.rfproj.json"
    ));
    let project = feed_heater_flash_binary_hydrocarbon_project();
    write_project_file(&first_project, &project).expect("expected first recent project");
    write_project_file(&second_project, &project).expect("expected second recent project");
    let mut app = ready_app_state(&synced_workspace_config());
    app.project_open.recent_projects = vec![first_project.clone(), second_project.clone()];
    app.home_selected_recent_project = Some(second_project.clone());

    app.open_selected_recent_project_or_picker();

    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(
        app.project_open.path_input,
        second_project.display().to_string()
    );
    assert_eq!(
        app.home_selected_recent_project.as_deref(),
        Some(second_project.as_path())
    );

    let _ = fs::remove_file(first_project);
    let _ = fs::remove_file(second_project);
}

#[test]
fn home_open_example_uses_selected_example_project() {
    let mut app = ready_app_state(&synced_workspace_config());
    let window = app.platform_host.snapshot().window_model();
    let target_project = window
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    app.home_selected_example_project = Some(target_project.clone());

    app.open_selected_example_project(&window);

    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(
        app.project_open.path_input,
        target_project.display().to_string()
    );
    assert_eq!(
        app.home_selected_example_project.as_deref(),
        Some(target_project.as_path())
    );
}

#[test]
fn home_dashboard_renders_pending_project_operation_actions() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();

    app.dispatch_ui_command("canvas.accept_focused");
    assert!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );

    app.open_example_project(target_project);
    let open_texts = render_home_dashboard_texts(&mut app);
    for expected in ["未保存更改", "仍然打开", "取消打开"] {
        assert!(
            open_texts.iter().any(|text| text.contains(expected)),
            "expected pending open action `{expected}` on home dashboard, rendered texts: {:?}",
            open_texts
        );
    }

    app.create_blank_project();
    let blank_texts = render_home_dashboard_texts(&mut app);
    for expected in ["未保存更改", "仍然新建", "取消新建"] {
        assert!(
            blank_texts.iter().any(|text| text.contains(expected)),
            "expected pending blank action `{expected}` on home dashboard, rendered texts: {:?}",
            blank_texts
        );
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn opening_case_from_home_switches_to_workbench() {
    let mut app = ready_app_state(&synced_workspace_config());
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();

    app.open_example_project(target_project);

    assert_eq!(app.screen, StudioShellScreen::Workbench);
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
fn bottom_results_table_uses_localized_compact_phase_column() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.bottom_drawer_tab = StudioShellBottomDrawerTab::ResultsTable;

    let texts = render_bottom_results_table_direct_texts(&mut app);

    assert!(
        texts.iter().any(|text| text == "流股"),
        "expected localized stream table header, rendered texts: {:?}",
        texts
    );
    assert!(
        texts.iter().any(|text| text == "相态"),
        "expected localized phase table header, rendered texts: {:?}",
        texts
    );
    assert!(
        texts
            .iter()
            .any(|text| text.contains("液相") || text.contains("气相")),
        "expected compact localized phase summary in result table, rendered texts: {:?}",
        texts
    );
    assert!(
        !texts.iter().any(|text| text.contains("phases:")),
        "expected result table to avoid long raw phase text in cells, rendered texts: {:?}",
        texts
    );
    assert!(
        !texts.iter().any(|text| text.contains("没有相态结果。")),
        "expected result table to avoid long no-phase text in cells, rendered texts: {:?}",
        texts
    );
    assert!(
        texts.iter().any(|text| text == "无"),
        "expected result table to render a compact no-phase value, rendered texts: {:?}",
        texts
    );
    for expected in ["单元", "状态", "步骤", "消费流股", "产出流股"] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected bottom result table to render unit result header `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    for expected in ["feed-1", "stream-feed"] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected bottom result table to include unit/step result `{expected}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn bottom_convergence_tab_consumes_status_summary_and_current_snapshot() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.bottom_drawer_tab = StudioShellBottomDrawerTab::Convergence;

    let texts = render_bottom_drawer_texts(&mut app);

    for expected in ["收敛", "运行", "步骤", "诊断", "当前", "已收敛"] {
        assert!(
            texts.iter().any(|text| text == expected),
            "expected convergence tab to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }
    assert!(
        texts.iter().any(|text| text.contains("已求解")),
        "expected convergence tab to render current solve summary, rendered texts: {:?}",
        texts
    );
    assert!(
        texts.iter().any(|text| text.contains("快照")),
        "expected convergence tab to render current snapshot identity, rendered texts: {:?}",
        texts
    );
}

#[test]
fn bottom_suggestions_tab_consumes_canvas_suggestions() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.create_blank_project();
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    app.bottom_drawer_tab = StudioShellBottomDrawerTab::Suggestions;

    let window = app.platform_host.snapshot().window_model();
    assert!(
        window.canvas.suggestion_count > 0,
        "expected feed placement to produce canvas suggestions before rendering"
    );
    let texts = render_bottom_drawer_texts(&mut app);

    assert!(
        texts.iter().any(|text| text == "建议")
            && texts.iter().any(|text| text == "已聚焦")
            && texts.iter().any(|text| text == "创建流股"),
        "expected suggestions tab to render focused canvas suggestion action, rendered texts: {:?}",
        texts
    );
}

#[test]
fn runtime_result_summary_is_localized_in_workbench() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.right_sidebar_tab = StudioShellRightSidebarTab::ModuleResults;
    app.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;

    let texts = render_alpha_workbench_texts(&mut app);

    assert!(
        texts.iter().any(|text| text.contains("已求解")),
        "expected localized solve summary in workbench, rendered texts: {:?}",
        texts
    );
    assert!(
        !texts.iter().any(|text| text.contains("solved flowsheet")),
        "expected workbench to hide English solve summary, rendered texts: {:?}",
        texts
    );
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
