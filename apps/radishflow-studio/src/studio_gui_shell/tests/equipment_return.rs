use super::equipment_editor::change;
use super::equipment_export::EquipmentPicker;
use super::*;
use egui::accesskit::{Action, ActionRequest, NodeId, TreeUpdate};
use equipment::CalculationTableFormat as Format;
use rf_model::equipment::SeparatorRatingInput;
use rf_ui::engineering::{
    EquipmentCaseKey, EquipmentResultStatus,
    editor::{EquipmentDraft, EquipmentEditorCommand, EquipmentEditorTarget},
};

fn context(width: f32) -> egui::Context {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    fonts::configure_studio_fonts(&ctx);
    if width == 400.0 {
        ctx.style_mut(|style| {
            for kind in [egui::TextStyle::Body, egui::TextStyle::Button] {
                style.text_styles.get_mut(&kind).unwrap().size = 20.0;
            }
        });
    }
    ctx
}

fn frame(
    ctx: &egui::Context,
    app: &mut ReadyAppState,
    width: f32,
    focused: bool,
    events: Vec<egui::Event>,
) -> TreeUpdate {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 940.0),
            )),
            focused,
            events,
            ..Default::default()
        },
        |ctx| {
            app.dispatch_shortcuts(ctx);
            app.render_equipment_page(ctx);
            app.render_equipment_export_dialog(ctx);
        },
    )
    .platform_output
    .accesskit_update
    .unwrap()
}

fn action(tree: &TreeUpdate, label: &str) -> NodeId {
    tree.nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("Missing {label:?}"))
        .0
}

fn keys(key: egui::Key) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        })
        .collect()
}

fn activate(ctx: &egui::Context, app: &mut ReadyAppState, width: f32, target: NodeId) {
    let tree = frame(
        ctx,
        app,
        width,
        true,
        vec![egui::Event::AccessKitActionRequest(ActionRequest {
            target,
            action: Action::Focus,
            data: None,
        })],
    );
    assert_eq!(tree.focus, target);
    frame(ctx, app, width, true, keys(egui::Key::Enter));
}

#[test]
fn rating_basis_source_returns_to_the_same_named_cases_original_field_action() {
    for (locale, width, open, back) in [
        (
            StudioShellLocale::ZhCn,
            1280.0,
            "查看基础来源",
            "返回输入字段",
        ),
        (
            StudioShellLocale::En,
            400.0,
            "View basis source",
            "Return to input field",
        ),
    ] {
        let (mut app, project, _) = super::equipment_adoption::fixture("r2-basis-source");
        app.locale = locale;
        let member = EquipmentCaseKey {
            task_id: "matrix-v2".into(),
            case_id: "matrix-v2-case2".into(),
        };
        let target = EquipmentEditorTarget::RatingCase(member.clone());
        app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
        change(&mut app, &target, |draft| {
            let EquipmentDraft::RatingCase(draft) = draft else {
                panic!("expected rating case")
            };
            draft
                .inputs
                .get_mut(&SeparatorRatingInput::OperatingTemperature)
                .unwrap()
                .value
                .set_text("1e-".into());
        });
        let before = app.platform_host.modeling_inputs().workspace.clone();
        let members = &before
            .engineering
            .inputs()
            .rating_task(&member.task_id)
            .unwrap()
            .cases;
        assert_eq!(members[0].name, members[1].name);
        assert_ne!(members[0].id, members[1].id);

        let ctx = context(width);
        frame(&ctx, &mut app, width, true, vec![]);
        let tree = frame(&ctx, &mut app, width, true, vec![]);
        let origin = action(&tree, open);
        activate(&ctx, &mut app, width, origin);
        let tree = frame(&ctx, &mut app, width, true, vec![]);
        assert_eq!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .equipment_editor
                .active(),
            Some(&EquipmentEditorTarget::Source("source-test".into())),
        );
        let return_button = action(&tree, back);
        assert_eq!(tree.focus, return_button);
        activate(&ctx, &mut app, width, return_button);
        let tree = frame(&ctx, &mut app, width, true, vec![]);
        assert_eq!(tree.focus, origin);
        let workspace = &app.platform_host.modeling_inputs().workspace;
        assert_eq!(workspace.equipment_editor.active(), Some(&target));
        assert_eq!(
            workspace.equipment_editor.draft(&target),
            before.equipment_editor.draft(&target)
        );
        assert_eq!(workspace.engineering, before.engineering);
        assert_eq!(
            app.equipment.rating_selection.get(&member.task_id),
            Some(&member.case_id)
        );
        assert_eq!(app.equipment.section, equipment::EquipmentSection::Inputs);
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
}

#[test]
fn failed_rating_source_returns_to_the_launching_evidence_tab_and_issue() {
    let (mut app, project) =
        super::equipment_rating_export::fixture("r2-rating-source-issue", |data| {
            data["engineering"]["sources"][1]["review"] = serde_json::json!("unreviewed");
        });
    app.locale = StudioShellLocale::En;
    let member = super::equipment_rating_export::member();
    let target = EquipmentEditorTarget::RatingCase(member.clone());
    app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
    app.calculate_active_equipment();
    let status = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .case_result_status(&member);
    assert_eq!(status, EquipmentResultStatus::Failed);
    app.equipment.section = equipment::EquipmentSection::Evidence;

    let ctx = context(1280.0);
    frame(&ctx, &mut app, 1280.0, true, vec![]);
    let tree = frame(&ctx, &mut app, 1280.0, true, vec![]);
    let origin = action(
        &tree,
        "Input source：Explicitly adopt or review this source",
    );
    activate(&ctx, &mut app, 1280.0, origin);
    let tree = frame(&ctx, &mut app, 1280.0, true, vec![]);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .active(),
        Some(&EquipmentEditorTarget::Source("drawing".into())),
    );
    let return_button = action(&tree, "Return to evidence");
    assert_eq!(tree.focus, return_button);
    activate(&ctx, &mut app, 1280.0, return_button);
    let tree = frame(&ctx, &mut app, 1280.0, true, vec![]);
    assert_eq!(tree.focus, origin);
    assert_eq!(app.equipment.section, equipment::EquipmentSection::Evidence);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .active(),
        Some(&target)
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

#[test]
fn rating_export_cancel_returns_to_the_same_format_after_window_focus() {
    for (locale, width, format, label, overwrite) in [
        (
            StudioShellLocale::ZhCn,
            1280.0,
            Format::Csv,
            "当前工况 CSV",
            false,
        ),
        (
            StudioShellLocale::En,
            400.0,
            Format::Json,
            "Current case JSON",
            true,
        ),
    ] {
        let (mut app, project, member) =
            super::equipment_adoption::fixture("r2-rating-export-return");
        app.locale = locale;
        let target = EquipmentEditorTarget::RatingCase(member.clone());
        app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
        app.calculate_active_equipment();
        let before = app.platform_host.modeling_inputs().workspace.clone();
        assert_eq!(
            before.engineering.case_result_status(&member),
            EquipmentResultStatus::Current
        );
        let output = project.with_file_name(format!("preserved.{}", format.extension()));
        let content = b"an existing calculation table";
        fs::write(&output, content).unwrap();
        app.project_file_picker = Box::new(EquipmentPicker {
            path: overwrite.then(|| output.clone()),
            supported: true,
        });

        let ctx = context(width);
        frame(&ctx, &mut app, width, true, vec![]);
        let tree = frame(&ctx, &mut app, width, true, vec![]);
        activate(&ctx, &mut app, width, action(&tree, label));
        if overwrite {
            assert!(app.equipment.pending_export.is_some());
            frame(&ctx, &mut app, width, true, vec![]);
            frame(&ctx, &mut app, width, true, keys(egui::Key::Escape));
        }
        assert!(app.equipment.pending_export.is_none());
        ctx.memory_mut(|memory| {
            if let Some(id) = memory.focused() {
                memory.surrender_focus(id);
            }
        });
        let tree = frame(&ctx, &mut app, width, false, vec![]);
        assert_ne!(tree.focus, action(&tree, label));
        let tree = frame(&ctx, &mut app, width, true, vec![]);
        let origin = action(&tree, label);
        assert_eq!(tree.focus, origin);
        let workspace = &app.platform_host.modeling_inputs().workspace;
        assert_eq!(workspace.equipment_editor.active(), Some(&target));
        assert_eq!(workspace.engineering, before.engineering);
        assert_eq!(workspace.project_presentation, before.project_presentation);
        assert_eq!(fs::read(&output).unwrap(), content);
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
}

#[test]
fn canceled_rating_export_does_not_focus_another_same_named_case() {
    let (mut app, project, _) = super::equipment_adoption::fixture("r2-export-other-case");
    app.locale = StudioShellLocale::En;
    let first = EquipmentCaseKey {
        task_id: "matrix-v2".into(),
        case_id: "matrix-v2-case1".into(),
    };
    let second = EquipmentCaseKey {
        task_id: first.task_id.clone(),
        case_id: "matrix-v2-case2".into(),
    };
    for member in [&first, &second] {
        app.equipment_command(EquipmentEditorCommand::Select(
            EquipmentEditorTarget::RatingCase(member.clone()),
        ));
        app.calculate_active_equipment();
        assert_eq!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .engineering
                .case_result_status(member),
            EquipmentResultStatus::Current
        );
    }
    app.project_file_picker = Box::new(EquipmentPicker {
        path: None,
        supported: true,
    });

    let ctx = context(1280.0);
    frame(&ctx, &mut app, 1280.0, true, vec![]);
    let tree = frame(&ctx, &mut app, 1280.0, true, vec![]);
    activate(&ctx, &mut app, 1280.0, action(&tree, "Current case CSV"));
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingCase(first.clone()),
    ));
    let tree = frame(&ctx, &mut app, 1280.0, true, vec![]);
    assert_ne!(tree.focus, action(&tree, "Current case CSV"));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .active(),
        Some(&EquipmentEditorTarget::RatingCase(first)),
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

#[test]
fn matrix_overview_shows_expired_overwrite_notice_without_opening_a_case() {
    for (locale, matrix_button, current_case_button, expired) in [
        (
            StudioShellLocale::ZhCn,
            "矩阵 JSON",
            "当前工况 JSON",
            "导出覆盖确认已失效",
        ),
        (
            StudioShellLocale::En,
            "Matrix JSON",
            "Current case JSON",
            "Export overwrite confirmation expired",
        ),
    ] {
        let (mut app, project, _) = super::equipment_adoption::fixture("matrix-overwrite-notice");
        app.locale = locale;
        app.equipment_command(EquipmentEditorCommand::Select(
            EquipmentEditorTarget::RatingTask("matrix-v2".into()),
        ));
        let output = project.with_file_name("review.json");
        fs::write(&output, b"original").unwrap();
        app.project_file_picker = Box::new(EquipmentPicker {
            path: Some(output.clone()),
            supported: true,
        });

        let width = 400.0;
        let ctx = context(width);
        let tree = frame(&ctx, &mut app, width, true, vec![]);
        let button = action(&tree, matrix_button);
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, n)| n.label() == Some(current_case_button))
        );
        activate(&ctx, &mut app, width, button);
        assert!(app.equipment.pending_export.is_some());
        fs::write(&output, b"external").unwrap();
        app.confirm_equipment_export();
        assert_eq!(fs::read(&output).unwrap(), b"external");

        let tree = frame(&ctx, &mut app, width, true, vec![]);
        let texts: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == egui::accesskit::Role::Label)
            .filter_map(|(_, node)| node.value())
            .collect();
        assert_eq!(
            texts.iter().filter(|t| t.contains(expired)).count(),
            1,
            "{texts:?}"
        );
        assert!(texts.iter().any(|t| t.contains(output.to_str().unwrap())));
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
}
