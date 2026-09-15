mod studio_gui_preferences_store;
mod studio_gui_shell;

fn main() -> eframe::Result<()> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() == Some(std::ffi::OsStr::new("--headless")) {
        let code = radishflow_studio::headless::run_cli(
            &args.collect::<Vec<_>>(),
            &mut std::io::stdout().lock(),
            &mut std::io::stderr().lock(),
        );
        std::process::exit(code);
    }
    studio_gui_shell::run()
}
