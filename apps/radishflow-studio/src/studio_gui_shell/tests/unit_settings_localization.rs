use super::unit_accessibility::accessibility_frame;
use super::*;
use egui::accesskit::{Role, TreeUpdate};
use rf_types::units::DisplayUnitSet;
use rf_ui::ProjectPresentationCommand;

fn labels(tree: &TreeUpdate) -> String {
    tree.nodes
        .iter()
        .flat_map(|(_, node)| [node.label(), node.value()].into_iter().flatten())
        .collect::<Vec<_>>()
        .join("\n")
}

fn settings_frame(app: &mut ReadyAppState) -> TreeUpdate {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    fonts::configure_studio_fonts(&ctx);
    let mut tree = None;
    // A modal's first sizing pass omits noninteractive text from the accessible tree.
    for _ in 0..2 {
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 1000.0),
            )),
            ..Default::default()
        });
        assert!(app.render_unit_settings(&ctx));
        tree = ctx.end_pass().platform_output.accesskit_update;
    }
    tree.unwrap()
}

fn assert_english(tree: &TreeUpdate) {
    let labels = labels(tree);
    assert!(
        !labels
            .chars()
            .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)),
        "{labels}"
    );
}

#[test]
fn project_units_and_saved_default_notice_follow_locale_without_changing_choices() {
    let preferences = test_preferences_path("unit-settings-localization");
    let defaults = preferences.with_file_name(rf_store::UNIT_DEFAULTS_FILE_NAME);
    let mut app =
        ReadyAppState::from_config(&synced_workspace_config(), preferences.clone()).unwrap();
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    app.open_unit_settings();
    app.unit_settings.draft.as_mut().unwrap().1 = DisplayUnitSet::si();
    app.save_unit_default(false);
    let document = app.platform_host.document().clone();
    let presentation = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    let choices = app.unit_settings.draft.clone();
    for (locale, title, saved, unapplied) in [
        (
            StudioShellLocale::En,
            "Project display units",
            "Personal defaults saved",
            "These choices are not applied yet",
        ),
        (
            StudioShellLocale::ZhCn,
            "项目显示单位",
            "个人默认已保存",
            "当前选择尚未应用",
        ),
    ] {
        app.locale = locale;
        let tree = settings_frame(&mut app);
        let rendered = labels(&tree);
        assert!(
            rendered.contains(title) && rendered.contains(saved) && rendered.contains(unapplied),
            "{rendered}"
        );
        if locale == StudioShellLocale::En {
            assert_english(&tree);
        }
        assert_eq!(app.unit_settings.draft, choices);
        assert_eq!(app.platform_host.document(), &document);
        assert_eq!(
            app.platform_host
                .snapshot()
                .runtime
                .workspace_document
                .presentation,
            presentation
        );
        assert_eq!(
            rf_store::read_unit_defaults(&defaults).unwrap(),
            Some(DisplayUnitSet::engineering())
        );
    }
    app.locale = StudioShellLocale::En;
    let menu = accessibility_frame(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.render_settings_top_menu_content(ui);
        });
    });
    assert!(labels(&menu).contains("Project display units…"));
    assert!(!labels(&menu).contains("项目显示单位"));
    fs::remove_dir_all(preferences.parent().unwrap()).unwrap();
}

#[test]
fn unavailable_defaults_and_failed_recovery_remain_actionable_in_each_locale() {
    let preferences = test_preferences_path("unit-settings-recovery-localization");
    let defaults = preferences.with_file_name(rf_store::UNIT_DEFAULTS_FILE_NAME);
    fs::create_dir_all(&defaults).unwrap();
    let mut app =
        ReadyAppState::from_config(&synced_workspace_config(), preferences.clone()).unwrap();
    app.locale = StudioShellLocale::En;
    app.unit_settings.pending_new = Some(None);
    let document = app.platform_host.document().clone();
    let unavailable = settings_frame(&mut app);
    assert_english(&unavailable);
    let rendered = labels(&unavailable);
    assert!(rendered.contains("Personal unit defaults unavailable"));
    assert!(rendered.contains("Create this project with SI"));
    assert_eq!(app.platform_host.document(), &document);
    app.unit_settings.pending_new = None;
    app.open_unit_settings();
    let settings = settings_frame(&mut app);
    assert_english(&settings);
    let save = settings
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| {
            node.role() == Role::Button && node.label() == Some("Use as defaults for new projects")
        })
        .unwrap();
    assert!(save.is_disabled());
    app.request_unit_default_recovery();
    app.save_unit_default(true);
    for (locale, title, failure, retry) in [
        (
            StudioShellLocale::En,
            "Recover personal unit defaults",
            "Could not save personal defaults",
            "Back up and replace defaults",
        ),
        (
            StudioShellLocale::ZhCn,
            "恢复个人单位默认",
            "个人默认保存失败",
            "备份并替换默认",
        ),
    ] {
        app.locale = locale;
        let recovery = settings_frame(&mut app);
        let rendered = labels(&recovery);
        assert!(
            rendered.contains(title) && rendered.contains(failure) && rendered.contains(retry),
            "{rendered}"
        );
        if locale == StudioShellLocale::En {
            assert_english(&recovery);
        }
        assert!(app.unit_settings.is_open());
        assert!(app.unit_settings.default_units.is_err());
        assert_eq!(app.platform_host.document(), &document);
        assert!(defaults.is_dir());
    }
    fs::remove_dir_all(preferences.parent().unwrap()).unwrap();
}
