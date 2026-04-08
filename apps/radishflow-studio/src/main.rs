use radishflow_studio::{
    EntitlementSessionEventOutcome, StudioAppHostEntitlementTimerEffect, StudioAppResultDispatch,
    StudioGuiDriverOutcome, StudioGuiEvent, StudioGuiHostCommandOutcome,
    StudioGuiNativeTimerEffects, StudioGuiNativeTimerOperation, StudioGuiPlatformDispatch,
    StudioGuiPlatformHost, StudioGuiPlatformNativeTimerId, StudioGuiPlatformTimerCommand,
    StudioGuiPlatformTimerRequest, StudioGuiPlatformTimerStartAckResult, StudioGuiWindowAreaId,
    StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion, StudioGuiWindowDropTarget,
    StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutMutation, StudioGuiWindowModel,
    StudioRuntimeConfig, StudioRuntimeDispatch, StudioRuntimeReport, StudioWindowHostId,
    StudioWindowHostRetirement, StudioWindowTimerDriverAckResult,
};

fn print_text_view(title: &str, lines: &[String]) {
    println!("{title}:");
    for line in lines {
        println!("  {line}");
    }
}

fn print_run_panel(report: &StudioRuntimeReport) {
    let text = report.run_panel.text();
    print_text_view(text.title, &text.lines);
}

fn print_window_model(title: &str, window: &StudioGuiWindowModel) {
    let layout = window.layout();
    println!("{title}:");
    println!("  {}", layout.titlebar.title);
    println!("  {}", layout.titlebar.subtitle);
    println!(
        "  Titlebar: foreground={:?} windows={} close={}",
        layout.titlebar.foreground_window_id,
        layout.titlebar.registered_window_count,
        if layout.titlebar.close_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!(
        "  Layout: center={:?} default_focus={:?}",
        layout.center_area, layout.default_focus_area
    );
    println!(
        "  Layout state: scope={:?} window={:?} role={:?} slot={:?} key={}",
        layout.state.scope.kind,
        layout.state.scope.window_id,
        layout.state.scope.window_role,
        layout.state.scope.layout_slot,
        layout.state.scope.layout_key
    );
    println!("  Region weights:");
    for region in &layout.region_weights {
        println!("    - {:?}: {}", region.dock_region, region.weight);
    }
    println!("  Panels:");
    for panel in &layout.panels {
        println!(
            "    - {:?} @ {:?} group={} order={} mode={:?} active={} [{}] collapsed={} badge={} :: {}",
            panel.area_id,
            panel.dock_region,
            panel.stack_group,
            panel.order,
            panel.display_mode,
            panel.active_in_stack,
            if panel.visible { "visible" } else { "hidden" },
            panel.collapsed,
            panel.badge.as_deref().unwrap_or("none"),
            panel.summary
        );
    }
    println!("  Stack groups:");
    for stack_group in &layout.stack_groups {
        let tabs = stack_group
            .tabs
            .iter()
            .map(|tab| format!("{:?}{}", tab.area_id, if tab.active { "*" } else { "" }))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "    - {:?} group={} tabbed={} active={:?} tabs=[{}]",
            stack_group.dock_region,
            stack_group.stack_group,
            stack_group.tabbed,
            stack_group.active_area_id,
            tabs
        );
    }
    println!(
        "  Commands: {} total / {} enabled",
        window.commands.total_command_count, window.commands.enabled_command_count
    );
    for section in &window.commands.sections {
        println!("  {}:", section.title);
        for command in &section.commands {
            let shortcut = command
                .shortcut
                .as_ref()
                .map(format_shortcut)
                .unwrap_or_else(|| "none".to_string());
            println!(
                "    - {} ({}) [{}] shortcut={} target={:?}",
                command.label,
                command.command_id,
                if command.enabled {
                    "enabled"
                } else {
                    "disabled"
                },
                shortcut,
                command.target_window_id
            );
            println!("      {}", command.detail);
        }
    }

    print_text_view(
        window.canvas.widget.text().title,
        &window.canvas.widget.text().lines,
    );
    println!(
        "Canvas actions: {} enabled / {} total",
        window.canvas.enabled_action_count,
        window.canvas.widget.actions.len()
    );
    for action in &window.canvas.widget.actions {
        let shortcut = action
            .shortcut
            .as_ref()
            .map(format_shortcut)
            .unwrap_or_else(|| "none".to_string());
        println!(
            "  - {} [{}] shortcut={} :: {}",
            action.label,
            if action.enabled {
                "enabled"
            } else {
                "disabled"
            },
            shortcut,
            action.detail
        );
    }

    let run_panel_text = window.runtime.run_panel.text();
    print_text_view(run_panel_text.title, &run_panel_text.lines);
    if let Some(notice) = window.runtime.platform_notice.as_ref() {
        println!(
            "Platform notice: {:?}: {} :: {}",
            notice.level, notice.title, notice.message
        );
    }
    if let Some(entitlement_host) = window.runtime.entitlement_host.as_ref() {
        print_text_view(
            entitlement_host.presentation.panel.text.title,
            &entitlement_host.presentation.panel.text.lines,
        );
        print_text_view(
            entitlement_host.presentation.text.title,
            &entitlement_host.presentation.text.lines,
        );
    }
    if let Some(entry) = window.runtime.latest_log_entry.as_ref() {
        println!("Latest window log: {:?}: {}", entry.level, entry.message);
    }
    if let Some(preview) = window.drop_preview.as_ref() {
        println!(
            "  Drop preview: query={:?} kind={:?} region={:?} changed={:?}",
            preview.query,
            preview.drop_target.kind,
            preview.drop_target.dock_region,
            preview.changed_area_ids
        );
        println!(
            "  Drop overlay: stack={:?}/group={} group_index={} tab_index={} active={:?} tabs={:?} anchor={:?}",
            preview.overlay.target_dock_region,
            preview.overlay.target_stack_group,
            preview.overlay.target_group_index,
            preview.overlay.target_tab_index,
            preview.overlay.target_stack_active_area_id,
            preview.overlay.target_stack_area_ids,
            preview.overlay.anchor_area_id
        );
    }
}

fn print_drop_target_preview(
    title: &str,
    app_host: &mut StudioGuiPlatformHost,
    window_id: Option<StudioWindowHostId>,
    query: StudioGuiWindowDropTargetQuery,
) {
    println!("{title}:");
    match app_host
        .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested { window_id, query })
    {
        Ok(StudioGuiPlatformDispatch {
            outcome:
                StudioGuiDriverOutcome::HostCommand(
                    StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(result),
                ),
            window,
            ..
        }) => {
            match result.drop_target {
                Some(target) => print_drop_target(&target),
                None => println!("  none"),
            }
            if let Some(preview) = window.drop_preview.as_ref() {
                let preview_layout = &preview.preview_layout;
                println!(
                    "  preview center={:?} commands={:?} runtime={:?}",
                    preview_layout.center_area,
                    preview_layout
                        .panel(StudioGuiWindowAreaId::Commands)
                        .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
                    preview_layout
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                );
            }
        }
        Ok(dispatch) => println!("  unexpected outcome: {:?}", dispatch.outcome),
        Err(error) => println!(
            "  query failed [{}]: {}",
            error.code().as_str(),
            error.message()
        ),
    }
    let _ = app_host.dispatch_event(StudioGuiEvent::WindowDropTargetPreviewCleared { window_id });
}

fn print_drop_target(target: &StudioGuiWindowDropTarget) {
    println!(
        "  area={:?} kind={:?} region={:?} anchor={:?} placement={:?}",
        target.area_id, target.kind, target.dock_region, target.anchor_area_id, target.placement
    );
    println!(
        "  source=({:?}, group={}) -> target=(group={}, group_index={}, tab_index={})",
        target.source_dock_region,
        target.source_stack_group,
        target.target_stack_group,
        target.target_group_index,
        target.target_tab_index
    );
    println!(
        "  creates_new_stack={} merges_into_existing_stack={} active={:?} tabs={:?}",
        target.creates_new_stack,
        target.merges_into_existing_stack,
        target.preview_active_area_id,
        target.preview_area_ids
    );
}

fn main() {
    let config = StudioRuntimeConfig::default();
    let mut app_host = match StudioGuiPlatformHost::new(&config) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!(
                "RadishFlow Studio bootstrap failed [{}]: {}",
                error.code().as_str(),
                error.message()
            );
            std::process::exit(1);
        }
    };
    let mut next_platform_native_timer_id = 9001;

    let opened = expect_window_opened(&mut app_host, "open initial window");
    let opened_result = match &opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(result)) => {
            result
        }
        other => unreachable!("expected window opened outcome, got {other:?}"),
    };
    let window = &opened_result.registration;
    println!(
        "Opened window host #{} as {:?}",
        window.window_id, window.role
    );
    println!(
        "Foreground window: {:?}",
        opened_result.projection.state.foreground_window_id
    );
    print_window_model(
        "Window model after opening window",
        &app_host.snapshot().window_model(),
    );
    print_drop_target_preview(
        "Drop target preview for moving runtime to the start of the left sidebar",
        &mut app_host,
        Some(window.window_id),
        StudioGuiWindowDropTargetQuery::DockRegion {
            area_id: StudioGuiWindowAreaId::Runtime,
            dock_region: StudioGuiWindowDockRegion::LeftSidebar,
            placement: StudioGuiWindowDockPlacement::Start,
        },
    );
    let layout_update = app_host
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(window.window_id),
            mutation: StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                area_id: StudioGuiWindowAreaId::Commands,
                collapsed: true,
            },
        })
        .expect("expected window layout update");
    print_window_model(
        "Window model after collapsing commands panel",
        &layout_update.window,
    );
    let centered_runtime = app_host
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(window.window_id),
            mutation: StudioGuiWindowLayoutMutation::SetCenterArea {
                area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .expect("expected window center update");
    print_window_model(
        "Window model after centering runtime area",
        &centered_runtime.window,
    );
    let reordered_runtime = app_host
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(window.window_id),
            mutation: StudioGuiWindowLayoutMutation::SetPanelOrder {
                area_id: StudioGuiWindowAreaId::Runtime,
                order: 5,
            },
        })
        .expect("expected window panel order update");
    print_window_model(
        "Window model after moving runtime panel to the first order slot",
        &reordered_runtime.window,
    );
    let moved_commands = app_host
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(window.window_id),
            mutation: StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                area_id: StudioGuiWindowAreaId::Commands,
                dock_region: StudioGuiWindowDockRegion::RightSidebar,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            },
        })
        .expect("expected panel dock region update");
    print_window_model(
        "Window model after inserting commands panel before runtime in the right sidebar",
        &moved_commands.window,
    );
    print_drop_target_preview(
        "Drop target preview for stacking commands with runtime in the right sidebar",
        &mut app_host,
        Some(window.window_id),
        StudioGuiWindowDropTargetQuery::Stack {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        },
    );
    let stacked_commands = app_host
        .dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested {
            window_id: Some(window.window_id),
            query: StudioGuiWindowDropTargetQuery::Stack {
                area_id: StudioGuiWindowAreaId::Commands,
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            },
        })
        .expect("expected panel stack update");
    print_window_model(
        "Window model after stacking commands with runtime in the right sidebar",
        &stacked_commands.window,
    );
    print_drop_target_preview(
        "Drop target preview for moving runtime before commands inside the shared stack",
        &mut app_host,
        Some(window.window_id),
        StudioGuiWindowDropTargetQuery::CurrentStack {
            area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Commands,
            },
        },
    );
    let switched_active_tab = app_host
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(window.window_id),
            mutation: StudioGuiWindowLayoutMutation::SetActivePanelInStack {
                area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .expect("expected stack active tab update");
    print_window_model(
        "Window model after switching the active tab in the right sidebar stack",
        &switched_active_tab.window,
    );
    let cycled_previous_tab = app_host
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(window.window_id),
            mutation: StudioGuiWindowLayoutMutation::ActivatePreviousPanelInStack {
                area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .expect("expected stack previous-tab update");
    print_window_model(
        "Window model after cycling back to the previous active tab",
        &cycled_previous_tab.window,
    );
    let reordered_tabs = app_host
        .dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested {
            window_id: Some(window.window_id),
            query: StudioGuiWindowDropTargetQuery::CurrentStack {
                area_id: StudioGuiWindowAreaId::Runtime,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Commands,
                },
            },
        })
        .expect("expected stack reorder update");
    print_window_model(
        "Window model after reordering runtime before commands inside the shared stack",
        &reordered_tabs.window,
    );
    print_drop_target_preview(
        "Drop target preview for unstacking commands back into a standalone right-sidebar group",
        &mut app_host,
        Some(window.window_id),
        StudioGuiWindowDropTargetQuery::Unstack {
            area_id: StudioGuiWindowAreaId::Commands,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        },
    );
    let unstacked_commands = app_host
        .dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested {
            window_id: Some(window.window_id),
            query: StudioGuiWindowDropTargetQuery::Unstack {
                area_id: StudioGuiWindowAreaId::Commands,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            },
        })
        .expect("expected panel unstack update");
    print_window_model(
        "Window model after unstacking commands into its own right-sidebar group",
        &unstacked_commands.window,
    );
    if let Some(slot) = window.restored_entitlement_timer.as_ref() {
        println!("Restored parked timer slot into window host: {:?}", slot);
    }
    if !opened_result.native_timers.operations.is_empty() {
        println!("Window host native timer effects:");
        print_native_timer_effects(&opened_result.native_timers);
    }
    consume_platform_timer_request(
        &mut app_host,
        opened.native_timer_request.as_ref(),
        &mut next_platform_native_timer_id,
    );

    let dispatch = expect_window_dispatch(
        &mut app_host,
        window.window_id,
        config.trigger.clone(),
        "dispatch initial trigger",
    );
    let dispatch_result = match &dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowDispatched(
            result,
        )) => result,
        other => unreachable!("expected window dispatched outcome, got {other:?}"),
    };
    let dispatch_state = &dispatch_result.projection.state;
    let effects = &dispatch_result.effects;
    let report = &effects.runtime_report;
    let entitlement_timer_effect = effects.entitlement_timer_effect.as_ref();
    let native_timers = &dispatch_result.native_timers;
    println!("RadishFlow Studio bootstrap");
    println!("Project: {}", config.project_path.display());
    println!("Requested trigger: {:?}", config.trigger);
    println!("Entitlement preflight: {:?}", config.entitlement_preflight);
    println!("App host state: {:?}", dispatch_state);
    print_window_model(
        "Window model after initial dispatch",
        &app_host.snapshot().window_model(),
    );
    println!("Control mode: {:?}", report.control_state.simulation_mode);
    println!("Control pending: {:?}", report.control_state.pending_reason);
    println!("Control status: {:?}", report.control_state.run_status);
    print_run_panel(&report);
    let entitlement_host = &report.entitlement_host.presentation;
    print_text_view(
        entitlement_host.panel.text.title,
        &entitlement_host.panel.text.lines,
    );
    print_text_view(entitlement_host.text.title, &entitlement_host.text.lines);

    if let Some(preflight) = report.entitlement_preflight.as_ref() {
        println!("Preflight action: {:?}", preflight.decision.action);
        println!("Preflight reason: {}", preflight.decision.reason);
    }

    if let Some(effect) = entitlement_timer_effect {
        println!("Runtime timer command:");
        print_entitlement_timer_effect(effect);
    }
    if !native_timers.operations.is_empty() {
        println!("Timer driver commands:");
        print_native_timer_effects(&native_timers);
    }
    consume_platform_timer_request(
        &mut app_host,
        dispatch.native_timer_request.as_ref(),
        &mut next_platform_native_timer_id,
    );
    if let Some(binding) = app_host.current_platform_timer_binding().cloned() {
        println!(
            "Next platform timer due at: {:?} (native_id={} window={:?} handle={})",
            binding.schedule.slot.timer.due_at,
            binding.native_timer_id,
            binding.schedule.window_id,
            binding.schedule.handle_id
        );
        let callback_dispatch = app_host
            .dispatch_native_timer_elapsed_by_native_id(binding.native_timer_id)
            .expect("expected native timer callback dispatch");
        println!(
            "Simulated platform native timer callback via native_id={}",
            binding.native_timer_id
        );
        println!("  - due callback outcome: {:?}", callback_dispatch.outcome);
        consume_platform_timer_request(
            &mut app_host,
            callback_dispatch.native_timer_request.as_ref(),
            &mut next_platform_native_timer_id,
        );
        print_window_model(
            "Window model after simulated native timer callback",
            &app_host.snapshot().window_model(),
        );
    }

    match &report.dispatch {
        StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                println!("Run status: {:?}", dispatch.run_status);
                println!("Outcome: {:?}", dispatch.outcome);
                if let Some(package_id) = &dispatch.package_id {
                    println!("Package: {package_id}");
                }
                if let Some(snapshot_id) = &dispatch.latest_snapshot_id {
                    println!("Latest snapshot: {snapshot_id}");
                }
                if let Some(summary) = &dispatch.latest_snapshot_summary {
                    println!("Summary: {summary}");
                }
                println!("Log entries: {}", dispatch.log_entry_count);
                if let Some(entry) = &dispatch.latest_log_entry {
                    println!("Latest log: {:?}: {}", entry.level, entry.message);
                }
            }
            StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                println!("Run status: {:?}", dispatch.run_status);
                if let Some(snapshot_id) = &dispatch.latest_snapshot_id {
                    println!("Latest snapshot: {snapshot_id}");
                }
                if let Some(summary) = &dispatch.latest_snapshot_summary {
                    println!("Summary: {summary}");
                }
                println!("Log entries: {}", dispatch.log_entry_count);
                if let Some(entry) = &dispatch.latest_log_entry {
                    println!("Latest log: {:?}: {}", entry.level, entry.message);
                }
            }
            StudioAppResultDispatch::Entitlement(dispatch) => {
                println!("Entitlement status: {:?}", dispatch.entitlement_status);
                println!("Entitlement outcome: {:?}", dispatch.outcome);
                if let Some(notice) = &dispatch.notice {
                    println!("Entitlement notice: {:?}: {}", notice.level, notice.message);
                }
                if let Some(entry) = &dispatch.latest_log_entry {
                    println!("Latest log: {:?}: {}", entry.level, entry.message);
                }
            }
        },
        StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
            println!("Run panel recovery: {}", outcome.action.title);
            println!("Recovery detail: {}", outcome.action.detail);
            println!("Applied target: {:?}", outcome.applied_target);
        }
        StudioRuntimeDispatch::EntitlementSessionEvent(outcome) => {
            println!("Entitlement session event: {:?}", outcome.event);
            match &outcome.outcome {
                EntitlementSessionEventOutcome::Tick(tick) => {
                    if let Some(preflight) = tick.preflight.as_ref() {
                        println!("Session action: {:?}", preflight.decision.action);
                        println!("Session reason: {}", preflight.decision.reason);
                    } else {
                        println!("Session action: None");
                    }
                }
                EntitlementSessionEventOutcome::RecordedCommand { action } => {
                    println!("Session recorded command: {:?}", action);
                }
            }
        }
    }

    if !report.log_entries.is_empty() {
        println!("Logs:");
        for entry in &report.log_entries {
            println!("  - {:?}: {}", entry.level, entry.message);
        }
    }

    let lifecycle = expect_lifecycle_event(
        &mut app_host,
        StudioGuiEvent::NetworkRestored,
        "dispatch lifecycle network restored",
    );
    let lifecycle_result = match &lifecycle.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
            result,
        )) => result,
        other => unreachable!("expected lifecycle outcome, got {other:?}"),
    };
    match &lifecycle_result.dispatch {
        Some(global_dispatch) => {
            println!(
                "Global network restored routed to window #{}",
                global_dispatch.target_window_id
            );
        }
        None => {
            println!(
                "Global event ignored: {:?}",
                StudioGuiEvent::NetworkRestored
            );
        }
    }
    print_window_model(
        "Window model after network restored",
        &app_host.snapshot().window_model(),
    );
    consume_platform_timer_request(
        &mut app_host,
        lifecycle.native_timer_request.as_ref(),
        &mut next_platform_native_timer_id,
    );

    let close = expect_close_window(&mut app_host, window.window_id, "close initial window");
    let close_result = match &close.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(result)) => {
            result
        }
        other => unreachable!("expected close outcome, got {other:?}"),
    };
    let close_state = &close_result.projection.state;
    match &close_result.close {
        Some(shutdown) => {
            if let Some(slot) = &shutdown.cleared_entitlement_timer {
                println!("Window host shutdown cleared timer slot: {:?}", slot);
            }
            if !shutdown.native_timer_transitions.is_empty() {
                println!("Window host shutdown driver commands:");
                print_native_timer_effects(&close_result.native_timers);
            }
            match &shutdown.retirement {
                StudioWindowHostRetirement::None => {}
                StudioWindowHostRetirement::Transferred {
                    new_owner_window_id,
                    restored_entitlement_timer,
                } => {
                    println!("Timer ownership transferred to window #{new_owner_window_id}");
                    if let Some(slot) = restored_entitlement_timer {
                        println!("Transferred timer slot: {:?}", slot);
                    }
                }
                StudioWindowHostRetirement::Parked {
                    parked_entitlement_timer,
                } => {
                    println!("Timer ownership parked after last window closed");
                    if let Some(slot) = parked_entitlement_timer {
                        println!("Parked timer slot: {:?}", slot);
                    }
                }
            }
            println!(
                "Next foreground window: {:?}",
                shutdown.next_foreground_window_id
            );
            println!("App host state: {:?}", close_state);
            print_window_model(
                "Window model after closing window",
                &app_host.snapshot().window_model(),
            );
            consume_platform_timer_request(
                &mut app_host,
                close.native_timer_request.as_ref(),
                &mut next_platform_native_timer_id,
            );
        }
        None => {
            println!("Window host close ignored for window #{}", window.window_id);
        }
    }
}

fn print_entitlement_timer_effect(effect: &StudioAppHostEntitlementTimerEffect) {
    match effect {
        StudioAppHostEntitlementTimerEffect::Keep {
            owner_window_id,
            effect_id,
            slot,
            follow_up_trigger,
            ack,
        } => {
            println!("  owner window: #{owner_window_id}");
            println!("  - #{} Keep {:?}", effect_id, slot.timer);
            println!("    follow-up trigger: {:?}", follow_up_trigger);
            println!("Timer host slot: {:?}", slot);
            println!("Timer host ack: {:?}", ack);
        }
        StudioAppHostEntitlementTimerEffect::Arm {
            owner_window_id,
            effect_id,
            slot,
            follow_up_trigger,
            ack,
        } => {
            println!("  owner window: #{owner_window_id}");
            println!("  - #{} Arm {:?}", effect_id, slot.timer);
            println!("    follow-up trigger: {:?}", follow_up_trigger);
            println!("Timer host slot: {:?}", slot);
            println!("Timer host ack: {:?}", ack);
        }
        StudioAppHostEntitlementTimerEffect::Rearm {
            owner_window_id,
            effect_id,
            previous_slot,
            next_slot,
            follow_up_trigger,
            ack,
        } => {
            println!("  owner window: #{owner_window_id}");
            println!(
                "  - #{} Rearm {:?} -> {:?}",
                effect_id, previous_slot, next_slot.timer
            );
            println!("    follow-up trigger: {:?}", follow_up_trigger);
            println!("Timer host slot: {:?}", next_slot);
            println!("Timer host ack: {:?}", ack);
        }
        StudioAppHostEntitlementTimerEffect::Clear {
            owner_window_id,
            effect_id,
            previous_slot,
            follow_up_trigger,
            ack,
        } => {
            println!("  owner window: #{owner_window_id}");
            println!("  - #{} Clear {:?}", effect_id, previous_slot);
            println!("    follow-up trigger: {:?}", follow_up_trigger);
            println!("Timer host cleared: {:?}", previous_slot);
            println!("Timer host ack: {:?}", ack);
        }
        StudioAppHostEntitlementTimerEffect::IgnoreStale {
            owner_window_id,
            stale_effect_id,
            current_slot,
            ack,
        } => {
            println!("  owner window: #{owner_window_id}");
            println!("  - Ignore stale effect #{}", stale_effect_id);
            println!("Timer host current: {:?}", current_slot);
            println!("Timer host ack: {:?}", ack);
        }
    }
}

fn expect_window_opened(
    app_host: &mut StudioGuiPlatformHost,
    context: &str,
) -> StudioGuiPlatformDispatch {
    match app_host.dispatch_event(StudioGuiEvent::OpenWindowRequested) {
        Ok(dispatch) => match &dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(_)) => {
                dispatch
            }
            _ => {
                eprintln!(
                    "RadishFlow Studio host command failed during {}: expected window open outcome, got {:?}",
                    context, dispatch
                );
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!(
                "RadishFlow Studio host command failed during {} [{}]: {}",
                context,
                error.code().as_str(),
                error.message()
            );
            std::process::exit(1);
        }
    }
}

fn expect_window_dispatch(
    app_host: &mut StudioGuiPlatformHost,
    window_id: u64,
    trigger: radishflow_studio::StudioRuntimeTrigger,
    context: &str,
) -> StudioGuiPlatformDispatch {
    match app_host.dispatch_event(StudioGuiEvent::WindowTriggerRequested { window_id, trigger }) {
        Ok(dispatch) => match &dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowDispatched(
                _,
            )) => dispatch,
            _ => {
                eprintln!(
                    "RadishFlow Studio host command failed during {}: expected window dispatch outcome, got {:?}",
                    context, dispatch
                );
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!(
                "RadishFlow Studio host command failed during {} [{}]: {}",
                context,
                error.code().as_str(),
                error.message()
            );
            std::process::exit(1);
        }
    }
}

fn expect_lifecycle_event(
    app_host: &mut StudioGuiPlatformHost,
    event: StudioGuiEvent,
    context: &str,
) -> StudioGuiPlatformDispatch {
    match app_host.dispatch_event(event) {
        Ok(dispatch) => match &dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::LifecycleDispatched(_),
            ) => dispatch,
            _ => {
                eprintln!(
                    "RadishFlow Studio host command failed during {}: expected lifecycle outcome, got {:?}",
                    context, dispatch
                );
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!(
                "RadishFlow Studio host command failed during {} [{}]: {}",
                context,
                error.code().as_str(),
                error.message()
            );
            std::process::exit(1);
        }
    }
}

fn expect_close_window(
    app_host: &mut StudioGuiPlatformHost,
    window_id: u64,
    context: &str,
) -> StudioGuiPlatformDispatch {
    match app_host.dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id }) {
        Ok(dispatch) => match &dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(_)) => {
                dispatch
            }
            _ => {
                eprintln!(
                    "RadishFlow Studio host command failed during {}: expected close outcome, got {:?}",
                    context, dispatch
                );
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!(
                "RadishFlow Studio host command failed during {} [{}]: {}",
                context,
                error.code().as_str(),
                error.message()
            );
            std::process::exit(1);
        }
    }
}

fn print_platform_timer_request(request: Option<&StudioGuiPlatformTimerRequest>) {
    match request {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => {
            println!(
                "Platform timer request: arm window={:?} handle={} due_at={:?}",
                schedule.window_id, schedule.handle_id, schedule.slot.timer.due_at
            );
        }
        Some(StudioGuiPlatformTimerRequest::Rearm { previous, schedule }) => {
            println!(
                "Platform timer request: rearm window={:?} handle={} due_at={:?} -> window={:?} handle={} due_at={:?}",
                previous.window_id,
                previous.handle_id,
                previous.slot.timer.due_at,
                schedule.window_id,
                schedule.handle_id,
                schedule.slot.timer.due_at
            );
        }
        Some(StudioGuiPlatformTimerRequest::Clear { previous }) => {
            println!(
                "Platform timer request: clear window={:?} handle={} due_at={:?}",
                previous.window_id, previous.handle_id, previous.slot.timer.due_at
            );
        }
        None => {}
    }
}

fn consume_platform_timer_request(
    host: &mut StudioGuiPlatformHost,
    request: Option<&StudioGuiPlatformTimerRequest>,
    next_native_timer_id: &mut StudioGuiPlatformNativeTimerId,
) {
    print_platform_timer_request(request);
    let Some(command) = host.apply_platform_timer_request(request) else {
        return;
    };

    print_platform_timer_command(&command);
    match &command {
        StudioGuiPlatformTimerCommand::Arm { schedule }
        | StudioGuiPlatformTimerCommand::Rearm { schedule, .. } => {
            let native_timer_id = *next_native_timer_id;
            *next_native_timer_id += 1;
            let ack = host.acknowledge_platform_timer_started(schedule, native_timer_id);
            print_platform_timer_start_ack(&ack);
        }
        StudioGuiPlatformTimerCommand::Clear { .. } => {}
    }
}

fn print_platform_timer_command(command: &StudioGuiPlatformTimerCommand) {
    match command {
        StudioGuiPlatformTimerCommand::Arm { schedule } => {
            println!(
                "Platform timer command: arm native timer window={:?} handle={} due_at={:?}",
                schedule.window_id, schedule.handle_id, schedule.slot.timer.due_at
            );
        }
        StudioGuiPlatformTimerCommand::Rearm { previous, schedule } => {
            println!(
                "Platform timer command: rearm native timer window={:?} handle={} due_at={:?}",
                schedule.window_id, schedule.handle_id, schedule.slot.timer.due_at
            );
            if let Some(previous) = previous {
                println!(
                    "  replacing native_id={} window={:?} handle={} due_at={:?}",
                    previous.native_timer_id,
                    previous.schedule.window_id,
                    previous.schedule.handle_id,
                    previous.schedule.slot.timer.due_at
                );
            } else {
                println!("  previous native timer binding missing");
            }
        }
        StudioGuiPlatformTimerCommand::Clear { previous } => match previous {
            Some(previous) => {
                println!(
                    "Platform timer command: clear native_id={} window={:?} handle={} due_at={:?}",
                    previous.native_timer_id,
                    previous.schedule.window_id,
                    previous.schedule.handle_id,
                    previous.schedule.slot.timer.due_at
                );
            }
            None => {
                println!(
                    "Platform timer command: clear requested but previous native timer binding missing"
                );
            }
        },
    }
}

fn print_platform_timer_start_ack(ack: &StudioGuiPlatformTimerStartAckResult) {
    println!(
        "Platform timer ack: native_id={} status={:?} window={:?} handle={} due_at={:?}",
        ack.native_timer_id,
        ack.status,
        ack.schedule.window_id,
        ack.schedule.handle_id,
        ack.schedule.slot.timer.due_at
    );
}

fn print_native_timer_effects(effects: &StudioGuiNativeTimerEffects) {
    for operation in &effects.operations {
        print_timer_driver_transition(operation);
    }
    for ack in &effects.acks {
        print_timer_driver_ack(ack);
    }
}

fn print_timer_driver_transition(transition: &StudioGuiNativeTimerOperation) {
    match transition {
        StudioGuiNativeTimerOperation::Arm {
            window_id,
            previous_binding,
            slot,
        } => {
            println!(
                "  - Arm native timer on window #{}: {:?} -> {:?}",
                window_id, previous_binding, slot
            );
        }
        StudioGuiNativeTimerOperation::Keep { window_id, binding } => {
            println!(
                "  - Keep native timer handle #{} on window #{} for {:?}",
                binding.handle_id, window_id, binding.slot
            );
        }
        StudioGuiNativeTimerOperation::Rearm {
            window_id,
            previous_binding,
            next_slot,
        } => {
            println!(
                "  - Rearm native timer on window #{}: {:?} -> {:?}",
                window_id, previous_binding, next_slot
            );
        }
        StudioGuiNativeTimerOperation::Clear {
            window_id,
            previous_binding,
        } => {
            println!(
                "  - Clear native timer on window #{} from {:?}",
                window_id, previous_binding
            );
        }
        StudioGuiNativeTimerOperation::IgnoreStale {
            window_id,
            current_binding,
            stale_effect_id,
        } => {
            println!(
                "  - Ignore stale timer effect #{} on window #{} with {:?}",
                stale_effect_id, window_id, current_binding
            );
        }
        StudioGuiNativeTimerOperation::Transfer {
            from_window_id,
            to_window_id,
            binding,
            requested_slot,
        } => {
            println!(
                "  - Transfer native timer {:?} from window #{} to #{} for {:?}",
                binding, from_window_id, to_window_id, requested_slot
            );
        }
        StudioGuiNativeTimerOperation::Park {
            from_window_id,
            binding,
            requested_slot,
        } => {
            println!(
                "  - Park native timer {:?} after closing window #{} for {:?}",
                binding, from_window_id, requested_slot
            );
        }
        StudioGuiNativeTimerOperation::RestoreParked {
            window_id,
            binding,
            requested_slot,
        } => {
            println!(
                "  - Restore parked native timer {:?} into window #{} for {:?}",
                binding, window_id, requested_slot
            );
        }
    }
}

fn print_timer_driver_ack(ack: &StudioWindowTimerDriverAckResult) {
    println!("  native timer ack: {:?}", ack);
}

fn format_shortcut(shortcut: &radishflow_studio::StudioGuiShortcut) -> String {
    let mut parts = shortcut
        .modifiers
        .iter()
        .map(|modifier| match modifier {
            radishflow_studio::StudioGuiShortcutModifier::Ctrl => "Ctrl",
            radishflow_studio::StudioGuiShortcutModifier::Shift => "Shift",
            radishflow_studio::StudioGuiShortcutModifier::Alt => "Alt",
        })
        .collect::<Vec<_>>();
    let key = match shortcut.key {
        radishflow_studio::StudioGuiShortcutKey::F5 => "F5",
        radishflow_studio::StudioGuiShortcutKey::F6 => "F6",
        radishflow_studio::StudioGuiShortcutKey::F8 => "F8",
        radishflow_studio::StudioGuiShortcutKey::Tab => "Tab",
        radishflow_studio::StudioGuiShortcutKey::Escape => "Escape",
    };
    parts.push(key);
    parts.join("+")
}
