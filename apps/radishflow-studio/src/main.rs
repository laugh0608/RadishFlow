use radishflow_studio::{
    EntitlementSessionEventOutcome, StudioAppResultDispatch, StudioRuntimeConfig,
    StudioRuntimeDispatch, StudioRuntimeReport, StudioRuntimeTimerHostCommand,
    StudioRuntimeTimerHostTransition, StudioWindowHostEvent, StudioWindowHostRetirement,
    StudioWindowSession, StudioWindowTimerDriverAckResult, StudioWindowTimerDriverTransition,
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
    let mut window_session = match StudioWindowSession::new(&config) {
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
    let window = window_session.open_window();
    println!(
        "Opened window host #{} as {:?}",
        window.window_id, window.role
    );
    if let Some(slot) = window.restored_entitlement_timer.as_ref() {
        println!("Restored parked timer slot into window host: {:?}", slot);
    }
    if !window.timer_driver_commands.is_empty() {
        println!("Window host driver commands:");
        println!("  registration commands are now auto-applied by session adapter");
    }

    match window_session.dispatch_trigger(window.window_id, &config.trigger) {
        Ok(dispatch) => {
            let window_event = dispatch.host_output.window_event;
            let timer_driver_commands = dispatch.host_output.timer_driver_commands;
            let report = dispatch.host_output.runtime_output.report;
            println!("RadishFlow Studio bootstrap");
            println!("Project: {}", config.project_path.display());
            println!("Requested trigger: {:?}", config.trigger);
            println!("Entitlement preflight: {:?}", config.entitlement_preflight);
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

            if let Some(event) = window_event {
                println!("Runtime timer command:");
                match event {
                    StudioWindowHostEvent::EntitlementTimerApplied {
                        window_id,
                        command,
                        transition,
                        ack,
                    } => {
                        println!("  owner window: #{window_id}");
                        match &command {
                            StudioRuntimeTimerHostCommand::KeepTimer {
                                effect_id,
                                timer,
                                follow_up_trigger,
                            } => {
                                println!("  - #{} Keep {:?}", effect_id, timer);
                                println!("    follow-up trigger: {:?}", follow_up_trigger);
                            }
                            StudioRuntimeTimerHostCommand::ArmTimer {
                                effect_id,
                                timer,
                                follow_up_trigger,
                            } => {
                                println!("  - #{} Arm {:?}", effect_id, timer);
                                println!("    follow-up trigger: {:?}", follow_up_trigger);
                            }
                            StudioRuntimeTimerHostCommand::RearmTimer {
                                effect_id,
                                previous,
                                next,
                                follow_up_trigger,
                            } => {
                                println!("  - #{} Rearm {:?} -> {:?}", effect_id, previous, next);
                                println!("    follow-up trigger: {:?}", follow_up_trigger);
                            }
                            StudioRuntimeTimerHostCommand::ClearTimer {
                                effect_id,
                                previous,
                                follow_up_trigger,
                            } => {
                                println!("  - #{} Clear {:?}", effect_id, previous);
                                println!("    follow-up trigger: {:?}", follow_up_trigger);
                            }
                        }
                        println!("Timer host transition: {:?}", transition);
                        println!("Timer host ack: {:?}", ack);
                        match &transition {
                            StudioRuntimeTimerHostTransition::KeepTimer { slot, .. }
                            | StudioRuntimeTimerHostTransition::ArmTimer { slot, .. } => {
                                println!("Timer host slot: {:?}", slot);
                            }
                            StudioRuntimeTimerHostTransition::RearmTimer { next, .. } => {
                                println!("Timer host slot: {:?}", next);
                            }
                            StudioRuntimeTimerHostTransition::ClearTimer { previous, .. } => {
                                println!("Timer host cleared: {:?}", previous);
                            }
                            StudioRuntimeTimerHostTransition::IgnoreStale { current, .. } => {
                                println!("Timer host current: {:?}", current);
                            }
                        }
                    }
                }
            }
            if !timer_driver_commands.is_empty() {
                println!("Timer driver commands:");
                for command in &timer_driver_commands {
                    println!("  host command: {:?}", command);
                }
                for transition in &dispatch.timer_driver_transitions {
                    print_timer_driver_transition(transition);
                }
                for ack in &dispatch.timer_driver_acks {
                    print_timer_driver_ack(ack);
                }
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

            if let Some(shutdown) = window_session.close_window(window.window_id) {
                if let Some(slot) = shutdown.host_shutdown.cleared_entitlement_timer {
                    println!("Window host shutdown cleared timer slot: {:?}", slot);
                }
                if !shutdown.host_shutdown.timer_driver_commands.is_empty() {
                    println!("Window host shutdown driver commands:");
                    for command in &shutdown.host_shutdown.timer_driver_commands {
                        println!("  host command: {:?}", command);
                    }
                    for transition in &shutdown.timer_driver_transitions {
                        print_timer_driver_transition(transition);
                    }
                    for ack in &shutdown.timer_driver_acks {
                        print_timer_driver_ack(ack);
                    }
                }
                match shutdown.host_shutdown.retirement {
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
            }
        }
        Err(error) => {
            eprintln!(
                "RadishFlow Studio bootstrap failed [{}]: {}",
                error.code().as_str(),
                error.message()
            );
            std::process::exit(1);
        }
    }
}

fn print_timer_driver_transition(transition: &StudioWindowTimerDriverTransition) {
    match transition {
        StudioWindowTimerDriverTransition::ArmNativeTimer {
            window_id,
            previous_binding,
            slot,
        } => {
            println!(
                "  - Arm native timer on window #{}: {:?} -> {:?}",
                window_id, previous_binding, slot
            );
        }
        StudioWindowTimerDriverTransition::KeepNativeTimer { window_id, binding } => {
            println!(
                "  - Keep native timer handle #{} on window #{} for {:?}",
                binding.handle_id, window_id, binding.slot
            );
        }
        StudioWindowTimerDriverTransition::RearmNativeTimer {
            window_id,
            previous_binding,
            next_slot,
        } => {
            println!(
                "  - Rearm native timer on window #{}: {:?} -> {:?}",
                window_id, previous_binding, next_slot
            );
        }
        StudioWindowTimerDriverTransition::ClearNativeTimer {
            window_id,
            previous_binding,
        } => {
            println!(
                "  - Clear native timer on window #{} from {:?}",
                window_id, previous_binding
            );
        }
        StudioWindowTimerDriverTransition::IgnoreStale {
            window_id,
            current_binding,
            stale_effect_id,
        } => {
            println!(
                "  - Ignore stale timer effect #{} on window #{} with {:?}",
                stale_effect_id, window_id, current_binding
            );
        }
        StudioWindowTimerDriverTransition::TransferNativeTimer {
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
        StudioWindowTimerDriverTransition::ParkNativeTimer {
            from_window_id,
            binding,
            requested_slot,
        } => {
            println!(
                "  - Park native timer {:?} after closing window #{} for {:?}",
                binding, from_window_id, requested_slot
            );
        }
        StudioWindowTimerDriverTransition::RestoreParkedTimer {
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
