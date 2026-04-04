use radishflow_studio::{
    EntitlementSessionEventOutcome, StudioAppResultDispatch, StudioBootstrapConfig,
    StudioBootstrapDispatch, run_studio_bootstrap,
};

fn print_run_panel(report: &radishflow_studio::StudioBootstrapReport) {
    let text = report.run_panel.text();
    println!("{}:", text.title);
    for line in &text.lines {
        println!("  {line}");
    }
}

fn print_entitlement_panel(report: &radishflow_studio::StudioBootstrapReport) {
    let text = report.entitlement_panel.text();
    println!("{}:", text.title);
    for line in &text.lines {
        println!("  {line}");
    }
}

fn main() {
    let config = StudioBootstrapConfig::default();

    match run_studio_bootstrap(&config) {
        Ok(report) => {
            println!("RadishFlow Studio bootstrap");
            println!("Project: {}", config.project_path.display());
            println!("Requested trigger: {:?}", config.trigger);
            println!("Entitlement preflight: {:?}", config.entitlement_preflight);
            println!("Control mode: {:?}", report.control_state.simulation_mode);
            println!("Control pending: {:?}", report.control_state.pending_reason);
            println!("Control status: {:?}", report.control_state.run_status);
            print_run_panel(&report);
            print_entitlement_panel(&report);

            if let Some(preflight) = report.entitlement_preflight.as_ref() {
                println!("Preflight action: {:?}", preflight.decision.action);
                println!("Preflight reason: {}", preflight.decision.reason);
            }
            println!(
                "Entitlement next check: {:?}",
                report.entitlement_session_schedule.next_check_at
            );
            println!(
                "Entitlement next sync window: {:?}",
                report.entitlement_session_schedule.next_sync_at
            );
            println!(
                "Entitlement next offline refresh window: {:?}",
                report.entitlement_session_schedule.next_offline_refresh_at
            );
            if let Some(action) = report.entitlement_session_schedule.recommended_action {
                println!("Entitlement recommended action: {:?}", action);
            }
            if let Some(reason) = report.entitlement_session_schedule.recommended_reason {
                println!("Entitlement schedule reason: {reason}");
            }
            if report.entitlement_session_schedule.blocked_by_backoff {
                println!("Entitlement scheduler is currently backing off");
            }

            match report.dispatch {
                StudioBootstrapDispatch::AppCommand(outcome) => match outcome.dispatch {
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
                StudioBootstrapDispatch::EntitlementSessionEvent(outcome) => {
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
