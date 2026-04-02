use radishflow_studio::{StudioAppResultDispatch, StudioBootstrapConfig, run_studio_bootstrap};

fn main() {
    let config = StudioBootstrapConfig::default();

    match run_studio_bootstrap(&config) {
        Ok(report) => {
            println!("RadishFlow Studio bootstrap");
            println!("Project: {}", config.project_path.display());

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
                }
            }

            if let Some(summary) = report.latest_snapshot_summary {
                println!("Summary: {summary}");
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
