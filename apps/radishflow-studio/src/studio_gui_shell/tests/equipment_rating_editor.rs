use super::equipment_editor::change;
use super::equipment_export::apply;
use super::equipment_rating_export::{calculate, fixture, member};
use super::*;
use rf_model::equipment::*;
use rf_ui::engineering::{EquipmentResultStatus, editor::*};

#[test]
fn rating_creation_is_explicit_and_has_no_synthetic_inputs() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.start_independent_equipment_project();
    assert!(app.equipment.creating_task);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs()
            .tasks
            .is_empty()
    );
    app.create_equipment_task(equipment::EquipmentTaskMode::Rating);
    let inputs = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .inputs();
    let task = inputs.tasks[0].as_rating().unwrap();
    assert!(task.geometry.inputs.is_empty());
    assert!(task.geometry.axial_sections.is_empty());
    assert_eq!(task.cases.len(), 1);
    assert!(task.cases[0].inputs.is_empty());
    assert!(inputs.basis.sources.is_empty());
}

#[test]
fn rating_check_calculate_labels_and_geometry_confirmation_follow_application_state() {
    let (mut app, project) = fixture("rating-editor-transaction", |_| {});
    let key = member();
    app.equipment_command(EquipmentEditorCommand::CheckRating(key.clone()));
    let w = &app.platform_host.modeling_inputs().workspace;
    assert!(
        w.equipment_editor
            .checked_rating_issues(&key, &w.engineering)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        w.engineering.case_result_status(&key),
        EquipmentResultStatus::Missing
    );
    app.calculate_active_equipment();
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&key),
        EquipmentResultStatus::Current
    );
    let target = EquipmentEditorTarget::RatingTask(key.task_id.clone());
    change(&mut app, &target, |d| {
        let EquipmentDraft::RatingTask(d) = d else {
            panic!()
        };
        d.tag = "V-RENAMED".into();
    });
    apply(&mut app, &target);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&key),
        EquipmentResultStatus::Current
    );
    let geometry = EquipmentEditorTarget::RatingGeometry(key.task_id.clone());
    app.equipment_command(EquipmentEditorCommand::Select(geometry.clone()));
    change(&mut app, &geometry, |d| {
        let EquipmentDraft::RatingGeometry(d) = d else {
            panic!()
        };
        d.fields
            .get_mut(&RatingGeometryField::Input(
                SeparatorGeometryInput::WorkingLiquidHeight,
            ))
            .unwrap()
            .value
            .set_text("700".into());
    });
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .inputs()
        .clone();
    app.equipment_command(EquipmentEditorCommand::Apply(geometry.clone()));
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .confirmation()
            .is_some()
    );
    app.equipment_command(EquipmentEditorCommand::CancelConfirmation);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs(),
        &before
    );
    apply(&mut app, &geometry);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&key),
        EquipmentResultStatus::Stale
    );
    assert!(
        app.capture_equipment_output(&key.task_id, equipment::CalculationTableFormat::Json)
            .is_none()
    );
    app.equipment_command(EquipmentEditorCommand::History { redo: false });
    let restored = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .inputs()
        .rating_task(&key.task_id)
        .unwrap();
    assert_eq!(
        restored.geometry.inputs,
        before.rating_task(&key.task_id).unwrap().geometry.inputs
    );
    assert!(
        restored.geometry.revision > before.rating_task(&key.task_id).unwrap().geometry.revision
    );
    assert_ne!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&key),
        EquipmentResultStatus::Current
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

pub(super) fn fits(shape: &egui::epaint::Shape, clip: egui::Rect, values: &[String], case: &str) {
    match shape {
        egui::epaint::Shape::Text(t) => {
            let rect = t.galley.rect.translate(t.pos.to_vec2());
            if rect.intersects(clip) && !values.contains(&t.galley.job.text) {
                assert!(
                    rect.left() >= clip.left() - 0.5 && rect.right() <= clip.right() + 0.5,
                    "{case}: {:?} overflows {clip:?}: {rect:?}",
                    t.galley.job.text
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
#[test]
fn rating_page_actions_and_evidence_fit_both_languages_narrow_and_doubled_text() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for width in [400.0, 1280.0] {
            for scale in [1.0, 2.0] {
                let (mut app, project) = fixture("rating-layout", |_| {});
                calculate(&mut app);
                app.locale = locale;
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                fonts::configure_studio_fonts(&ctx);
                ctx.style_mut(|s| {
                    for font in s.text_styles.values_mut() {
                        font.size *= scale;
                    }
                });
                for target in [
                    EquipmentEditorTarget::RatingGeometry(member().task_id),
                    EquipmentEditorTarget::RatingCase(member()),
                    EquipmentEditorTarget::RatingTask(member().task_id),
                ] {
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
                            for shape in out.shapes {
                                fits(
                                    &shape.shape,
                                    shape.clip_rect,
                                    &values,
                                    &format!("{locale:?} {width} {scale} {target:?} {section:?}"),
                                );
                            }
                        }
                    }
                }
                fs::remove_dir_all(project.parent().unwrap()).unwrap();
            }
        }
    }
}

#[test]
fn rating_numeric_focus_source_return_and_f5_preserve_unsubmitted_input() {
    use egui::accesskit::{Action, ActionRequest, Role};
    let (mut app, project) = fixture("rating-focus-return", |_| {});
    app.locale = StudioShellLocale::En;
    let geometry = EquipmentEditorTarget::RatingGeometry(member().task_id);
    app.equipment_command(EquipmentEditorCommand::Select(geometry.clone()));
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let frame = |app: &mut ReadyAppState, events| {
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 1000.0),
            )),
            focused: true,
            events,
            ..Default::default()
        });
        app.dispatch_shortcuts(&ctx);
        app.render_equipment_page(&ctx);
        ctx.end_pass().platform_output.accesskit_update.unwrap()
    };
    let key = |key, modifiers| egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    };
    let action = |target, action| {
        egui::Event::AccessKitActionRequest(ActionRequest {
            target,
            action,
            data: None,
        })
    };
    frame(&mut app, vec![]);
    let tree = frame(&mut app, vec![]);
    let input = tree
        .nodes
        .iter()
        .find(|(_, n)| {
            n.role() == Role::TextInput && n.value().is_some_and(|v| v.starts_with("1128.379"))
        })
        .map(|(id, _)| *id)
        .unwrap_or_else(|| {
            panic!(
                "numeric values: {:?}",
                tree.nodes
                    .iter()
                    .filter_map(|(_, n)| n.value())
                    .collect::<Vec<_>>()
            )
        });
    frame(&mut app, vec![action(input, Action::Focus)]);
    frame(&mut app, vec![]);
    frame(&mut app, vec![key(egui::Key::A, egui::Modifiers::COMMAND)]);
    let tree = frame(&mut app, vec![egui::Event::Text("1e-".into())]);
    assert_eq!(tree.focus, input);
    frame(&mut app, vec![]);
    let tree = frame(&mut app, vec![]);
    let source = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == Role::Button && n.label() == Some("Edit / view source"))
        .min_by(|(_, a), (_, b)| a.bounds().unwrap().y0.total_cmp(&b.bounds().unwrap().y0))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let pos = egui::pos2(
        ((source.x0 + source.x1) / 2.0) as f32,
        ((source.y0 + source.y1) / 2.0) as f32,
    );
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let tree = frame(&mut app, vec![]);
    assert!(
        tree.nodes
            .iter()
            .any(|(_, n)| n.value() == Some("Source details · committed record")),
        "{:?}",
        tree.nodes
            .iter()
            .filter_map(|(_, n)| n.label())
            .collect::<Vec<_>>()
    );
    frame(
        &mut app,
        vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
    );
    let tree = frame(&mut app, vec![]);
    assert_eq!(tree.focus, input);
    let w = &app.platform_host.modeling_inputs().workspace;
    assert_eq!(w.equipment_editor.active(), Some(&geometry));
    let EquipmentDraft::RatingGeometry(d) = w.equipment_editor.draft(&geometry).unwrap().value()
    else {
        panic!()
    };
    assert_eq!(
        d.fields[&RatingGeometryField::Input(SeparatorGeometryInput::InternalDiameter)]
            .value
            .text(),
        "1e-"
    );
    frame(&mut app, vec![key(egui::Key::F5, egui::Modifiers::NONE)]);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    app.equipment_command(EquipmentEditorCommand::Discard(geometry));
    ctx.memory_mut(|m| {
        if let Some(id) = m.focused() {
            m.surrender_focus(id);
        }
    });
    frame(&mut app, vec![key(egui::Key::F5, egui::Modifiers::NONE)]);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

#[test]
fn failed_rating_issue_navigates_to_the_given_geometry_field() {
    let (mut app, project) = fixture("rating-failed-location", |v| {
        v["engineering"]["tasks"][0]["geometry"]["inputs"]["internalDiameter"]["valueSi"] =
            0.0.into();
    });
    app.locale = StudioShellLocale::En;
    app.calculate_active_equipment();
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&member()),
        EquipmentResultStatus::Failed
    );
    assert!(
        app.capture_equipment_output(&member().task_id, equipment::CalculationTableFormat::Json)
            .is_none()
    );
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let frame = |app: &mut ReadyAppState, events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280.0, 1000.0),
                )),
                focused: true,
                events,
                ..Default::default()
            },
            |ctx| app.render_equipment_page(ctx),
        )
    };
    frame(&mut app, vec![]);
    let tree = frame(&mut app, vec![])
        .platform_output
        .accesskit_update
        .unwrap();
    let rect = tree
        .nodes
        .iter()
        .find(|(_, n)| {
            [n.label(), n.value()]
                .into_iter()
                .flatten()
                .any(|s| s == "Given internal diameter：Must be positive")
        })
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let pos = egui::pos2(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    );
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let tree = frame(&mut app, vec![])
        .platform_output
        .accesskit_update
        .unwrap();
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .active(),
        Some(&EquipmentEditorTarget::RatingGeometry(member().task_id))
    );
    assert!(tree.nodes.iter().any(|(id, n)| *id == tree.focus
        && n.role() == egui::accesskit::Role::TextInput
        && n.label() == Some("Given internal diameter")));
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}
