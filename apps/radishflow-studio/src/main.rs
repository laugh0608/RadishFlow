use radishflow_studio::{
    EntitlementSessionEventOutcome, StudioAppHostEntitlementTimerEffect, StudioAppHostState,
    StudioAppResultDispatch, StudioGuiDriver, StudioGuiDriverDispatch, StudioGuiDriverOutcome,
    StudioGuiEvent, StudioGuiHostCloseWindowResult, StudioGuiHostCommandOutcome,
    StudioGuiHostDispatch, StudioGuiHostLifecycleDispatch, StudioGuiHostWindowOpened,
    StudioGuiNativeTimerEffects, StudioGuiNativeTimerOperation, StudioRuntimeConfig,
    StudioRuntimeDispatch, StudioRuntimeReport, StudioWindowHostRetirement,
    StudioWindowTimerDriverAckResult,
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

fn main() {
    let config = StudioRuntimeConfig::default();
    let mut app_host = match StudioGuiDriver::new(&config) {
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

    let opened = expect_window_opened(&mut app_host, "open initial window");
    let window = opened.registration;
    println!(
        "Opened window host #{} as {:?}",
        window.window_id, window.role
    );
    println!(
        "Foreground window: {:?}",
        opened.projection.state.foreground_window_id
    );
    if let Some(slot) = window.restored_entitlement_timer.as_ref() {
        println!("Restored parked timer slot into window host: {:?}", slot);
    }
    if !window.timer_driver_commands.is_empty() {
        println!("Window host driver commands:");
        println!("  registration commands are now auto-applied by session adapter");
    }

    let dispatch = expect_window_dispatch(
        &mut app_host,
        window.window_id,
        config.trigger.clone(),
        "dispatch initial trigger",
    );
    let dispatch_state = dispatch.projection.state.clone();
    let effects = dispatch.effects;
    let report = effects.runtime_report;
    let entitlement_timer_effect = effects.entitlement_timer_effect;
    let native_timers = dispatch.native_timers;
    println!("RadishFlow Studio bootstrap");
    println!("Project: {}", config.project_path.display());
    println!("Requested trigger: {:?}", config.trigger);
    println!("Entitlement preflight: {:?}", config.entitlement_preflight);
    println!("App host state: {:?}", dispatch_state);
    print_ui_actions(&dispatch_state);
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

    if let Some(effect) = entitlement_timer_effect.as_ref() {
        println!("Runtime timer command:");
        print_entitlement_timer_effect(effect);
    }
    if !native_timers.operations.is_empty() {
        println!("Timer driver commands:");
        print_native_timer_effects(&native_timers);
    }

    match report.dispatch {
        StudioRuntimeDispatch::AppCommand(outcome) => match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                println!("Run status: {:?}", dispatch.run_status);
                println!("Outcome: {:?}", dispatch.outcome);
                if let Some(package_id) = dispatch.package_id {
                    println!("Package: {package_id}");
                }
                if let Some(snapshot_id) = dispatch.latest_snapshot_id {
                    println!("Latest snapshot: {snapshot_id}");
                }
                if let Some(summary) = dispatch.latest_snapshot_summary {
                    println!("Summary: {summary}");
                }
                println!("Log entries: {}", dispatch.log_entry_count);
                if let Some(entry) = dispatch.latest_log_entry {
                    println!("Latest log: {:?}: {}", entry.level, entry.message);
                }
            }
            StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                println!("Run status: {:?}", dispatch.run_status);
                if let Some(snapshot_id) = dispatch.latest_snapshot_id {
                    println!("Latest snapshot: {snapshot_id}");
                }
                if let Some(summary) = dispatch.latest_snapshot_summary {
                    println!("Summary: {summary}");
                }
                println!("Log entries: {}", dispatch.log_entry_count);
                if let Some(entry) = dispatch.latest_log_entry {
                    println!("Latest log: {:?}: {}", entry.level, entry.message);
                }
            }
            StudioAppResultDispatch::Entitlement(dispatch) => {
                println!("Entitlement status: {:?}", dispatch.entitlement_status);
                println!("Entitlement outcome: {:?}", dispatch.outcome);
                if let Some(notice) = dispatch.notice {
                    println!("Entitlement notice: {:?}: {}", notice.level, notice.message);
                }
                if let Some(entry) = dispatch.latest_log_entry {
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
            match outcome.outcome {
                EntitlementSessionEventOutcome::Tick(tick) => {
                    if let Some(preflight) = tick.preflight {
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
        for entry in report.log_entries {
            println!("  - {:?}: {}", entry.level, entry.message);
        }
    }

    match expect_lifecycle_event(
        &mut app_host,
        StudioGuiEvent::NetworkRestored,
        "dispatch lifecycle network restored",
    )
    .dispatch
    {
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

    let close = expect_close_window(&mut app_host, window.window_id, "close initial window");
    let close_state = close.projection.state.clone();
    match close.close {
        Some(shutdown) => {
            if let Some(slot) = shutdown.cleared_entitlement_timer {
                println!("Window host shutdown cleared timer slot: {:?}", slot);
            }
            if !shutdown.native_timer_transitions.is_empty() {
                println!("Window host shutdown driver commands:");
                print_native_timer_effects(&close.native_timers);
            }
            match shutdown.retirement {
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
            print_ui_actions(&close_state);
        }
        None => {
            println!("Window host close ignored for window #{}", window.window_id);
        }
    }
}

fn print_ui_actions(state: &StudioAppHostState) {
    let model = state.ui_command_model();
    if model.actions.is_empty() {
        println!("UI actions: none");
        return;
    }

    println!("UI actions:");
    for action in &model.actions {
        println!(
            "  - {} ({}) / {:?}#{} [{}] on {:?}",
            action.label,
            action.command_id,
            action.group,
            action.sort_order,
            if action.enabled {
                "enabled"
            } else {
                "disabled"
            },
            action.target_window_id
        );
        println!("    {}", action.detail);
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

fn expect_window_opened(app_host: &mut StudioGuiDriver, context: &str) -> StudioGuiHostWindowOpened {
    match app_host.dispatch_event(StudioGuiEvent::OpenWindowRequested) {
        Ok(StudioGuiDriverDispatch {
            outcome: StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::WindowOpened(result),
            ),
            ..
        }) => result,
        Ok(other) => {
            eprintln!(
                "RadishFlow Studio host command failed during {}: expected window open outcome, got {:?}",
                context, other
            );
            std::process::exit(1);
        }
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
    app_host: &mut StudioGuiDriver,
    window_id: u64,
    trigger: radishflow_studio::StudioRuntimeTrigger,
    context: &str,
) -> StudioGuiHostDispatch {
    match app_host.dispatch_event(StudioGuiEvent::WindowTriggerRequested { window_id, trigger }) {
        Ok(StudioGuiDriverDispatch {
            outcome: StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::WindowDispatched(result),
            ),
            ..
        }) => result,
        Ok(other) => {
            eprintln!(
                "RadishFlow Studio host command failed during {}: expected window dispatch outcome, got {:?}",
                context, other
            );
            std::process::exit(1);
        }
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
    app_host: &mut StudioGuiDriver,
    event: StudioGuiEvent,
    context: &str,
) -> StudioGuiHostLifecycleDispatch {
    match app_host.dispatch_event(event) {
        Ok(StudioGuiDriverDispatch {
            outcome: StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::LifecycleDispatched(result),
            ),
            ..
        }) => result,
        Ok(other) => {
            eprintln!(
                "RadishFlow Studio host command failed during {}: expected lifecycle outcome, got {:?}",
                context, other
            );
            std::process::exit(1);
        }
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
    app_host: &mut StudioGuiDriver,
    window_id: u64,
    context: &str,
) -> StudioGuiHostCloseWindowResult {
    match app_host.dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id }) {
        Ok(StudioGuiDriverDispatch {
            outcome: StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::WindowClosed(result),
            ),
            ..
        }) => result,
        Ok(other) => {
            eprintln!(
                "RadishFlow Studio host command failed during {}: expected close outcome, got {:?}",
                context, other
            );
            std::process::exit(1);
        }
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
