use super::super::equipment_rating_editor::fits;
use super::*;
use egui::accesskit::{Action, ActionRequest, NodeId, Role};

fn frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    width: f32,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 1000.0),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| {
            app.dispatch_shortcuts(ctx);
            app.render_equipment_page(ctx);
        },
    )
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn focus(id: NodeId) -> egui::Event {
    egui::Event::AccessKitActionRequest(ActionRequest {
        action: Action::Focus,
        target: id,
        data: None,
    })
}
fn button(out: &egui::FullOutput, label: &str) -> NodeId {
    out.platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Button && n.label() == Some(label))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("missing button {label}"))
}
fn activate(app: &mut ReadyAppState, ctx: &egui::Context, label: &str, activation: egui::Key) {
    let out = frame(app, ctx, vec![], 1280.0);
    let id = button(&out, label);
    frame(app, ctx, vec![focus(id)], 1280.0);
    frame(
        app,
        ctx,
        vec![key(activation, egui::Modifiers::NONE)],
        1280.0,
    );
    frame(app, ctx, vec![], 1280.0);
}

#[test]
fn adoption_keyboard_target_preview_confirm_escape_and_modal_shortcuts() {
    let (mut app, source) = current_case();
    app.locale = StudioShellLocale::En;
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    frame(&mut app, &ctx, vec![], 1280.0);
    activate(&mut app, &ctx, "Adopt as design geometry", egui::Key::Enter);
    assert!(app.equipment.adoption.is_some()); // Opening Enter must not cancel or submit.
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .clone();
    frame(
        &mut app,
        &ctx,
        vec![
            key(egui::Key::F5, egui::Modifiers::NONE),
            key(egui::Key::Z, egui::Modifiers::COMMAND),
            key(egui::Key::S, egui::Modifiers::COMMAND),
        ],
        1280.0,
    );
    assert_eq!(
        app.platform_host.modeling_inputs().workspace.engineering,
        before
    );
    let out = frame(&mut app, &ctx, vec![], 1280.0);
    let cancel = button(&out, "Cancel");
    assert_eq!(
        out.platform_output.accesskit_update.as_ref().unwrap().focus,
        cancel,
        "{:?}",
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(id, _)| *id == out.platform_output.accesskit_update.as_ref().unwrap().focus)
            .collect::<Vec<_>>()
    );
    let out = frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
        1280.0,
    );
    assert_ne!(
        out.platform_output.accesskit_update.as_ref().unwrap().focus,
        cancel
    );
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Tab, egui::Modifiers::SHIFT)],
        1280.0,
    );
    let out = frame(&mut app, &ctx, vec![], 1280.0);
    assert_eq!(
        out.platform_output.accesskit_update.as_ref().unwrap().focus,
        cancel,
        "shift-tab {:?}",
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(id, _)| *id == out.platform_output.accesskit_update.as_ref().unwrap().focus)
            .collect::<Vec<_>>()
    );
    activate(&mut app, &ctx, "Preview adopted geometry", egui::Key::Enter);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .geometry_adoption_preview()
            .is_some()
    );
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
        1280.0,
    );
    assert!(app.equipment.adoption.is_none());
    assert_eq!(
        app.platform_host.modeling_inputs().workspace.engineering,
        before
    );
    app.begin_geometry_adoption(&source);
    app.equipment.adoption.as_mut().unwrap().copy_case = true;
    frame(&mut app, &ctx, vec![], 1280.0);
    activate(&mut app, &ctx, "Preview adopted geometry", egui::Key::Space);
    activate(&mut app, &ctx, "Confirm adoption", egui::Key::Enter);
    assert!(app.equipment.adoption.is_none());
    let w = &app.platform_host.modeling_inputs().workspace;
    let t = w
        .engineering
        .inputs()
        .tasks
        .iter()
        .find_map(|t| t.as_rating())
        .unwrap();
    assert_eq!(t.cases[0].inputs.len(), 9);
    assert_eq!(
        w.engineering.case_result_status(&EquipmentCaseKey {
            task_id: t.id.clone(),
            case_id: t.cases[0].id.clone()
        }),
        EquipmentResultStatus::Missing
    );
}

#[test]
fn adoption_evidence_keyboard_return_restores_geometry_draft_focus_without_submission() {
    let (mut app, path, member) = fixture("adoption-focus");
    app.locale = StudioShellLocale::En;
    app.equipment
        .rating_overview
        .insert(member.task_id.clone(), false);
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    frame(&mut app, &ctx, vec![], 1280.0);
    let out = frame(&mut app, &ctx, vec![], 1280.0);
    let input = out
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, n)| n.role() == Role::TextInput && n.label() == Some("Given internal diameter"))
        .map(|(id, _)| id)
        .expect("geometry input");
    frame(&mut app, &ctx, vec![focus(input)], 1280.0);
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::A, egui::Modifiers::COMMAND)],
        1280.0,
    );
    frame(
        &mut app,
        &ctx,
        vec![egui::Event::Text("1e-".into())],
        1280.0,
    );
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .inputs()
        .clone();
    activate(&mut app, &ctx, "View adoption evidence", egui::Key::Enter);
    assert!(app.equipment.adoption_evidence.is_some());
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
        1280.0,
    );
    let out = frame(&mut app, &ctx, vec![], 1280.0);
    assert_eq!(
        out.platform_output.accesskit_update.as_ref().unwrap().focus,
        input,
        "{:?}",
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(id, _)| *id == out.platform_output.accesskit_update.as_ref().unwrap().focus)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs(),
        &before
    );
    let target = EquipmentEditorTarget::RatingGeometry(member.task_id);
    let EquipmentDraft::RatingGeometry(d) = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .draft(&target)
        .unwrap()
        .value()
    else {
        panic!()
    };
    assert_eq!(
        d.fields[&RatingGeometryField::Input(SeparatorGeometryInput::InternalDiameter)]
            .value
            .text(),
        "1e-"
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn adoption_target_preview_evidence_fit_languages_widths_and_font_scales() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for width in [400.0, 1440.0] {
            for scale in [1.0, 2.0] {
                let (mut app, source) = current_case();
                app.locale = locale;
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                fonts::configure_studio_fonts(&ctx);
                ctx.style_mut(|s| {
                    for font in s.text_styles.values_mut() {
                        font.size *= scale;
                    }
                });
                app.begin_geometry_adoption(&source);
                for stage in 0..3 {
                    if stage == 1 {
                        app.equipment.adoption.as_mut().unwrap().diameter =
                            Some(DimensionAdjustmentDraft {
                                value: QuantityDraft::from_si(
                                    1.2,
                                    QuantityKind::Length,
                                    MeasurementUnit::Millimetre,
                                ),
                                reason: "明确选择 1200 mm · Explicit design choice".into(),
                            });
                        app.preview_geometry_adoption_form();
                    }
                    if stage == 2 {
                        app.confirm_geometry_adoption_form();
                        let w = &app.platform_host.modeling_inputs().workspace;
                        let id = w
                            .engineering
                            .inputs()
                            .tasks
                            .iter()
                            .find_map(|t| t.as_rating())
                            .unwrap()
                            .id
                            .clone();
                        app.open_adoption_evidence(id, None);
                    }
                    for _ in 0..3 {
                        let out = frame(&mut app, &ctx, vec![], width);
                        let values = out
                            .platform_output
                            .accesskit_update
                            .unwrap()
                            .nodes
                            .iter()
                            .filter(|(_, n)| {
                                matches!(n.role(), Role::TextInput | Role::MultilineTextInput)
                            })
                            .filter_map(|(_, n)| n.value().map(str::to_owned))
                            .collect::<Vec<_>>();
                        for shape in out.shapes {
                            fits(
                                &shape.shape,
                                shape.clip_rect,
                                &values,
                                &format!("adoption {locale:?} {width} {scale} stage {stage}"),
                            );
                        }
                    }
                }
            }
        }
    }
}
