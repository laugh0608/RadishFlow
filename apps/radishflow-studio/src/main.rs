use radishflow_studio::{StudioAppResultDispatch, StudioBootstrapConfig, run_studio_bootstrap};

fn print_run_panel(report: &radishflow_studio::StudioBootstrapReport) {
    let text = report.run_panel.text();
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
            println!("Control mode: {:?}", report.control_state.simulation_mode);
            println!("Control pending: {:?}", report.control_state.pending_reason);
            println!("Control status: {:?}", report.control_state.run_status);
            print_run_panel(&report);

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
