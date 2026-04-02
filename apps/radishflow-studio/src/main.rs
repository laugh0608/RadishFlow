use radishflow_studio::{StudioAppResultDispatch, StudioBootstrapConfig, run_studio_bootstrap};

fn main() {
    let config = StudioBootstrapConfig::default();

    match run_studio_bootstrap(&config) {
        Ok(report) => {
            println!("RadishFlow Studio bootstrap");
            println!("Project: {}", config.project_path.display());
            println!("Requested intent: {:?}", config.intent);
            println!("Control mode: {:?}", report.control_state.simulation_mode);
            println!("Control pending: {:?}", report.control_state.pending_reason);
            println!("Control status: {:?}", report.control_state.run_status);
            println!(
                "Control actions: run_manual={}, resume={}, set_hold={}, set_active={}",
                report.control_state.can_run_manual,
                report.control_state.can_resume,
                report.control_state.can_set_hold,
                report.control_state.can_set_active
            );

            match report.outcome.dispatch {
                StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                    println!("Run status: {:?}", dispatch.run_status);
                    println!("Dispatch: {:?}", dispatch.solve_dispatch);
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
