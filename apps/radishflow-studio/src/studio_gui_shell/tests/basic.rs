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

#[derive(Debug, Clone)]
struct RenderedText {
    text: String,
    pos: egui::Pos2,
}

fn render_workbench_positioned_texts(app: &mut ReadyAppState) -> Vec<RenderedText> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let windows = snapshot.app_host_state.windows.clone();
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
            app.render_top_bar(ctx, &windows, &window, &mut hovered_drop_target);
            app.render_left_sidebar(ctx, &window, &mut hovered_drop_target);
            app.render_right_sidebar(ctx, &window, &mut hovered_drop_target);
            app.render_bottom_status_bar(ctx, &window);
            app.render_bottom_drawer(ctx, &window);
            app.render_center_stage(ctx, &window, &mut hovered_drop_target);
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_positioned_shape_texts(&clipped_shape.shape, &mut texts);
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

fn render_bottom_status_bar_texts(app: &mut ReadyAppState) -> Vec<String> {
    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 120.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            app.render_bottom_status_bar(ctx, &window);
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

fn collect_positioned_shape_texts(shape: &egui::epaint::Shape, texts: &mut Vec<RenderedText>) {
    match shape {
        egui::epaint::Shape::Text(text) => texts.push(RenderedText {
            text: text.galley.job.text.clone(),
            pos: text.pos,
        }),
        egui::epaint::Shape::Vec(shapes) => {
            for shape in shapes {
                collect_positioned_shape_texts(shape, texts);
            }
        }
        _ => {}
    }
}

fn first_text_position(
    texts: &[RenderedText],
    label: &str,
    predicate: impl Fn(&str) -> bool,
) -> egui::Pos2 {
    first_text_position_where(texts, label, |item| predicate(&item.text))
}

fn first_text_position_where(
    texts: &[RenderedText],
    label: &str,
    predicate: impl Fn(&RenderedText) -> bool,
) -> egui::Pos2 {
    texts
        .iter()
        .find(|item| predicate(item))
        .map(|item| item.pos)
        .unwrap_or_else(|| panic!("expected rendered text `{label}`, rendered texts: {texts:?}"))
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
    assert_eq!(locale.runtime_label("Canvas tools").as_ref(), "画布工具");
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

#[cfg(target_os = "macos")]
#[test]
fn native_options_disable_macos_default_menu_so_command_q_uses_shell_close_flow() {
    let options = studio_native_options();

    assert!(
        options.event_loop_builder.is_some(),
        "macOS must override the default App menu so Command+Q reaches the Studio close confirmation flow"
    );
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
    for hidden in [
        "放置进料",
        "放置闪蒸罐",
        "左移",
        "右移",
        "帮助",
        "完整报表",
        "自动布线",
        "自由连线",
        "完整参数表",
    ] {
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
        "建模",
        "进入流程图建模",
        "源: 内置",
        "未选择",
        "未完成",
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
fn property_main_path_readiness_tracks_property_package_and_components() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.create_blank_project();

    let initial_window = app.platform_host.snapshot().window_model();
    assert!(!initial_window.property_page.flowsheet_modeling_enabled);
    assert_eq!(
        initial_window.property_page.flowsheet_modeling_status_label,
        "Incomplete"
    );
    let initial_modeling_item = initial_window
        .property_context_toolbar
        .sections
        .iter()
        .find(|section| section.title == "Modeling")
        .and_then(|section| section.items.first())
        .expect("expected property modeling toolbar item");
    assert_eq!(
        initial_modeling_item.target,
        radishflow_studio::StudioGuiWindowContextToolbarItemTarget::FlowsheetModeling
    );
    assert!(!initial_modeling_item.enabled);
    let initial_status = initial_window
        .property_context_toolbar
        .status_items
        .iter()
        .find(|status| status.label == "Modeling")
        .expect("expected property modeling status");
    assert_eq!(initial_status.value, "Property");
    assert_eq!(initial_status.status_label, "Incomplete");

    select_builtin_binary_hydrocarbon_basis(&mut app);

    let ready_window = app.platform_host.snapshot().window_model();
    assert!(ready_window.property_page.flowsheet_modeling_enabled);
    assert_eq!(
        ready_window.property_page.flowsheet_modeling_status_label,
        "Ready"
    );
    let ready_modeling_item = ready_window
        .property_context_toolbar
        .sections
        .iter()
        .find(|section| section.title == "Modeling")
        .and_then(|section| section.items.first())
        .expect("expected ready property modeling toolbar item");
    assert!(ready_modeling_item.enabled);
    assert_eq!(ready_modeling_item.label, "Enter Flowsheet Modeling");
    assert_eq!(ready_modeling_item.status_label.as_deref(), Some("Ready"));
    let ready_status = ready_window
        .property_context_toolbar
        .status_items
        .iter()
        .find(|status| status.label == "Modeling")
        .expect("expected ready property modeling status");
    assert_eq!(ready_status.value, "Flowsheet");
    assert_eq!(ready_status.status_label, "Ready");
}

#[test]
fn run_context_toolbar_renders_existing_run_commands_and_state() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
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
        "可用",
        "无",
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
        "Inspect Result Stream",
        "Heated Outlet",
        "Liquid Outlet",
        "Vapor Outlet",
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
        "案例",
        "已保存",
        "收敛",
        "快照",
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
fn workbench_real_viewport_places_major_roles_in_expected_regions() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.screen = StudioShellScreen::Workbench;
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_unit:heater-1");

    let texts = render_workbench_positioned_texts(&mut app);

    let module_tab = first_text_position(&texts, "模块", |text| text == "模块");
    let object_tree = first_text_position(&texts, "对象树", |text| text == "对象树");
    assert!(
        module_tab.x < 300.0 && object_tree.x < 300.0,
        "expected left sidebar texts to stay in left region, module={module_tab:?}, object_tree={object_tree:?}, texts={texts:?}"
    );

    let canvas_tools = first_text_position(&texts, "画布工具", |text| text == "画布工具");
    let canvas_status = first_text_position(&texts, "画布状态", |text| text == "画布状态");
    let canvas_actions = first_text_position(&texts, "画布操作", |text| text == "画布操作");
    for (label, pos) in [
        ("画布工具", canvas_tools),
        ("画布状态", canvas_status),
        ("画布操作", canvas_actions),
    ] {
        assert!(
            (280.0..900.0).contains(&pos.x) && pos.y < 560.0,
            "expected center canvas text `{label}` to stay in center first viewport, pos={pos:?}, texts={texts:?}"
        );
    }

    let right_selection = first_text_position(&texts, "画布选择", |text| text == "画布选择");
    let module_settings = first_text_position(&texts, "模块设置", |text| text == "模块设置");
    assert!(
        right_selection.x > 900.0 && module_settings.x > 900.0 && right_selection.y < 560.0,
        "expected right sidebar texts to stay in right region, selection={right_selection:?}, settings={module_settings:?}, texts={texts:?}"
    );

    let status_summary = first_text_position(&texts, "状态汇总", |text| text == "状态汇总");
    let bottom_results = first_text_position_where(&texts, "底部结果表", |item| {
        item.text == "结果表" && item.pos.y > 560.0
    });
    assert!(
        status_summary.y > 560.0 && bottom_results.y > 560.0,
        "expected bottom workbench texts to stay below center viewport, status={status_summary:?}, results={bottom_results:?}, texts={texts:?}"
    );

    let thin_status_selection = first_text_position(&texts, "单元已选择: heater-1", |text| {
        text == "单元已选择: heater-1"
    });
    assert!(
        thin_status_selection.y > 820.0,
        "expected thin status bar to stay at bottom edge, selection={thin_status_selection:?}, texts={texts:?}"
    );

    for top_toolbar_duplicate in ["放置进料", "放置闪蒸罐", "左移", "右移", "上移", "下移"]
    {
        assert!(
            !texts
                .iter()
                .any(|item| item.text == top_toolbar_duplicate && item.pos.y < 120.0),
            "top flowsheet toolbar must not repeat `{top_toolbar_duplicate}`, rendered texts: {texts:?}"
        );
    }
}

#[test]
fn canvas_stage_keeps_object_tree_in_project_sidebar() {
    let mut app = ready_app_state(&synced_workspace_config());

    let center_texts = render_center_stage_texts(&mut app);
    for expected in ["画布", "画布状态", "适应内容", "物料线"] {
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
        "选择",
        "视口",
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
fn canvas_toolbar_keeps_place_palette_in_left_module_sidebar() {
    let mut app = ready_app_state(&synced_workspace_config());

    let center_texts = render_center_stage_texts(&mut app);
    for expected in ["画布工具", "视图", "适应内容", "画布状态"] {
        assert!(
            center_texts.iter().any(|text| text == expected),
            "expected center canvas toolbar to render `{expected}`, rendered texts: {:?}",
            center_texts
        );
    }
    for palette_label in [
        "放置",
        "放置进料",
        "放置加热器",
        "放置冷却器",
        "放置阀门",
        "放置混合器",
        "放置闪蒸罐",
    ] {
        assert!(
            !center_texts.iter().any(|text| text == palette_label),
            "center canvas toolbar must not duplicate module palette label `{palette_label}`, rendered texts: {:?}",
            center_texts
        );
    }

    app.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;
    let left_texts = render_left_sidebar_texts(&mut app);
    for palette_label in ["放置进料", "放置加热器", "放置闪蒸罐"] {
        assert!(
            left_texts.iter().any(|text| text == palette_label),
            "expected left module sidebar to retain palette label `{palette_label}`, rendered texts: {:?}",
            left_texts
        );
    }
}

#[test]
fn canvas_stage_keeps_selection_detail_in_right_sidebar() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("inspector.focus_unit:heater-1");

    let center_texts = render_center_stage_texts(&mut app);
    assert!(
        center_texts.iter().any(|text| text == "画布操作")
            && center_texts.iter().any(|text| text == "单元")
            && center_texts.iter().any(|text| text == "heater-1")
            && center_texts.iter().any(|text| text == "聚焦"),
        "expected canvas stage to keep selected object actions available, rendered texts: {:?}",
        center_texts
    );
    for hidden in [
        "Edit",
        "Heater (heater) ports 2/2",
        "layout sidecar position",
    ] {
        assert!(
            !center_texts.iter().any(|text| text.contains(hidden)),
            "canvas stage must not render right-sidebar selection detail `{hidden}`, rendered texts: {:?}",
            center_texts
        );
    }

    let right_texts = render_right_sidebar_texts(&mut app);
    assert!(
        right_texts.iter().any(|text| text == "画布选择")
            && right_texts.iter().any(|text| text == "heater-1")
            && right_texts
                .iter()
                .any(|text| text.contains("都跟随这个已选单元")),
        "expected right sidebar to keep selected object context detail, rendered texts: {:?}",
        right_texts
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

    for hidden in [
        "完整模块库",
        "自由连线",
        "自动布线",
        "完整拖拽布局",
        "Start placing",
        "创建定义组成",
        "在分离前调整",
        "用当前支持的单元",
    ] {
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

    for expected in ["状态汇总", "当前", "已收敛", "步骤", "单元", "诊断"] {
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
fn workbench_first_viewport_keeps_selection_and_status_roles_separated() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_unit:heater-1");

    let center_texts = render_center_stage_texts(&mut app);
    for expected in ["画布状态", "画布操作", "heater-1", "聚焦"] {
        assert!(
            center_texts.iter().any(|text| text == expected),
            "expected center canvas role to render `{expected}`, rendered texts: {:?}",
            center_texts
        );
    }
    for hidden in ["画布选择", "状态汇总", "模块设置", "模块结果"] {
        assert!(
            !center_texts.iter().any(|text| text.contains(hidden)),
            "center canvas must not render sidebar or bottom role `{hidden}`, rendered texts: {:?}",
            center_texts
        );
    }

    let right_texts = render_right_sidebar_texts(&mut app);
    for expected in [
        "画布选择",
        "单元",
        "heater-1",
        "检查器",
        "模块设置",
        "模块结果",
    ] {
        assert!(
            right_texts.iter().any(|text| text == expected),
            "expected right sidebar role to render `{expected}`, rendered texts: {:?}",
            right_texts
        );
    }
    assert!(
        right_texts
            .iter()
            .any(|text| text.contains("都跟随这个已选单元")),
        "expected right sidebar to explain unit-pane coordination, rendered texts: {:?}",
        right_texts
    );
    for hidden in ["状态汇总", "结果表"] {
        assert!(
            !right_texts.iter().any(|text| text.contains(hidden)),
            "right sidebar must not render bottom status role `{hidden}`, rendered texts: {:?}",
            right_texts
        );
    }

    let bottom_texts = render_bottom_drawer_texts(&mut app);
    for expected in ["状态汇总", "当前", "已收敛", "步骤", "单元", "诊断"] {
        assert!(
            bottom_texts.iter().any(|text| text == expected),
            "expected bottom workbench role to render `{expected}`, rendered texts: {:?}",
            bottom_texts
        );
    }
    for hidden in ["画布选择", "检查器", "模块设置", "模块结果"] {
        assert!(
            !bottom_texts.iter().any(|text| text.contains(hidden)),
            "bottom workbench must not render right sidebar role `{hidden}`, rendered texts: {:?}",
            bottom_texts
        );
    }

    let status_texts = render_bottom_status_bar_texts(&mut app);
    for expected in [
        "运行",
        "当前",
        "单位: SI",
        "求解器: 顺序模块法",
        "流程图模式",
        "单元已选择: heater-1",
    ] {
        assert!(
            status_texts.iter().any(|text| text == expected),
            "expected thin status bar to render `{expected}`, rendered texts: {:?}",
            status_texts
        );
    }
    for hidden in ["状态汇总", "案例", "收敛", "步骤", "诊断"] {
        assert!(
            !status_texts.iter().any(|text| text == hidden),
            "thin status bar must not duplicate full status summary item `{hidden}`, rendered texts: {:?}",
            status_texts
        );
    }
}

#[test]
fn property_screen_renders_independent_property_page_from_window_model() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.screen = StudioShellScreen::Property;
    let texts = render_property_page_texts(&mut app);

    for expected in [
        "物性",
        "物性包",
        "二元烃 Lite",
        "项目组分",
        "Methane",
        "Ethane",
        "摘要",
        "进入流程图建模",
        "未完成",
    ] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected property page to render `{expected}`, rendered texts: {:?}",
            texts
        );
    }

    for hidden in ["物性工作区", "参数", "分析", "来源"] {
        assert!(
            !texts.iter().any(|text| text.contains(hidden)),
            "property page should not render inactive workspace navigation `{hidden}`, rendered texts: {:?}",
            texts
        );
    }
}

#[test]
fn property_page_main_path_enters_flowsheet_modeling_after_basis_selection() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.create_blank_project();
    app.screen = StudioShellScreen::Property;

    let initial_texts = render_property_page_texts(&mut app);
    assert!(
        initial_texts.iter().any(|text| text.contains("未完成")),
        "expected incomplete property page before package/components, rendered texts: {:?}",
        initial_texts
    );

    select_builtin_binary_hydrocarbon_basis(&mut app);

    let ready_texts = render_property_page_texts(&mut app);
    for expected in ["进入流程图建模", "就绪", "物性包和项目组分已选择"] {
        assert!(
            ready_texts.iter().any(|text| text.contains(expected)),
            "expected ready property page text `{expected}`, rendered texts: {:?}",
            ready_texts
        );
    }

    app.enter_flowsheet_modeling_from_property();

    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    assert_eq!(app.bottom_drawer_tab, StudioShellBottomDrawerTab::Messages);
}

#[test]
fn blank_project_main_path_workbench_first_viewport_uses_modeling_roles() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.create_blank_project();

    app.enter_flowsheet_modeling_from_property();
    assert_eq!(
        app.screen,
        StudioShellScreen::Property,
        "incomplete property basis must keep the user on the property page"
    );

    select_builtin_binary_hydrocarbon_basis(&mut app);
    app.enter_flowsheet_modeling_from_property();

    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    assert_eq!(app.bottom_drawer_tab, StudioShellBottomDrawerTab::Messages);

    let top_texts = render_top_bar_texts(&mut app);
    for expected in ["流程图工具栏", "运行当前流程", "模块结果", "结果表"] {
        assert!(
            top_texts.iter().any(|text| text.contains(expected)),
            "expected Workbench top context to render `{expected}`, rendered texts: {:?}",
            top_texts
        );
    }
    for hidden in ["放置进料", "放置闪蒸罐", "自动布线", "完整报表"] {
        assert!(
            !top_texts.iter().any(|text| text.contains(hidden)),
            "Workbench top context must not duplicate module or out-of-scope entry `{hidden}`, rendered texts: {:?}",
            top_texts
        );
    }

    let left_texts = render_left_sidebar_texts(&mut app);
    for expected in [
        "模块",
        "项目",
        "放置单元",
        "流股源",
        "调节单元",
        "汇合与分离",
        "放置进料",
        "放置闪蒸罐",
    ] {
        assert!(
            left_texts.iter().any(|text| text == expected),
            "expected blank Workbench left rail to render `{expected}`, rendered texts: {:?}",
            left_texts
        );
    }
    for hidden in ["对象树", "审阅状态", "完整模块库", "自由连线", "自动布线"] {
        assert!(
            !left_texts.iter().any(|text| text.contains(hidden)),
            "blank Workbench left rail must keep `{hidden}` out of the active module tab, rendered texts: {:?}",
            left_texts
        );
    }

    let center_texts = render_center_stage_texts(&mut app);
    for expected in [
        "画布工具",
        "画布状态",
        "0 个单元",
        "0 条物料线",
        "0 条建议",
        "画布图例",
        "选择画布工具",
        "使用放置单元操作开始画布编辑",
    ] {
        assert!(
            center_texts.iter().any(|text| text == expected),
            "expected blank Workbench canvas to render `{expected}`, rendered texts: {:?}",
            center_texts
        );
    }
    for hidden in [
        "放置进料",
        "放置闪蒸罐",
        "对象树",
        "画布选择",
        "模块设置",
        "结果表",
    ] {
        assert!(
            !center_texts.iter().any(|text| text.contains(hidden)),
            "blank Workbench canvas must not render side or bottom role `{hidden}`, rendered texts: {:?}",
            center_texts
        );
    }

    let right_texts = render_right_sidebar_texts(&mut app);
    for expected in [
        "画布选择",
        "无",
        "检查器",
        "属性",
        "请从左侧项目树或画布选择流股/单元。",
    ] {
        assert!(
            right_texts.iter().any(|text| text == expected),
            "expected blank Workbench right rail to render `{expected}`, rendered texts: {:?}",
            right_texts
        );
    }
    for hidden in ["状态汇总", "结果表", "物性包"] {
        assert!(
            !right_texts.iter().any(|text| text.contains(hidden)),
            "blank Workbench right rail must not render `{hidden}`, rendered texts: {:?}",
            right_texts
        );
    }

    let bottom_texts = render_bottom_drawer_texts(&mut app);
    for expected in ["消息", "状态汇总", "还没有求解快照。", "快照", "无"] {
        assert!(
            bottom_texts.iter().any(|text| text == expected),
            "expected blank Workbench bottom area to render `{expected}`, rendered texts: {:?}",
            bottom_texts
        );
    }
    for hidden in ["画布选择", "检查器", "模块设置", "模块结果", "完整报表"] {
        assert!(
            !bottom_texts.iter().any(|text| text.contains(hidden)),
            "blank Workbench bottom area must not render `{hidden}`, rendered texts: {:?}",
            bottom_texts
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
fn home_dashboard_hides_return_workspace_action_before_user_opens_or_creates_case() {
    let mut app = ready_app_state(&synced_workspace_config());

    let texts = render_home_dashboard_texts(&mut app);

    assert!(
        !texts.iter().any(|text| text.contains("返回工作区")),
        "fresh Home should not expose a return action before the user opens or creates a case, rendered texts: {:?}",
        texts
    );
    assert!(
        !texts
            .iter()
            .any(|text| text.contains("继续当前已打开项目。")),
        "fresh Home should not render return-workspace helper text, rendered texts: {:?}",
        texts
    );
}

#[test]
fn home_dashboard_exposes_unsaved_current_project_return_tile() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    app.screen = StudioShellScreen::Home;

    let snapshot = app.platform_host.snapshot();
    let window = app.window_model_with_shell_home(&snapshot);
    assert!(
        app.project_open.recent_projects.is_empty(),
        "unsaved blank projects must not be persisted as recent paths"
    );
    let current_tile = window
        .home
        .recent_case_tiles
        .first()
        .expect("expected current workspace tile");
    assert_eq!(
        current_tile.source,
        radishflow_studio::StudioGuiWindowHomeCaseTileSource::Current
    );
    assert_eq!(
        current_tile.status,
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current
    );
    assert_eq!(current_tile.title, "Blank Project");
    assert_eq!(current_tile.path_text, "Current workspace");
    assert_eq!(current_tile.package_summary, "Unselected");
    assert_eq!(current_tile.component_summary, "No components");

    let texts = render_home_dashboard_texts(&mut app);
    for expected in ["Blank Project", "当前", "当前工作区", "未选择"] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected home dashboard current workspace text `{expected}`, rendered texts: {:?}",
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
fn home_open_project_returns_to_current_workspace_when_current_tile_is_selected() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    app.screen = StudioShellScreen::Home;
    let texts = render_home_dashboard_texts(&mut app);

    assert!(app.home_selected_current_workspace);
    assert_eq!(app.home_selected_recent_project, None);
    for expected in ["返回工作区", "继续当前已打开项目。"] {
        assert!(
            texts.iter().any(|text| text.contains(expected)),
            "expected home dashboard to render current workspace action `{expected}`, rendered texts: {:?}",
            texts
        );
    }

    app.open_selected_recent_project_or_picker();

    assert_eq!(app.screen, StudioShellScreen::Property);
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .title,
        "Blank Project"
    );
    assert!(
        app.project_open.recent_projects.is_empty(),
        "returning to the current unsaved workspace must not persist a fake recent path"
    );
}

#[test]
fn home_current_workspace_tile_returns_to_flowsheet_after_property_main_path_ready() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.create_blank_project();
    select_builtin_binary_hydrocarbon_basis(&mut app);
    app.screen = StudioShellScreen::Home;
    let texts = render_home_dashboard_texts(&mut app);

    assert!(app.home_selected_current_workspace);
    assert_eq!(app.home_selected_recent_project, None);
    assert!(
        texts.iter().any(|text| text.contains("返回工作区")),
        "expected home dashboard to expose an explicit return action, rendered texts: {:?}",
        texts
    );

    app.open_selected_recent_project_or_picker();

    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(app.left_sidebar_tab, StudioShellLeftSidebarTab::Palette);
    assert!(
        app.project_open.recent_projects.is_empty(),
        "returning to a ready unsaved workspace must still avoid fake recent paths"
    );
}

#[test]
fn home_open_project_returns_to_current_workspace_when_selected_recent_is_current() {
    let mut app = ready_app_state(&synced_workspace_config());
    let current_project = PathBuf::from(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .project_path
            .as_ref()
            .expect("expected current project path"),
    );
    app.project_open.recent_projects = vec![current_project.clone()];
    app.home_selected_recent_project = Some(current_project.clone());
    app.screen = StudioShellScreen::Home;

    app.open_selected_recent_project_or_picker();

    assert_eq!(app.screen, StudioShellScreen::Property);
    assert_eq!(
        app.home_selected_recent_project.as_deref(),
        Some(current_project.as_path())
    );
    assert_eq!(app.project_open.pending_confirmation, None);
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
fn bottom_results_table_routes_stream_and_unit_focus_to_matching_right_tab() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    let snapshot = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .expect("expected solve snapshot");
    let snapshot_id = snapshot.snapshot_id.clone();

    app.right_sidebar_tab = StudioShellRightSidebarTab::ModuleResults;
    app.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
    app.focus_result_table_stream(&snapshot_id, "stream-heated");

    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    assert_eq!(
        app.bottom_drawer_tab,
        StudioShellBottomDrawerTab::ResultsTable
    );
    assert_eq!(
        app.result_inspector
            .selected_stream_id_for_snapshot(&snapshot)
            .as_deref(),
        Some("stream-heated")
    );
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Stream", "stream-heated"))
    );

    app.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
    app.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
    app.focus_result_table_unit(&snapshot_id, "heater-1");

    assert_eq!(
        app.right_sidebar_tab,
        StudioShellRightSidebarTab::ModuleResults
    );
    assert_eq!(
        app.bottom_drawer_tab,
        StudioShellBottomDrawerTab::ResultsTable
    );
    assert_eq!(
        app.result_inspector
            .selected_unit_id_for_snapshot(&snapshot)
            .as_deref(),
        Some("heater-1")
    );
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Unit", "heater-1"))
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
