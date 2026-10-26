use super::equipment_editor::{change, complete_case};
use super::equipment_export::apply;
use super::*;
use rf_model::equipment::SeparatorInput;
use rf_ui::engineering::editor::*;

fn frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 1200.0),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| app.render_equipment_page(ctx),
    )
}
fn labels(out: &egui::FullOutput) -> String {
    fn collect(shape: &egui::epaint::Shape, text: &mut Vec<String>) {
        match shape {
            egui::epaint::Shape::Text(value) => text.push(value.galley.job.text.clone()),
            egui::epaint::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, text);
                }
            }
            _ => {}
        }
    }
    let mut text = Vec::new();
    for shape in &out.shapes {
        collect(&shape.shape, &mut text);
    }
    text.join("\n")
}
#[test]
fn equipment_source_keyboard_expansion_is_read_only_and_stale_values_are_collapsed() {
    let (mut app, target, _) = complete_case();
    app.locale = StudioShellLocale::En;
    let EquipmentEditorTarget::Task(id) = &target else {
        panic!()
    };
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
    app.equipment.section = equipment::EquipmentSection::Results;
    let before = app.platform_host.modeling_inputs().workspace.clone();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    ctx.style_mut(|s| s.animation_time = 0.0);
    fonts::configure_studio_fonts(&ctx);
    frame(&mut app, &ctx, vec![]);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(labels(&out).contains("Calculation origin"));
    let origin = out
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, n)| {
            n.role() == egui::accesskit::Role::Button && n.label() == Some("Calculation origin")
        })
        .map(|(id, _)| *id)
        .unwrap();
    frame(
        &mut app,
        &ctx,
        vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target: origin,
                data: None,
            },
        )],
    );
    frame(
        &mut app,
        &ctx,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: Some(egui::Key::Enter),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let out = frame(&mut app, &ctx, vec![]);
    assert!(
        labels(&out).contains("Executed task revision"),
        "{}",
        labels(&out)
    );
    assert_eq!(app.platform_host.modeling_inputs().workspace, before);
    change(&mut app, &target, |draft| {
        let EquipmentDraft::Task(task) = draft else {
            panic!()
        };
        task.fields
            .get_mut(&EquipmentField::Input(SeparatorInput::GasVolumeFlow))
            .unwrap()
            .value
            .set_text("0.9".into());
    });
    apply(&mut app, &target);
    frame(&mut app, &ctx, vec![]);
    let text = labels(&frame(&mut app, &ctx, vec![]));
    assert!(text.contains("Historical evidence · not a current result"));
    assert!(!text.contains("Principal dimensions"));
}

#[test]
fn bottom_result_table_shows_origin_and_disables_foreign_result_navigation() {
    let config = synced_workspace_config();
    let mut owner = ready_app_state(&config);
    owner.dispatch_ui_command("run_panel.run_manual");
    let window = owner.platform_host.snapshot().window_model();
    let mut foreign = ready_app_state(&config);
    foreign.dispatch_ui_command("run_panel.run_manual");
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    for (app, disabled) in [(&mut owner, false), (&mut foreign, true)] {
        app.locale = StudioShellLocale::En;
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default()
                .show(ctx, |ui| app.render_bottom_results_table(ui, &window));
        });
        assert!(labels(&output).contains("Calculation origin"));
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        for name in ["Feed", "feed-1", "heater-1", "flash-1"] {
            let button = nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button && node.label() == Some(name)
                })
                .unwrap();
            assert_eq!(button.1.is_disabled(), disabled);
        }
    }
}
