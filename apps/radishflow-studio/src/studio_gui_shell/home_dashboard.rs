use std::path::Path;

use super::*;

const HOME_START_WIDTH: f32 = 220.0;
const HOME_ENVIRONMENT_WIDTH: f32 = 280.0;

impl ReadyAppState {
    pub(super) fn render_home_dashboard(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
    ) {
        self.render_home_app_bar(ctx, window);
        self.render_home_messages(ctx, window);
        egui::SidePanel::left("studio.home_start_actions")
            .default_width(HOME_START_WIDTH)
            .min_width(HOME_START_WIDTH)
            .max_width(HOME_START_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                self.render_home_start_actions(ui, window);
            });
        egui::SidePanel::right("studio.home_environment")
            .default_width(HOME_ENVIRONMENT_WIDTH)
            .min_width(HOME_ENVIRONMENT_WIDTH)
            .max_width(HOME_ENVIRONMENT_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .id_salt("studio.home_environment_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.render_home_environment(ui, window));
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .id_salt("studio.home_cases_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| self.render_home_cases(ui, window));
        });
    }

    fn render_home_app_bar(&mut self, ctx: &egui::Context, window: &StudioGuiWindowModel) {
        egui::TopBottomPanel::top("studio.home_app_bar")
            .exact_height(56.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.heading("RadishFlow Studio");
                    ui.small("Steady-State Process Simulation");
                    ui.separator();
                    render_status_chip(
                        ui,
                        "v26.5.1-dev internal",
                        egui::Color32::from_rgb(86, 118, 168),
                    );
                    render_status_chip(ui, "Local ready", egui::Color32::from_rgb(52, 128, 89));
                    render_status_chip(
                        ui,
                        "Server offline",
                        egui::Color32::from_rgb(180, 70, 60),
                    );
                    render_status_chip(
                        ui,
                        "Signed out",
                        egui::Color32::from_rgb(120, 120, 120),
                    );
                    if window.runtime.workspace_document.has_unsaved_changes {
                        render_status_chip(
                            ui,
                            self.locale.text(ShellText::Unsaved),
                            egui::Color32::from_rgb(160, 120, 40),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.locale.text(ShellText::ViewOptions)).clicked() {
                            self.command_palette.toggle();
                        }
                        if ui.button("Sign in").clicked() {
                            self.project_open.notice = Some(ProjectOpenNotice {
                                level: ProjectOpenNoticeLevel::Info,
                                title: "Sign in unavailable".to_string(),
                                detail: "OIDC / PKCE browser sign-in is not attached to this internal build yet."
                                    .to_string(),
                            });
                        }
                        ui.small(self.locale.text(ShellText::UnitsSi));
                    });
                });
            });
    }

    fn render_home_start_actions(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.heading("Start");
        ui.add_space(8.0);

        if let Some(last_case) = self.project_open.recent_projects.first().cloned() {
            let response = ui
                .add(
                    egui::Button::new("Continue Last Case")
                        .fill(egui::Color32::from_rgb(230, 239, 252))
                        .min_size(egui::vec2(ui.available_width(), 44.0)),
                )
                .on_hover_text(last_case.display().to_string());
            if response.clicked() {
                self.open_recent_project(last_case);
            }
        } else {
            let response = ui
                .add(
                    egui::Button::new("Open Example Case")
                        .fill(egui::Color32::from_rgb(230, 239, 252))
                        .min_size(egui::vec2(ui.available_width(), 44.0)),
                )
                .on_hover_text("Open the first bundled example case.");
            if response.clicked() {
                if let Some(example) = window.runtime.example_projects.first() {
                    self.open_example_project(example.project_path.clone());
                }
            }
        }

        ui.add_space(8.0);
        if ui
            .add(
                egui::Button::new("New Blank Case")
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            self.create_blank_project();
        }
        ui.add_space(5.0);
        if ui
            .add(egui::Button::new("Open Case").min_size(egui::vec2(ui.available_width(), 36.0)))
            .clicked()
        {
            self.open_project_from_picker();
        }
        ui.add_space(5.0);
        if ui
            .add(
                egui::Button::new("Open Example Case")
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            if let Some(example) = window.runtime.example_projects.first() {
                self.open_example_project(example.project_path.clone());
            }
        }
    }

    fn render_home_cases(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.horizontal(|ui| {
            ui.heading("Recent Cases");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Open Case").clicked() {
                    self.open_project_from_picker();
                }
            });
        });
        self.render_home_recent_cases(ui);
        ui.add_space(18.0);
        ui.horizontal(|ui| {
            ui.heading("Example Cases");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.small("examples/flowsheets");
            });
        });
        self.render_home_example_cases(ui, window);
    }

    fn render_home_recent_cases(&mut self, ui: &mut egui::Ui) {
        if self.project_open.recent_projects.is_empty() {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.small("No recent cases yet.");
                ui.horizontal(|ui| {
                    if ui.button("Open Case").clicked() {
                        self.open_project_from_picker();
                    }
                    if ui.button("Open Example Case").clicked() {
                        self.project_open.notice = Some(ProjectOpenNotice {
                            level: ProjectOpenNoticeLevel::Info,
                            title: "Choose an example".to_string(),
                            detail: "Use the Example Cases section to open a bundled case."
                                .to_string(),
                        });
                    }
                });
            });
            return;
        }

        for project_path in self
            .project_open
            .recent_projects
            .clone()
            .into_iter()
            .take(5)
        {
            let status = recent_case_status(&project_path);
            let case_name = project_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("case");
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    if ui.button(truncate_middle(case_name, 34)).clicked() {
                        self.open_recent_project(project_path.clone());
                    }
                    render_status_chip(ui, status, recent_case_status_color(status));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small("Last opened: MRU");
                    });
                });
                render_muted_small(ui, truncate_middle(&parent_display(&project_path), 72));
                ui.horizontal_wrapped(|ui| {
                    ui.small("Property Package:");
                    ui.small("binary-hydrocarbon-lite-v1");
                });
            });
            ui.add_space(6.0);
        }
    }

    fn render_home_example_cases(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        if window.runtime.example_projects.is_empty() {
            ui.group(|ui| {
                ui.colored_label(egui::Color32::from_rgb(160, 120, 40), "Examples missing");
                ui.small("The bundled examples directory was not discovered.");
            });
            return;
        }

        for example in window.runtime.example_projects.iter().take(6) {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(example_case_title(example.id, example.title)).strong(),
                    );
                    render_status_chip(ui, "Ready", egui::Color32::from_rgb(52, 128, 89));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(!example.is_current, egui::Button::new("Open"))
                            .on_hover_text(example.project_path.display().to_string())
                            .clicked()
                        {
                            self.open_example_project(example.project_path.clone());
                        }
                    });
                });
                render_wrapped_small(ui, example.detail);
                ui.add_space(4.0);
                render_muted_small(ui, example_case_flow_summary(example.id));
                ui.horizontal_wrapped(|ui| {
                    ui.small("Components:");
                    ui.small(example_case_components(example.id));
                    ui.separator();
                    ui.small("Property Package:");
                    ui.small(example_case_property_package(example.id));
                });
            });
            ui.add_space(8.0);
        }
    }

    fn render_home_environment(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.heading("Environment");
        ui.add_space(8.0);
        self.render_environment_section(
            ui,
            "Client",
            &[
                ("Studio", "v26.5.1-dev"),
                ("Mode", "Portable / internal"),
                ("Examples", examples_status(window)),
            ],
        );
        ui.add_space(8.0);
        self.render_environment_section(
            ui,
            "Server",
            &[
                ("Auth", "Signed out"),
                ("Control Plane", "Offline"),
                ("Package Sync", "Local only"),
            ],
        );
        ui.add_space(8.0);
        self.render_environment_section(
            ui,
            "Device",
            &[
                ("Local Cache", "Ready"),
                ("Runtime", "Ready"),
                ("OS", std::env::consts::OS),
            ],
        );
    }

    fn render_environment_section(&self, ui: &mut egui::Ui, title: &str, rows: &[(&str, &str)]) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(title).strong());
            ui.add_space(4.0);
            for (label, value) in rows {
                ui.horizontal_wrapped(|ui| {
                    ui.small(
                        egui::RichText::new(*label).color(egui::Color32::from_rgb(92, 104, 117)),
                    );
                    ui.small(*value);
                });
            }
        });
    }

    fn render_home_messages(&mut self, ctx: &egui::Context, window: &StudioGuiWindowModel) {
        egui::TopBottomPanel::bottom("studio.home_messages")
            .exact_height(136.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Messages");
                    render_status_chip(ui, "3", egui::Color32::from_rgb(56, 126, 214));
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("studio.home_messages_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(notice) = self.project_open.notice.as_ref() {
                            self.render_home_message_row(
                                ui,
                                "NOTICE",
                                &notice.title,
                                Some(&notice.detail),
                            );
                        }
                        self.render_home_message_row(
                            ui,
                            "AUTH",
                            "You are not signed in. Cloud packages and team cases are unavailable.",
                            Some("Sign in"),
                        );
                        self.render_home_message_row(
                            ui,
                            "EXAMPLES",
                            if window.runtime.example_projects.is_empty() {
                                "Built-in examples were not discovered."
                            } else {
                                "Built-in examples are available locally."
                            },
                            Some("examples/flowsheets"),
                        );
                        self.render_home_message_row(
                            ui,
                            "CACHE",
                            "Local property package cache is ready.",
                            Some("binary-hydrocarbon-lite-v1"),
                        );
                    });
            });
    }

    fn render_home_message_row(
        &self,
        ui: &mut egui::Ui,
        tag: &str,
        message: &str,
        action: Option<&str>,
    ) {
        ui.horizontal_wrapped(|ui| {
            render_status_chip(ui, tag, message_tag_color(tag));
            render_wrapped_label(ui, message);
            if let Some(action) = action {
                ui.small(action);
            }
        });
    }
}

fn render_muted_small(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.small(egui::RichText::new(text.into()).color(egui::Color32::from_rgb(92, 104, 117)));
}

fn recent_case_status(project_path: &Path) -> &'static str {
    if project_path.exists() {
        "Ready"
    } else {
        "Missing file"
    }
}

fn recent_case_status_color(status: &str) -> egui::Color32 {
    match status {
        "Ready" => egui::Color32::from_rgb(52, 128, 89),
        "Missing file" => egui::Color32::from_rgb(180, 70, 60),
        _ => egui::Color32::from_rgb(120, 120, 120),
    }
}

fn message_tag_color(tag: &str) -> egui::Color32 {
    match tag {
        "AUTH" => egui::Color32::from_rgb(160, 120, 40),
        "EXAMPLES" => egui::Color32::from_rgb(56, 126, 214),
        "CACHE" => egui::Color32::from_rgb(52, 128, 89),
        "NOTICE" => egui::Color32::from_rgb(86, 118, 168),
        _ => egui::Color32::from_rgb(120, 120, 120),
    }
}

fn parent_display(project_path: &Path) -> String {
    project_path
        .parent()
        .map(Path::display)
        .map(|display| display.to_string())
        .unwrap_or_else(|| "local".to_string())
}

fn truncate_middle(value: &str, max_chars: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= max_chars || max_chars < 4 {
        return value.to_string();
    }

    let prefix_len = (max_chars - 3) / 2;
    let suffix_len = max_chars - 3 - prefix_len;
    let prefix = value.chars().take(prefix_len).collect::<String>();
    let suffix = value
        .chars()
        .rev()
        .take(suffix_len)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("{prefix}...{suffix}")
}

fn examples_status(window: &StudioGuiWindowModel) -> &'static str {
    if window.runtime.example_projects.is_empty() {
        "Missing"
    } else {
        "Ready"
    }
}

fn example_case_title<'a>(id: &str, fallback: &'a str) -> &'a str {
    match id {
        "feed-heater-flash" => "Heater / Cooler / Valve",
        "feed-valve-flash" => "Valve",
        "feed-cooler-flash" => "Cooler",
        "feed-mixer-flash" => "Mixer",
        "water-ethanol-heater-flash" => "PME Sample",
        _ => fallback,
    }
}

fn example_case_flow_summary(id: &str) -> &'static str {
    match id {
        "feed-mixer-flash" | "feed-mixer-heater-flash" => "Feed + Feed -> Mixer -> Flash Drum",
        "feed-valve-flash" => "Feed -> Valve -> Flash Drum",
        "feed-cooler-flash" => "Feed -> Cooler -> Flash Drum",
        "water-ethanol-heater-flash" => "Feed -> Heater -> Flash Drum (water / ethanol)",
        _ => "Feed -> Heater -> Flash Drum",
    }
}

fn example_case_components(id: &str) -> &'static str {
    match id {
        "water-ethanol-heater-flash" => "Water, Ethanol",
        "feed-mixer-heater-flash" => "Methane, Ethane, Nitrogen",
        _ => "Methane, Ethane",
    }
}

fn example_case_property_package(id: &str) -> &'static str {
    match id {
        "water-ethanol-heater-flash" => "NRTL / PME sample",
        _ => "binary-hydrocarbon-lite-v1",
    }
}
