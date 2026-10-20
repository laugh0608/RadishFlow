use super::*;
use rf_types::units::{DisplayUnitSet, MeasurementUnit};
use rf_ui::{NumericEditCommand, NumericEditEvent, ProjectPresentationCommand};

fn parameter_summary(
    app: &ReadyAppState,
) -> radishflow_studio::StudioGuiWindowModuleSettingsParameterSummaryModel {
    app.platform_host
        .snapshot()
        .window_model()
        .module_settings
        .parameter_summary
        .unwrap()
}

#[test]
fn module_summary_counts_unsubmitted_numeric_sessions_without_counting_inherited_values() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    app.dispatch_ui_command("inspector.focus_unit:feed-1");
    let document = app.platform_host.document().clone();
    assert_eq!(parameter_summary(&app).dirty_field_count, 0);
    assert_eq!(parameter_summary(&app).status_label, "Synced");

    let mut field = app
        .platform_host
        .snapshot()
        .window_model()
        .module_settings
        .parameter_fields
        .into_iter()
        .find(|field| field.key == "unit:feed-1:outlet_temperature_k")
        .unwrap()
        .numeric
        .unwrap()
        .unwrap();
    let variable = field.variable.clone();
    assert!(app.dispatch_numeric_command(&mut field, NumericEditCommand::Begin(variable.clone())));
    assert_eq!(parameter_summary(&app).dirty_field_count, 0);
    assert!(app.edit_numeric_field(
        &mut field,
        NumericEditEvent::SelectInputUnit(MeasurementUnit::Kelvin),
    ));
    assert_eq!(parameter_summary(&app).dirty_field_count, 1);
    assert!(app.edit_numeric_field(&mut field, NumericEditEvent::Undo));
    assert_eq!(parameter_summary(&app).dirty_field_count, 0);

    assert!(app.edit_numeric_field(&mut field, NumericEditEvent::ReplaceText("1e-".into())));
    let incomplete = parameter_summary(&app);
    assert_eq!(incomplete.dirty_field_count, 1);
    assert_eq!(incomplete.issue_count, 0);
    assert!(!incomplete.batch_commit_available);
    assert!(!incomplete.detail.contains("ready for batch commit"));
    assert!(app.edit_numeric_field(&mut field, NumericEditEvent::ReplaceText("invalid".into())));
    let invalid = parameter_summary(&app);
    assert_eq!(invalid.dirty_field_count, 1);
    assert!(invalid.issue_count > 0);
    assert!(!invalid.batch_commit_available);

    let generation = field.generation.unwrap();
    assert!(app.dispatch_numeric_command(
        &mut field,
        NumericEditCommand::Cancel {
            variable,
            generation,
        },
    ));
    assert_eq!(parameter_summary(&app).dirty_field_count, 0);
    assert_eq!(app.platform_host.document(), &document);
}

#[test]
fn non_si_unit_fields_describe_physical_bounds_without_relabeling_the_input_unit() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.locale = locale;
        app.apply_presentation_command(ProjectPresentationCommand::Apply(
            DisplayUnitSet::engineering(),
        ))
        .unwrap();
        for unit in ["feed-1", "heater-1", "flash-1"] {
            app.dispatch_ui_command(format!("inspector.focus_unit:{unit}"));
            let window = app.platform_host.snapshot().window_model();
            for module_settings in [false, true] {
                let texts = render_runtime_area_texts(&mut app, |app, ui| {
                    if module_settings {
                        app.render_runtime_module_settings_tab(ui, &window);
                    } else {
                        app.render_runtime_inspector_tab(ui, &window);
                    }
                });
                assert!(texts.iter().any(|text| text == "°C"));
                assert!(texts.iter().any(|text| text == "bar"));
                assert!(texts.iter().any(|text| text.contains("0 K")), "{texts:?}");
                assert!(texts.iter().any(|text| text.contains("0 Pa")), "{texts:?}");
                assert!(texts.iter().all(|text| {
                    !text.starts_with("单位 K")
                        && !text.starts_with("单位 Pa")
                        && !text.starts_with("Unit K;")
                        && !text.starts_with("Unit Pa;")
                }));
            }
        }
    }
}
