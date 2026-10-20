use super::*;
use rf_model::{engineering::*, equipment::*};
use rf_ui::engineering::{EquipmentResultStatus, editor::*};

pub(super) fn change(
    app: &mut ReadyAppState,
    target: &EquipmentEditorTarget,
    edit: impl FnOnce(&mut EquipmentDraft),
) {
    let record = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .draft(target)
        .unwrap();
    let generation = record.generation();
    let mut value = record.value().clone();
    edit(&mut value);
    app.equipment_command(EquipmentEditorCommand::Change {
        target: target.clone(),
        generation,
        value: Box::new(value),
    });
}
fn active(app: &ReadyAppState) -> EquipmentEditorTarget {
    app.platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .active()
        .unwrap()
        .clone()
}
fn pending(app: &ReadyAppState) -> usize {
    app.platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .pending_count()
}
pub(super) fn complete_case() -> (ReadyAppState, EquipmentEditorTarget, EquipmentEditorTarget) {
    let mut app = ready_app_state(&synced_workspace_config());
    app.start_independent_equipment_project();
    assert_eq!(app.screen, StudioShellScreen::Equipment);
    assert!(app.platform_host.document().flowsheet.units.is_empty());
    let task = active(&app);
    app.equipment_command(EquipmentEditorCommand::CreateSource);
    let source = active(&app);
    let EquipmentEditorTarget::Source(source_id) = &source else {
        panic!()
    };
    change(&mut app, &source, |draft| {
        let EquipmentDraft::Source(s) = draft else {
            panic!()
        };
        s.title = "Public synthetic arithmetic fixture".into();
        s.kind = SourceKind::Assumed;
        s.review = SourceReview::Adopted;
        s.adoption_note = "Explicit synthetic assumption; no physical validity claim".into();
        s.fluid_system = "Synthetic two-phase fluid".into();
        s.phase = SourcePhase::Both;
        s.temperature.set_text("300".into());
        s.pressure.set_text("100000".into());
        s.scope = "Full cylinder, declared internals and working volume criteria".into();
    });
    app.equipment_command(EquipmentEditorCommand::Apply(source.clone()));
    assert_eq!(pending(&app), 0);
    app.equipment_command(EquipmentEditorCommand::Select(task.clone()));
    change(&mut app, &task, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.tag = "V-101".into();
        t.name = "Synthetic separator".into();
        t.case_name = "Single case".into();
        t.purpose = "Arithmetic verification".into();
        t.fluid_system = "Synthetic two-phase fluid".into();
        t.applicability_sources = vec![source_id.clone()];
        t.applicability = SeparatorApplicability {
            orientation: Some(SeparatorOrientation::Vertical),
            gas_direction: Some(SeparatorGasDirection::Upward),
            service: Some(SeparatorService::CleanNonFoamingTwoPhase),
            geometry: Some(SeparatorSectionGeometry::FullEqualDiameterCylinder),
            k_basis: Some(SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals),
            flow_basis: Some(SeparatorFlowBasis::ActualAtOperatingConditions),
            internals_description: "No effective area reduction; synthetic assumption".into(),
        };
        for (field, value) in SeparatorInput::ALL.into_iter().zip([
            "300", "100000", "0.8", "0.005", "10", "1010", "0.1", "0.8", "120", "1",
        ]) {
            let b = t.fields.get_mut(&EquipmentField::Input(field)).unwrap();
            b.mode = BindingMode::Direct;
            b.value.set_text(value.into());
            b.source = Some(source_id.clone());
        }
    });
    app.equipment_command(EquipmentEditorCommand::Apply(task.clone()));
    assert_eq!(pending(&app), 0);
    (app, task, source)
}
#[test]
fn independent_equipment_checks_calculates_invalidates_and_reopens_without_property_basis() {
    let (mut app, task, source) = complete_case();
    let EquipmentEditorTarget::Task(id) = &task else {
        panic!()
    };
    app.equipment_command(EquipmentEditorCommand::Check(id.clone()));
    let workspace = &app.platform_host.modeling_inputs().workspace;
    assert_eq!(
        workspace
            .equipment_editor
            .checked_issues(id, &workspace.engineering),
        Some([].as_slice())
    );
    assert_eq!(
        workspace.engineering.result_status(id),
        EquipmentResultStatus::Missing
    );
    run_with_key_press(egui::Key::F5, egui::Modifiers::NONE, |ctx| {
        app.dispatch_shortcuts(ctx)
    });
    let session = &app.platform_host.modeling_inputs().workspace.engineering;
    assert_eq!(session.result_status(id), EquipmentResultStatus::Current);
    let candidate = session.current_candidate(id).unwrap();
    assert!(
        (candidate.dimensions().minimum_diameter_m - (4.0 / std::f64::consts::PI).sqrt()).abs()
            < 1e-12
    );
    assert_eq!(candidate.dimensions().tangent_to_tangent_height_m, None);
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .unit_count
            == 0
    );
    app.equipment_command(EquipmentEditorCommand::Select(source.clone()));
    change(&mut app, &source, |draft| {
        let EquipmentDraft::Source(s) = draft else {
            panic!()
        };
        s.adoption_note.push_str("; reviewed assumption");
    });
    app.equipment_command(EquipmentEditorCommand::Apply(source.clone()));
    let confirmation = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .confirmation()
        .unwrap()
        .clone();
    assert_eq!(confirmation.impacts().len(), 1);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(id),
        EquipmentResultStatus::Current
    );
    app.equipment_command(EquipmentEditorCommand::Confirm(confirmation.key()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(id),
        EquipmentResultStatus::Stale
    );
    let path = test_preferences_path("equipment-ui-roundtrip").with_extension("rfproj.json");
    app.request_save_project_as(path.clone(), false);
    if app.pending_format_upgrade.is_some() {
        app.confirm_format_upgrade();
    }
    assert!(path.is_file(), "{:?}", app.project_open.notice);
    app.open_project(path.clone(), "equipment UI test");
    assert_eq!(app.screen, StudioShellScreen::Equipment);
    let workspace = &app.platform_host.modeling_inputs().workspace;
    assert_eq!(
        workspace.engineering.result_status(id),
        EquipmentResultStatus::Missing
    );
    assert_eq!(workspace.engineering.inputs().tasks[0].tag, "V-101");
    assert!(workspace.document.flowsheet.units.is_empty());
    // A predictable external artifact is useful for the separately executed native smoke.
    if let Some(path) = std::env::var_os("RADISHFLOW_EQUIPMENT_NATIVE_FIXTURE") {
        fs::copy(path_from_saved(&app), PathBuf::from(path)).unwrap();
    }
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
fn path_from_saved(app: &ReadyAppState) -> PathBuf {
    PathBuf::from(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .unwrap(),
    )
}
#[test]
fn equipment_incomplete_draft_save_and_project_departure_never_apply_or_drop_input() {
    let (mut app, task, _) = complete_case();
    change(&mut app, &task, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.fields
            .get_mut(&EquipmentField::Input(SeparatorInput::OperatingTemperature))
            .unwrap()
            .value
            .set_text("1e-".into());
    });
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .inputs()
        .clone();
    app.equipment_command(EquipmentEditorCommand::Apply(task.clone()));
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .error()
            .is_some()
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs(),
        &before
    );
    let path = test_preferences_path("equipment-pending-save").with_extension("rfproj.json");
    app.request_save_project_as(path.clone(), false);
    if app.pending_format_upgrade.is_some() {
        app.confirm_format_upgrade();
    }
    assert!(path.is_file());
    assert_eq!(pending(&app), 1);
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .needs_close_confirmation()
    );
    assert!(!app.close_current_window_for_viewport_request());
    assert!(app.project_open.pending_close_window_confirmation.is_some());
    let departure = app.project_departure_checkpoint();
    change(&mut app, &task, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.name.push_str(" pending");
    });
    assert_ne!(app.project_departure_checkpoint(), departure);
    assert!(!app.validate_project_departure());
    assert_eq!(pending(&app), 1);
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
#[test]
fn equipment_history_shortcuts_and_command_routes_do_not_touch_flowsheet_history() {
    let (mut app, task, _) = complete_case();
    let flow = app.platform_host.document().clone();
    let commands = app.platform_host.snapshot().window_model().commands;
    let palette = app.command_palette_items(&commands, "");
    assert!(
        palette
            .iter()
            .any(|item| item.command_id == radishflow_studio::EDIT_UNDO_COMMAND_ID && item.enabled)
    );

    change(&mut app, &task, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.name = "Renamed".into();
    });
    app.equipment_command(EquipmentEditorCommand::Apply(task.clone()));
    run_with_key_press(egui::Key::Z, egui::Modifiers::COMMAND, |ctx| {
        app.dispatch_shortcuts(ctx)
    });
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs()
            .tasks[0]
            .name,
        "Synthetic separator"
    );
    app.dispatch_ui_command(radishflow_studio::EDIT_REDO_COMMAND_ID);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs()
            .tasks[0]
            .name,
        "Renamed"
    );
    let checkpoint = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .checkpoint();
    run_with_key_press_and_focus(
        egui::Key::Z,
        egui::Modifiers::COMMAND,
        egui::Id::new("equipment-text"),
        |ctx| app.dispatch_shortcuts(ctx),
    );
    assert_eq!(
        checkpoint,
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .checkpoint()
    );
    assert_eq!(app.platform_host.document(), &flow);
    change(&mut app, &task, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.name = "Pending".into();
    });
    app.dispatch_ui_command(radishflow_studio::EDIT_UNDO_COMMAND_ID);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .confirmation()
            .is_some()
    );
    app.equipment_command(EquipmentEditorCommand::CancelConfirmation);
    assert_eq!(pending(&app), 1);
}
#[test]
fn equipment_page_and_confirmation_render_in_both_locales_at_narrow_large_text() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        let (mut app, task, source) = complete_case();
        app.locale = locale;
        let EquipmentEditorTarget::Task(id) = &task else {
            panic!()
        };
        app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        fonts::configure_studio_fonts(&ctx);
        ctx.style_mut(|s| {
            for font in s.text_styles.values_mut() {
                font.size *= 2.0;
            }
        });
        for target in [task.clone(), source] {
            app.equipment_command(EquipmentEditorCommand::Select(target));
            for results in [false, true] {
                app.equipment.section = if results {
                    equipment::EquipmentSection::Results
                } else {
                    equipment::EquipmentSection::Inputs
                };
                let mut labels = String::new();
                for _ in 0..2 {
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(400.0, 900.0),
                            )),
                            ..Default::default()
                        },
                        |ctx| app.render_equipment_page(ctx),
                    );
                    labels = out
                        .platform_output
                        .accesskit_update
                        .unwrap()
                        .nodes
                        .iter()
                        .filter_map(|(_, n)| n.label())
                        .collect::<Vec<_>>()
                        .join("\n");
                }
                assert!(
                    labels.contains(if locale == StudioShellLocale::ZhCn {
                        "应用输入"
                    } else {
                        "Apply inputs"
                    }),
                    "{labels}"
                );
                if locale == StudioShellLocale::En {
                    assert!(
                        !labels
                            .chars()
                            .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)),
                        "{labels}"
                    );
                }
            }
        }
        app.equipment_command(EquipmentEditorCommand::Select(task.clone()));
        change(&mut app, &task, |draft| {
            let EquipmentDraft::Task(t) = draft else {
                panic!()
            };
            t.name = "Unsubmitted".into();
        });
        app.equipment_command(EquipmentEditorCommand::History { redo: false });
        assert!(!app.close_current_window_for_viewport_request());
        for key in [None, None, Some(egui::Key::Escape)] {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 900.0),
                    )),
                    events: key
                        .map(|key| egui::Event::Key {
                            key,
                            physical_key: Some(key),
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        })
                        .into_iter()
                        .collect(),
                    ..Default::default()
                },
                |ctx| app.render_equipment_confirmation(ctx),
            );
        }
        assert!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .equipment_editor
                .confirmation()
                .is_none()
        );
        assert_eq!(pending(&app), 1);
    }
}

#[test]
fn basis_reference_override_and_shared_edit_preview_keep_provenance_and_stale_results() {
    let (mut app, task, source) = complete_case();
    let EquipmentEditorTarget::Task(id) = &task else {
        panic!()
    };
    let EquipmentEditorTarget::Source(source_id) = &source else {
        panic!()
    };
    app.equipment_command(EquipmentEditorCommand::NewBasis);
    let basis = active(&app);
    let EquipmentEditorTarget::Basis(basis_id) = &basis else {
        panic!()
    };
    change(&mut app, &basis, |draft| {
        let EquipmentDraft::Basis(b) = draft else {
            panic!()
        };
        b.name = "Working duration".into();
        b.value.set_text("120".into());
        b.source = Some(source_id.clone());
    });
    app.equipment_command(EquipmentEditorCommand::Apply(basis.clone()));
    app.equipment_command(EquipmentEditorCommand::Select(task.clone()));
    change(&mut app, &task, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        let b = t
            .fields
            .get_mut(&EquipmentField::Input(SeparatorInput::WorkingVolumeTime))
            .unwrap();
        b.mode = BindingMode::BasisRef;
        b.basis = Some(basis_id.clone());
    });
    app.equipment_command(EquipmentEditorCommand::Apply(task.clone()));
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .current_candidate(id)
            .unwrap()
            .dimensions()
            .working_volume_m3,
        0.6
    );
    change(&mut app, &task, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        let b = t
            .fields
            .get_mut(&EquipmentField::Input(SeparatorInput::WorkingVolumeTime))
            .unwrap();
        b.mode = BindingMode::Override;
        b.source = Some(source_id.clone());
        b.value.set_text("60".into());
        b.reason = "Explicit synthetic local criterion".into();
    });
    app.equipment_command(EquipmentEditorCommand::Apply(task.clone()));
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .current_candidate(id)
            .unwrap()
            .dimensions()
            .working_volume_m3,
        0.3
    );
    app.equipment_command(EquipmentEditorCommand::Select(basis.clone()));
    change(&mut app, &basis, |draft| {
        let EquipmentDraft::Basis(b) = draft else {
            panic!()
        };
        b.value.set_text("240".into());
    });
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .error(),
        Some(&EquipmentEditError::PendingDrafts)
    );
    app.equipment_command(EquipmentEditorCommand::Apply(basis.clone()));
    let confirmation = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .confirmation()
        .unwrap()
        .clone();
    assert!(confirmation.impacts()[0].input_changed);
    app.equipment_command(EquipmentEditorCommand::CancelConfirmation);
    assert_eq!(pending(&app), 1);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs()
            .basis
            .basis_entries[0]
            .value
            .value_si,
        120.0
    );
    app.equipment_command(EquipmentEditorCommand::Apply(basis.clone()));
    let confirmation = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .confirmation()
        .unwrap()
        .clone();
    app.equipment_command(EquipmentEditorCommand::Confirm(confirmation.key()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(id),
        EquipmentResultStatus::Stale
    );
    app.equipment_command(EquipmentEditorCommand::Delete(basis));
    assert!(matches!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .error(),
        Some(EquipmentEditError::Domain(
            rf_ui::engineering::EngineeringError::Referenced { .. }
        ))
    ));
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .current_candidate(id)
            .unwrap()
            .dimensions()
            .working_volume_m3,
        0.3
    );
}

#[test]
fn only_related_drafts_block_equipment_calculation_and_adoption_remains_explicit() {
    let (mut app, task, source) = complete_case();
    let EquipmentEditorTarget::Task(id) = &task else {
        panic!()
    };
    app.equipment_command(EquipmentEditorCommand::CreateSource);
    let unrelated = active(&app);
    change(&mut app, &unrelated, |draft| {
        let EquipmentDraft::Source(s) = draft else {
            panic!()
        };
        s.title = "Unrelated pending source".into();
    });
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(id),
        EquipmentResultStatus::Current
    );
    app.equipment_command(EquipmentEditorCommand::Select(source.clone()));
    change(&mut app, &source, |draft| {
        let EquipmentDraft::Source(s) = draft else {
            panic!()
        };
        s.review = SourceReview::Unreviewed;
    });
    app.equipment_command(EquipmentEditorCommand::Apply(source.clone()));
    let key = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .confirmation()
        .unwrap()
        .key();
    app.equipment_command(EquipmentEditorCommand::Confirm(key));
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    let session = &app.platform_host.modeling_inputs().workspace.engineering;
    assert_eq!(session.result_status(id), EquipmentResultStatus::Failed);
    assert!(session.current_candidate(id).is_none());
    assert!(
        session
            .current_failure(id)
            .unwrap()
            .issues
            .iter()
            .any(|issue| issue.kind == separator_sizing::SizingIssueKind::SourceNotAdopted)
    );
    change(&mut app, &source, |draft| {
        let EquipmentDraft::Source(s) = draft else {
            panic!()
        };
        s.review = SourceReview::Adopted;
        s.temperature.set_text("310".into());
    });
    app.equipment_command(EquipmentEditorCommand::Apply(source));
    let key = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .confirmation()
        .unwrap()
        .key();
    app.equipment_command(EquipmentEditorCommand::Confirm(key));
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .current_failure(id)
            .unwrap()
            .issues
            .iter()
            .any(|issue| issue.kind == separator_sizing::SizingIssueKind::SourceConditionMismatch)
    );
}

#[test]
fn equipment_text_focus_survives_pending_banner_and_local_history() {
    use egui::accesskit::{Action, ActionRequest, Role};
    let (mut app, task, _) = complete_case();
    app.locale = StudioShellLocale::En;
    let checkpoint = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .checkpoint();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let frame = |app: &mut ReadyAppState, events| {
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 900.0),
            )),
            focused: true,
            events,
            ..Default::default()
        });
        app.dispatch_shortcuts(&ctx);
        app.render_equipment_page(&ctx);
        ctx.end_pass().platform_output.accesskit_update.unwrap()
    };
    frame(&mut app, vec![]);
    let tree = frame(&mut app, vec![]);
    let tag = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::TextInput && n.value() == Some("V-101"))
        .unwrap()
        .0;
    frame(
        &mut app,
        vec![egui::Event::AccessKitActionRequest(ActionRequest {
            action: Action::Focus,
            target: tag,
            data: None,
        })],
    );
    frame(&mut app, vec![]);
    let key = |key, modifiers| egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    };
    frame(&mut app, vec![key(egui::Key::A, egui::Modifiers::COMMAND)]);
    let tree = frame(&mut app, vec![egui::Event::Text("V-NEW".into())]);
    assert_eq!(tree.focus, tag);
    let tree = frame(&mut app, vec![egui::Event::Text("-TAIL".into())]);
    assert_eq!(tree.focus, tag);
    let EquipmentDraft::Task(draft) = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .draft(&task)
        .unwrap()
        .value()
    else {
        panic!()
    };
    assert_eq!(draft.tag, "V-NEW-TAIL");
    frame(&mut app, vec![key(egui::Key::Z, egui::Modifiers::COMMAND)]);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .checkpoint(),
        checkpoint
    );
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .confirmation()
            .is_none()
    );
}

#[test]
fn equipment_page_labels_and_actions_fit_narrow_and_regular_widths() {
    fn fits(shape: &egui::epaint::Shape, clip: egui::Rect, values: &[String], case: &str) {
        match shape {
            egui::epaint::Shape::Text(text) => {
                let rect = text.galley.rect.translate(text.pos.to_vec2());
                if rect.intersects(clip) && !values.contains(&text.galley.job.text) {
                    assert!(
                        rect.left() >= clip.left() - 0.5 && rect.right() <= clip.right() + 0.5,
                        "{case}: {:?} overflows {clip:?}: {rect:?}",
                        text.galley.job.text
                    );
                }
            }
            egui::epaint::Shape::Vec(shapes) => {
                for shape in shapes {
                    fits(shape, clip, values, case);
                }
            }
            _ => {}
        }
    }
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for width in [400.0, 1280.0] {
            for scale in [1.0, 2.0] {
                let (mut app, task, source) = complete_case();
                app.locale = locale;
                let EquipmentEditorTarget::Task(id) = &task else {
                    panic!()
                };
                app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
                app.equipment_command(EquipmentEditorCommand::NewBasis);
                let basis = active(&app);
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                fonts::configure_studio_fonts(&ctx);
                ctx.style_mut(|s| {
                    for font in s.text_styles.values_mut() {
                        font.size *= scale;
                    }
                });
                for target in [task, source, basis] {
                    app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
                    for section in [
                        equipment::EquipmentSection::Inputs,
                        equipment::EquipmentSection::Results,
                        equipment::EquipmentSection::Evidence,
                    ] {
                        app.equipment.section = section;
                        for _ in 0..2 {
                            let out = ctx.run(
                                egui::RawInput {
                                    screen_rect: Some(egui::Rect::from_min_size(
                                        egui::Pos2::ZERO,
                                        egui::vec2(width, 1000.0),
                                    )),
                                    ..Default::default()
                                },
                                |ctx| app.render_equipment_page(ctx),
                            );
                            let values = out
                                .platform_output
                                .accesskit_update
                                .unwrap()
                                .nodes
                                .iter()
                                .filter(|(_, n)| {
                                    matches!(
                                        n.role(),
                                        egui::accesskit::Role::TextInput
                                            | egui::accesskit::Role::MultilineTextInput
                                    )
                                })
                                .filter_map(|(_, n)| n.value().map(str::to_owned))
                                .collect::<Vec<_>>();
                            let case = format!("{locale:?} {width} {scale} {target:?} {section:?}");
                            for shape in &out.shapes {
                                fits(&shape.shape, shape.clip_rect, &values, &case);
                            }
                        }
                    }
                }
            }
        }
    }
}
