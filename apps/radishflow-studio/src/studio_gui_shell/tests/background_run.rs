use super::*;
use radishflow_studio::{StudioBackgroundRunAction as Action, StudioRunOutcome, WorkspaceRunPhase};
use rf_ui::engineering::{EquipmentResultStatus, editor::*};
use std::{
    thread,
    time::{Duration, Instant},
};

fn flow_app() -> ReadyAppState {
    let mut app = ready_app_state(&synced_workspace_config());
    app.screen = StudioShellScreen::Workbench;
    app.enable_background_runs();
    app
}
fn start_flow(app: &mut ReadyAppState) -> radishflow_studio::StudioRunActivity {
    app.dispatch_ui_command("run_panel.run_manual");
    app.background_runs
        .as_ref()
        .unwrap()
        .borrow()
        .activity()
        .expect("background accepted")
}
fn drain(app: &mut ReadyAppState) {
    let ctx = egui::Context::default();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        app.poll_background_runs(&ctx);
        if !app.background_busy() {
            break;
        }
        assert!(Instant::now() < deadline, "background UI did not drain");
        thread::yield_now();
    }
}
fn commit_pressure(app: &mut ReadyAppState, value: &str) {
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        value,
    );
    app.dispatch_inspector_field_draft_commit(
        radishflow_studio::inspector_draft_commit_command_id(key),
    );
}

#[test]
fn gui_manual_submission_and_repeated_command_keep_one_task_then_refresh_snapshot() {
    let mut app = flow_app();
    let task = start_flow(&mut app);
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_none()
    );
    assert_eq!(task.phase, WorkspaceRunPhase::Preparing);
    assert!(!app.background_command_enabled("run_panel.run_manual"));
    app.dispatch_ui_command("run_panel.run_manual");
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .id,
        task.id
    );
    drain(&mut app);
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_some()
    );
    let owner = app.background_runs.as_ref().unwrap().borrow();
    assert_eq!(owner.history().len(), 1);
    assert_eq!(
        owner.history().back().unwrap().outcome,
        StudioRunOutcome::Succeeded
    );
    assert!(owner.history().back().unwrap().applied);
}

#[test]
fn active_formal_edit_cancels_old_and_runs_latest_once_but_hold_only_stops_future_work() {
    let mut app = flow_app();
    app.dispatch_ui_command("run_panel.set_active");
    let task = start_flow(&mut app);
    commit_pressure(&mut app, "110000");
    commit_pressure(&mut app, "105000");
    let final_revision = app.platform_host.document().revision;
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .id,
        task.id
    );
    drain(&mut app);
    let owner = app.background_runs.as_ref().unwrap().borrow();
    assert_eq!(owner.history().len(), 2);
    assert!(!owner.history()[0].applied);
    assert!(owner.history()[1].applied);
    drop(owner);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .snapshot_history
            .back()
            .unwrap()
            .document_revision,
        final_revision
    );
    start_flow(&mut app);
    app.dispatch_ui_command("run_panel.set_hold");
    assert_ne!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .phase,
        WorkspaceRunPhase::CancelRequested
    );
    drain(&mut app);
    assert!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .history()
            .back()
            .unwrap()
            .applied
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .solve_session
            .mode,
        SimulationMode::Hold
    );
}

#[test]
fn stale_palette_cancel_stays_bound_to_original_task() {
    let mut app = flow_app();
    let old = start_flow(&mut app);
    app.command_palette.open();
    app.capture_background_palette_target();
    let command = format!("background.cancel.{}", old.number);
    drain(&mut app);
    let new = start_flow(&mut app);
    app.dispatch_ui_command(command);
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .id,
        new.id
    );
    assert_ne!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .phase,
        WorkspaceRunPhase::CancelRequested
    );
    app.command_palette.close();
    app.command_palette.open();
    app.capture_background_palette_target();
    app.dispatch_ui_command(format!("background.cancel.{}", new.number));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .solve_session
            .mode,
        SimulationMode::Hold
    );
    drain(&mut app);
    assert!(
        !app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .history()
            .back()
            .unwrap()
            .applied
    );
}

#[test]
fn failed_open_preserves_run_while_successful_replacement_detaches_and_keeps_shared_owner() {
    let mut app = flow_app();
    let task = start_flow(&mut app);
    app.open_project(
        test_preferences_path("missing-project").with_extension("does-not-exist"),
        "test",
    );
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .id,
        task.id
    );
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .phase,
        WorkspaceRunPhase::Preparing
    );
    let owner = app.background_runs.as_ref().unwrap().clone();
    app.create_blank_project_with_units(None, rf_types::units::DisplayUnitSet::si());
    assert!(std::rc::Rc::ptr_eq(
        &owner,
        app.background_runs.as_ref().unwrap()
    ));
    assert!(app.background_busy());
    let blank = app.platform_host.modeling_inputs().workspace.clone();
    drain(&mut app);
    assert_eq!(app.platform_host.modeling_inputs().workspace, blank);
    assert!(!owner.borrow().history().back().unwrap().applied);
}

#[test]
fn dirty_close_confirm_then_return_preserves_drafts_and_requires_new_confirmation() {
    let mut app = flow_app();
    start_flow(&mut app);
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        "110000",
    );
    let before = app.platform_host.snapshot().runtime.workspace_document;
    assert!(!app.close_current_window_for_viewport_request());
    assert!(app.project_open.pending_close_window_confirmation.is_some());
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .phase,
        WorkspaceRunPhase::Preparing
    );
    assert!(!app.confirm_pending_close_window());
    assert!(app.pending_run_exit.is_some());
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    app.return_from_background_exit();
    drain(&mut app);
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    assert_eq!(app.logical_window_count(), 1);
    assert!(!app.close_current_window_for_viewport_request());
    assert!(app.project_open.pending_close_window_confirmation.is_some());
}

#[test]
fn closing_one_observer_does_not_cancel_shared_work() {
    let mut app = flow_app();
    app.dispatch_event(StudioGuiEvent::OpenWindowRequested);
    assert_eq!(app.logical_window_count(), 2);
    let task = start_flow(&mut app);
    assert!(!app.close_current_window_for_viewport_request());
    assert_eq!(app.logical_window_count(), 1);
    assert!(app.pending_run_exit.is_none());
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .id,
        task.id
    );
    assert_ne!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .phase,
        WorkspaceRunPhase::CancelRequested
    );
    drain(&mut app);
    assert!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .history()
            .back()
            .unwrap()
            .applied
    );
}

#[test]
fn clean_last_window_waits_for_reaping_before_viewport_close() {
    let mut app = flow_app();
    start_flow(&mut app);
    assert!(!app.close_current_window_for_viewport_request());
    assert!(app.project_open.pending_close_window_confirmation.is_none());
    assert!(app.pending_run_exit.is_some());
    assert_eq!(app.logical_window_count(), 1);
    drain(&mut app);
    let ctx = egui::Context::default();
    let out = ctx.run(Default::default(), |ctx| {
        assert!(app.render_background_exit(ctx));
    });
    assert_eq!(app.logical_window_count(), 0);
    assert!(out.viewport_output.values().any(|viewport| {
        viewport
            .commands
            .iter()
            .any(|command| matches!(command, egui::ViewportCommand::Close))
    }));
    assert!(!app.background_busy());
}

#[test]
fn read_only_overlay_detaches_but_return_does_not_revive_automatic_work() {
    let mut app = flow_app();
    app.dispatch_ui_command("run_panel.set_active");
    start_flow(&mut app);
    let original = app.platform_host.document().clone();
    let path = super::equipment_storage::v3_file("background-readonly", true);
    app.open_project(path.clone(), "test");
    assert!(app.read_only_project.is_some());
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .phase,
        WorkspaceRunPhase::CancelRequested
    );
    drain(&mut app);
    app.read_only_project = None;
    let count = app
        .background_runs
        .as_ref()
        .unwrap()
        .borrow()
        .history()
        .len();
    drain(&mut app);
    assert_eq!(app.platform_host.document(), &original);
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .history()
            .len(),
        count
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_none()
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn sizing_retains_last_candidate_but_blocks_adoption_and_exports_until_drained() {
    let (mut app, target, _) = super::equipment_editor::complete_case();
    let EquipmentEditorTarget::Task(id) = target else {
        panic!("sizing")
    };
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    let previous = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .calculation_id(&id)
        .cloned();
    app.enable_background_runs();
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(&id),
        EquipmentResultStatus::Current
    );
    assert!(
        app.capture_equipment_output(&id, equipment::CalculationTableFormat::Json)
            .is_none()
    );
    app.begin_geometry_adoption(&id);
    assert!(app.equipment.adoption.is_none());
    let activity = app
        .background_runs
        .as_ref()
        .unwrap()
        .borrow()
        .activity()
        .unwrap();
    app.background_action(Action::Cancel(activity.id));
    assert!(
        app.capture_equipment_output(&id, equipment::CalculationTableFormat::Json)
            .is_none()
    );
    drain(&mut app);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .calculation_id(&id),
        previous.as_ref()
    );
    assert!(
        app.capture_equipment_output(&id, equipment::CalculationTableFormat::Json)
            .is_some()
    );
}

#[test]
fn rating_recalculation_uses_precise_member_and_cancel_never_restores_old_current() {
    let (mut app, project) = super::equipment_rating_export::fixture("background-rating", |_| {});
    let member = super::equipment_rating_export::member();
    super::equipment_rating_export::calculate(&mut app);
    app.enable_background_runs();
    app.calculate_active_equipment();
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&member),
        EquipmentResultStatus::Stale
    );
    assert!(app.equipment_target_busy(&member.task_id, Some(&member.case_id)));
    let activity = app
        .background_runs
        .as_ref()
        .unwrap()
        .borrow()
        .activity()
        .unwrap();
    assert!(activity.target.contains(member.case_id.as_str()));
    app.background_action(Action::Cancel(activity.id));
    drain(&mut app);
    assert!(
        app.capture_rating_output(&member, equipment::CalculationTableFormat::Json)
            .is_none()
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&member),
        EquipmentResultStatus::Stale
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

#[test]
fn selecting_another_rating_row_keeps_cancel_target_and_unrelated_exports_available() {
    let (mut app, project) = super::equipment_rating_export::fixture("background-matrix", |data| {
        let task = &mut data["engineering"]["tasks"][0];
        task["method"]["version"] = serde_json::json!(2);
        let mut other = task["cases"][0].clone();
        other["id"] = serde_json::json!("other");
        task["cases"].as_array_mut().unwrap().push(other);
    });
    let running = super::equipment_rating_export::member();
    let other = rf_ui::engineering::EquipmentCaseKey {
        task_id: running.task_id.clone(),
        case_id: "other".into(),
    };
    app.equipment_command(EquipmentEditorCommand::CalculateRating(running.clone()));
    app.equipment_command(EquipmentEditorCommand::CalculateRating(other.clone()));
    let previous = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .case_calculation_id(&other)
        .cloned();
    app.enable_background_runs();
    app.equipment_command(EquipmentEditorCommand::CalculateRating(running.clone()));
    let activity = app
        .background_runs
        .as_ref()
        .unwrap()
        .borrow()
        .activity()
        .unwrap();
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingCase(other.clone()),
    ));
    let before_view = app.platform_host.modeling_inputs().workspace.clone();
    app.show_background_run_details();
    assert_eq!(app.screen, StudioShellScreen::Equipment);
    assert_eq!(app.platform_host.modeling_inputs().workspace, before_view);
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .unwrap()
            .id,
        activity.id
    );
    assert!(
        app.capture_rating_output(&other, equipment::CalculationTableFormat::Json)
            .is_some()
    );
    assert!(
        app.capture_rating_output(&running, equipment::CalculationTableFormat::Json)
            .is_none()
    );
    assert!(
        app.capture_matrix_output(&running.task_id, equipment::CalculationTableFormat::Json)
            .is_some()
    );
    app.background_action(Action::Cancel(activity.id));
    drain(&mut app);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_calculation_id(&other),
        previous.as_ref()
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&running),
        EquipmentResultStatus::Stale
    );
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

#[test]
fn activity_and_exit_wait_fit_narrow_bilingual_double_text_without_cancelling_on_escape() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for scale in [1.0, 2.0] {
            let mut app = flow_app();
            app.locale = locale;
            start_flow(&mut app);
            let ctx = egui::Context::default();
            let mut style = (*ctx.style()).clone();
            for font in style.text_styles.values_mut() {
                font.size *= scale;
            }
            ctx.set_style(style);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 900.0),
                )),
                ..Default::default()
            };
            for _ in 0..2 {
                let out = ctx.run(input.clone(), |ctx| {
                    app.render_background_activity(ctx);
                    egui::CentralPanel::default().show(ctx, |_| {});
                });
                for shape in out.shapes {
                    super::equipment_rating_editor::fits(
                        &shape.shape,
                        shape.clip_rect,
                        &[],
                        "background activity",
                    );
                }
            }
            let window = app.current_window_id().unwrap();
            app.begin_background_exit(window);
            let out = ctx.run(input.clone(), |ctx| {
                assert!(app.render_background_exit(ctx));
            });
            for shape in out.shapes {
                super::equipment_rating_editor::fits(
                    &shape.shape,
                    shape.clip_rect,
                    &[],
                    "background exit",
                );
            }
            let mut escape = input;
            escape.events.push(egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
            let _ = ctx.run(escape, |ctx| {
                app.render_background_exit(ctx);
            });
            assert!(app.pending_run_exit.is_none());
            drain(&mut app);
            assert_eq!(app.logical_window_count(), 1);
        }
    }
}

#[test]
fn background_result_projection_and_export_share_executed_origin() {
    let mut app = flow_app();
    start_flow(&mut app);
    drain(&mut app);
    let window = app.platform_host.snapshot().window_model();
    let snapshot = window.runtime.latest_solve_snapshot.as_ref().unwrap();
    let origin = snapshot.origin.as_ref().unwrap();
    let package = origin.execution().unwrap().package.as_ref().unwrap();
    assert_eq!(window.runtime.result_review.origin.as_ref(), Some(origin));
    assert!(
        snapshot
            .result_guard
            .as_ref()
            .unwrap()
            .is_current(&app.platform_host.modeling_inputs().workspace)
    );
    let text = snapshot.light_text_export().unwrap();
    for value in [
        &package.package_id,
        &package.version,
        &package.payload_fingerprint,
    ] {
        assert!(text.contains(value));
    }
    let inspector = snapshot.result_inspector_with_unit(None, None, None);
    for action in inspector
        .diagnostic_actions
        .iter()
        .chain(&inspector.unit_diagnostic_actions)
    {
        assert!(action.action.result_guard.is_some());
        assert!(
            action
                .action
                .result_is_current(&app.platform_host.modeling_inputs().workspace)
        );
    }
}

fn wait_for_unit(app: &mut ReadyAppState, step: usize) -> radishflow_studio::StudioRunActivity {
    let deadline = Instant::now() + Duration::from_secs(10);
    let ctx = egui::Context::default();
    loop {
        app.poll_background_runs(&ctx);
        let activity = app
            .background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .activity()
            .expect("active flow");
        if matches!(&activity.unit_control, Some(c) if matches!(c.phase, radishflow_studio::UnitRunPhase::Paused { step: s, .. } if s == step))
        {
            return activity;
        }
        assert!(
            Instant::now() < deadline,
            "did not reach stop {step}: {activity:?}"
        );
        thread::yield_now();
    }
}

#[test]
fn stale_palette_step_cannot_release_the_next_stop_or_a_new_run() {
    use crate::studio_gui_shell::background_run::START_PAUSED;
    let mut app = flow_app();
    app.dispatch_ui_command(START_PAUSED);
    let first = wait_for_unit(&mut app, 1);
    app.command_palette.open();
    app.capture_background_palette_target();
    let mut items = Vec::new();
    app.command_palette.query = "step".into();
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 900.0));
    let window = app.platform_host.snapshot().window_model();
    for _ in 0..3 {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ctx| app.render_command_palette(ctx, &window.commands),
        );
    }
    let palette = ctx
        .memory(|m| m.area_rect(egui::Id::new("studio.command_palette")))
        .unwrap();
    assert!(
        screen.contains_rect(palette),
        "palette {palette:?} exceeds {screen:?}"
    );
    app.append_background_palette_items(&mut items, "step");
    let command = items.iter().find(|i| i.enabled).unwrap().command_id.clone();
    app.dispatch_ui_command(&command);
    wait_for_unit(&mut app, 2);
    items.clear();
    app.append_background_palette_items(&mut items, "step");
    assert!(items.iter().any(|i| i.command_id == command && !i.enabled));
    app.dispatch_ui_command(&command);
    assert_eq!(wait_for_unit(&mut app, 2).id, first.id);
    app.background_action(Action::Cancel(first.id.clone()));
    drain(&mut app);
    app.dispatch_ui_command(START_PAUSED);
    let new = wait_for_unit(&mut app, 1);
    assert_ne!(new.id, first.id);
    app.dispatch_ui_command(command);
    assert_eq!(wait_for_unit(&mut app, 1).id, new.id);
    app.background_action(Action::Cancel(new.id));
    drain(&mut app);
}

#[test]
fn duplicate_first_stop_does_not_change_an_ordinary_run_and_readonly_cannot_submit() {
    use radishflow_studio::{
        flow_actions::FlowEditContext,
        run_actions::{StudioRunAccess, StudioRunReply},
    };
    let mut app = flow_app();
    let expected = FlowEditContext::capture(&app.platform_host.modeling_inputs().workspace);
    let submit = |app: &mut ReadyAppState, access| {
        app.platform_host
            .background_run_action(Action::StartAtFirstUnit {
                expected: expected.clone(),
                access,
            })
            .unwrap()
    };
    let denied = submit(&mut app, StudioRunAccess::ReadOnly);
    assert!(matches!(
        denied.submission,
        Some(StudioRunReply::Blocked(_))
    ));
    assert!(!app.background_busy());
    let run = start_flow(&mut app);
    let duplicate = submit(&mut app, StudioRunAccess::Allowed);
    assert_eq!(duplicate.unit_control, None);
    assert!(
        matches!(duplicate.submission, Some(StudioRunReply::AlreadyRunning(id)) if id == run.id)
    );
    drain(&mut app);
    assert!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .history()
            .back()
            .unwrap()
            .applied
    );
}

#[test]
fn stopped_activity_fits_400_points_and_keyboard_steps_exactly_one_unit() {
    use super::runtime::{assert_horizontal_fit, collect_shape_texts};
    use crate::studio_gui_shell::background_run::START_PAUSED;
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for scale in [1.0, 2.0] {
            let mut app = flow_app();
            app.locale = locale;
            app.dispatch_ui_command(START_PAUSED);
            let first = wait_for_unit(&mut app, 1);
            let ctx = egui::Context::default();
            fonts::configure_studio_fonts(&ctx);
            ctx.style_mut(|style| {
                for font in style.text_styles.values_mut() {
                    font.size *= scale;
                }
            });
            let frame = |app: &mut ReadyAppState, key: Option<egui::Key>| {
                let mut input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 1100.0),
                    )),
                    ..Default::default()
                };
                if let Some(key) = key {
                    input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                ctx.run(input, |ctx| app.render_background_activity(ctx))
            };
            let mut texts = Vec::new();
            frame(&mut app, None);
            for shape in frame(&mut app, None).shapes {
                assert_horizontal_fit(&shape.shape, shape.clip_rect, "stopped activity");
                collect_shape_texts(&shape.shape, &mut texts);
            }
            let joined = texts.join(" ");
            let labels = if locale == StudioShellLocale::ZhCn {
                ["已暂停", "单步（一个单元）", "继续本次运行", "取消本次运行"]
            } else {
                [
                    "Paused",
                    "Step one unit",
                    "Continue this run",
                    "Cancel this run",
                ]
            };
            for label in labels {
                assert!(joined.contains(label), "{label} missing from {joined}");
            }
            frame(&mut app, Some(egui::Key::Tab));
            let status = ctx.memory(|m| m.focused()).expect("focusable stop status");
            frame(&mut app, Some(egui::Key::Tab));
            assert_ne!(ctx.memory(|m| m.focused()), Some(status));
            frame(&mut app, Some(egui::Key::Enter));
            assert_eq!(wait_for_unit(&mut app, 2).id, first.id);
            frame(&mut app, None);
            assert_eq!(ctx.memory(|m| m.focused()), Some(status));
            app.background_action(Action::Cancel(first.id));
            drain(&mut app);
        }
    }
}

#[test]
fn single_phase_rejects_first_stop_and_its_normal_run_has_no_unit_controls() {
    use radishflow_studio::{
        UnitControlReply, flow_actions::FlowEditContext, run_actions::StudioRunAccess,
    };
    let mut app = flow_app();
    app.open_project(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/flowsheets/two-feed-single-vapor-sensible.rfproj.json"),
        "test",
    );
    let expected = FlowEditContext::capture(&app.platform_host.modeling_inputs().workspace);
    let reply = app
        .platform_host
        .background_run_action(Action::StartAtFirstUnit {
            expected,
            access: StudioRunAccess::Allowed,
        })
        .unwrap();
    assert_eq!(reply.unit_control, Some(UnitControlReply::Unsupported));
    assert!(!app.background_busy());
    let activity = start_flow(&mut app);
    assert!(activity.unit_control.is_none());
    drain(&mut app);
    assert!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .history()
            .back()
            .unwrap()
            .applied
    );
}
