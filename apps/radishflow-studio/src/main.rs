use radishflow_studio::{
    EntitlementSessionEventOutcome, StudioAppResultDispatch, StudioRuntime, StudioRuntimeConfig,
    StudioRuntimeDispatch, StudioRuntimeEffect, StudioRuntimeReport,
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

    let mut runtime = match StudioRuntime::new(&config) {
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

    match runtime.dispatch_trigger_output(&config.trigger) {
        Ok(output) => {
            let report = output.report;
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

            if !output.effects.is_empty() {
                println!("Runtime effects:");
                for effect in &output.effects {
                    match effect {
                        StudioRuntimeEffect::EntitlementTimer(effect) => {
                            println!("  - Entitlement timer: {:?}", effect);
                        }
                    }
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
