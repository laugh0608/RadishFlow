use std::path::Path;

use super::*;

const HOME_START_WIDTH: f32 = 220.0;
const HOME_ENVIRONMENT_WIDTH: f32 = 280.0;

#[derive(Debug, Clone, Copy)]
enum HomeText {
    Subtitle,
    LocalReady,
    ServerOffline,
    SignedOut,
    SignIn,
    SignInUnavailableTitle,
    SignInUnavailableDetail,
    Start,
    ContinueLastCase,
    OpenFirstExampleCase,
    OpenFirstExampleHover,
    NewBlankCase,
    OpenCase,
    OpenExampleCase,
    RecentCases,
    ExampleCases,
    NoRecentCases,
    ChooseExampleTitle,
    ChooseExampleDetail,
    LastOpenedMru,
    PropertyPackage,
    Components,
    Environment,
    Client,
    Studio,
    Mode,
    Examples,
    PortableInternal,
    Missing,
    Ready,
    Server,
    Auth,
    ControlPlane,
    PackageSync,
    Offline,
    LocalOnly,
    Device,
    LocalCache,
    Runtime,
    Os,
    ExamplesMissing,
    ExamplesMissingDetail,
    Messages,
    AuthMessage,
    ExamplesReadyMessage,
    ExamplesMissingMessage,
    CacheReadyMessage,
}

#[derive(Debug, Clone, Copy)]
enum HomeMessageTag {
    Notice,
    Auth,
    Examples,
    Cache,
}

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
                    ui.small(home_text(self.locale, HomeText::Subtitle));
                    ui.separator();
                    render_status_chip(
                        ui,
                        "v26.5.1-dev internal",
                        egui::Color32::from_rgb(86, 118, 168),
                    );
                    render_status_chip(
                        ui,
                        home_text(self.locale, HomeText::LocalReady),
                        egui::Color32::from_rgb(52, 128, 89),
                    );
                    render_status_chip(
                        ui,
                        home_text(self.locale, HomeText::ServerOffline),
                        egui::Color32::from_rgb(180, 70, 60),
                    );
                    render_status_chip(
                        ui,
                        home_text(self.locale, HomeText::SignedOut),
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
                        if ui
                            .button(self.locale.text(ShellText::ViewOptions))
                            .clicked()
                        {
                            self.command_palette.toggle();
                        }
                        if ui
                            .button(home_text(self.locale, HomeText::SignIn))
                            .clicked()
                        {
                            self.project_open.notice = Some(ProjectOpenNotice {
                                level: ProjectOpenNoticeLevel::Info,
                                title: home_text(self.locale, HomeText::SignInUnavailableTitle)
                                    .to_string(),
                                detail: home_text(self.locale, HomeText::SignInUnavailableDetail)
                                    .to_string(),
                            });
                        }
                        ui.small(self.locale.text(ShellText::UnitsSi));
                    });
                });
            });
    }

    fn render_home_start_actions(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.heading(home_text(self.locale, HomeText::Start));
        ui.add_space(8.0);

        if let Some(last_case) = self.project_open.recent_projects.first().cloned() {
            let response = ui
                .add(
                    egui::Button::new(home_text(self.locale, HomeText::ContinueLastCase))
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
                    egui::Button::new(home_text(self.locale, HomeText::OpenFirstExampleCase))
                        .fill(egui::Color32::from_rgb(230, 239, 252))
                        .min_size(egui::vec2(ui.available_width(), 44.0)),
                )
                .on_hover_text(home_text(self.locale, HomeText::OpenFirstExampleHover));
            if response.clicked() {
                if let Some(example) = window.runtime.example_projects.first() {
                    self.open_example_project(example.project_path.clone());
                }
            }
        }

        ui.add_space(8.0);
        if ui
            .add(
                egui::Button::new(home_text(self.locale, HomeText::NewBlankCase))
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            self.create_blank_project();
        }
        ui.add_space(5.0);
        if ui
            .add(
                egui::Button::new(home_text(self.locale, HomeText::OpenCase))
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            self.open_project_from_picker();
        }
        ui.add_space(5.0);
        if ui
            .add(
                egui::Button::new(home_text(self.locale, HomeText::OpenExampleCase))
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
            ui.heading(home_text(self.locale, HomeText::RecentCases));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(home_text(self.locale, HomeText::OpenCase))
                    .clicked()
                {
                    self.open_project_from_picker();
                }
            });
        });
        self.render_home_recent_cases(ui);
        ui.add_space(18.0);
        ui.horizontal(|ui| {
            ui.heading(home_text(self.locale, HomeText::ExampleCases));
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
                ui.small(home_text(self.locale, HomeText::NoRecentCases));
                ui.horizontal(|ui| {
                    if ui
                        .button(home_text(self.locale, HomeText::OpenCase))
                        .clicked()
                    {
                        self.open_project_from_picker();
                    }
                    if ui
                        .button(home_text(self.locale, HomeText::OpenExampleCase))
                        .clicked()
                    {
                        self.project_open.notice = Some(ProjectOpenNotice {
                            level: ProjectOpenNoticeLevel::Info,
                            title: home_text(self.locale, HomeText::ChooseExampleTitle).to_string(),
                            detail: home_text(self.locale, HomeText::ChooseExampleDetail)
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
                    render_status_chip(
                        ui,
                        recent_case_status_text(self.locale, status),
                        recent_case_status_color(status),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small(home_text(self.locale, HomeText::LastOpenedMru));
                    });
                });
                render_muted_small(ui, truncate_middle(&parent_display(&project_path), 72));
                ui.horizontal_wrapped(|ui| {
                    ui.small(home_text(self.locale, HomeText::PropertyPackage));
                    ui.small("binary-hydrocarbon-lite-v1");
                });
            });
            ui.add_space(6.0);
        }
    }

    fn render_home_example_cases(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        if window.runtime.example_projects.is_empty() {
            ui.group(|ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(160, 120, 40),
                    home_text(self.locale, HomeText::ExamplesMissing),
                );
                ui.small(home_text(self.locale, HomeText::ExamplesMissingDetail));
            });
            return;
        }

        for example in window.runtime.example_projects.iter().take(6) {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(example_case_title(
                            self.locale,
                            example.id,
                            example.title,
                        ))
                        .strong(),
                    );
                    render_status_chip(
                        ui,
                        home_text(self.locale, HomeText::Ready),
                        egui::Color32::from_rgb(52, 128, 89),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !example.is_current,
                                egui::Button::new(home_text(self.locale, HomeText::OpenCase)),
                            )
                            .on_hover_text(example.project_path.display().to_string())
                            .clicked()
                        {
                            self.open_example_project(example.project_path.clone());
                        }
                    });
                });
                render_wrapped_small(
                    ui,
                    example_case_detail(self.locale, example.id, example.detail),
                );
                ui.add_space(4.0);
                render_muted_small(ui, example_case_flow_summary(self.locale, example.id));
                ui.horizontal_wrapped(|ui| {
                    ui.small(home_text(self.locale, HomeText::Components));
                    ui.small(example_case_components(self.locale, example.id));
                    ui.separator();
                    ui.small(home_text(self.locale, HomeText::PropertyPackage));
                    ui.small(example_case_property_package(example.id));
                });
            });
            ui.add_space(8.0);
        }
    }

    fn render_home_environment(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.heading(home_text(self.locale, HomeText::Environment));
        ui.add_space(8.0);
        self.render_environment_section(
            ui,
            home_text(self.locale, HomeText::Client),
            &[
                (home_text(self.locale, HomeText::Studio), "v26.5.1-dev"),
                (
                    home_text(self.locale, HomeText::Mode),
                    home_text(self.locale, HomeText::PortableInternal),
                ),
                (
                    home_text(self.locale, HomeText::Examples),
                    examples_status(self.locale, window),
                ),
            ],
        );
        ui.add_space(8.0);
        self.render_environment_section(
            ui,
            home_text(self.locale, HomeText::Server),
            &[
                (
                    home_text(self.locale, HomeText::Auth),
                    home_text(self.locale, HomeText::SignedOut),
                ),
                (
                    home_text(self.locale, HomeText::ControlPlane),
                    home_text(self.locale, HomeText::Offline),
                ),
                (
                    home_text(self.locale, HomeText::PackageSync),
                    home_text(self.locale, HomeText::LocalOnly),
                ),
            ],
        );
        ui.add_space(8.0);
        self.render_environment_section(
            ui,
            home_text(self.locale, HomeText::Device),
            &[
                (
                    home_text(self.locale, HomeText::LocalCache),
                    home_text(self.locale, HomeText::Ready),
                ),
                (
                    home_text(self.locale, HomeText::Runtime),
                    home_text(self.locale, HomeText::Ready),
                ),
                (home_text(self.locale, HomeText::Os), std::env::consts::OS),
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
                    ui.heading(home_text(self.locale, HomeText::Messages));
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
                                HomeMessageTag::Notice,
                                &notice.title,
                                Some(&notice.detail),
                            );
                        }
                        self.render_home_message_row(
                            ui,
                            HomeMessageTag::Auth,
                            home_text(self.locale, HomeText::AuthMessage),
                            Some(home_text(self.locale, HomeText::SignIn)),
                        );
                        self.render_home_message_row(
                            ui,
                            HomeMessageTag::Examples,
                            if window.runtime.example_projects.is_empty() {
                                home_text(self.locale, HomeText::ExamplesMissingMessage)
                            } else {
                                home_text(self.locale, HomeText::ExamplesReadyMessage)
                            },
                            Some("examples/flowsheets"),
                        );
                        self.render_home_message_row(
                            ui,
                            HomeMessageTag::Cache,
                            home_text(self.locale, HomeText::CacheReadyMessage),
                            Some("binary-hydrocarbon-lite-v1"),
                        );
                    });
            });
    }

    fn render_home_message_row(
        &self,
        ui: &mut egui::Ui,
        tag: HomeMessageTag,
        message: &str,
        action: Option<&str>,
    ) {
        ui.horizontal_wrapped(|ui| {
            render_status_chip(
                ui,
                home_message_tag_text(self.locale, tag),
                message_tag_color(tag),
            );
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

fn recent_case_status_text(locale: StudioShellLocale, status: &str) -> &'static str {
    match status {
        "Ready" => home_text(locale, HomeText::Ready),
        "Missing file" => match locale {
            StudioShellLocale::En => "Missing file",
            StudioShellLocale::ZhCn => "文件缺失",
        },
        _ => match locale {
            StudioShellLocale::En => "Unknown",
            StudioShellLocale::ZhCn => "未知",
        },
    }
}

fn message_tag_color(tag: HomeMessageTag) -> egui::Color32 {
    match tag {
        HomeMessageTag::Auth => egui::Color32::from_rgb(160, 120, 40),
        HomeMessageTag::Examples => egui::Color32::from_rgb(56, 126, 214),
        HomeMessageTag::Cache => egui::Color32::from_rgb(52, 128, 89),
        HomeMessageTag::Notice => egui::Color32::from_rgb(86, 118, 168),
    }
}

fn home_message_tag_text(locale: StudioShellLocale, tag: HomeMessageTag) -> &'static str {
    match locale {
        StudioShellLocale::En => match tag {
            HomeMessageTag::Notice => "NOTICE",
            HomeMessageTag::Auth => "AUTH",
            HomeMessageTag::Examples => "EXAMPLES",
            HomeMessageTag::Cache => "CACHE",
        },
        StudioShellLocale::ZhCn => match tag {
            HomeMessageTag::Notice => "提示",
            HomeMessageTag::Auth => "认证",
            HomeMessageTag::Examples => "示例",
            HomeMessageTag::Cache => "缓存",
        },
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

fn examples_status(locale: StudioShellLocale, window: &StudioGuiWindowModel) -> &'static str {
    if window.runtime.example_projects.is_empty() {
        home_text(locale, HomeText::Missing)
    } else {
        home_text(locale, HomeText::Ready)
    }
}

fn example_case_title<'a>(
    locale: StudioShellLocale,
    id: &str,
    fallback: &'a str,
) -> std::borrow::Cow<'a, str> {
    match locale {
        StudioShellLocale::En => match id {
            "feed-heater-flash" => std::borrow::Cow::Borrowed("Heater / Cooler / Valve"),
            "feed-valve-flash" => std::borrow::Cow::Borrowed("Valve"),
            "feed-cooler-flash" => std::borrow::Cow::Borrowed("Cooler"),
            "feed-mixer-flash" => std::borrow::Cow::Borrowed("Mixer"),
            "water-ethanol-heater-flash" => std::borrow::Cow::Borrowed("PME Sample"),
            _ => std::borrow::Cow::Borrowed(fallback),
        },
        StudioShellLocale::ZhCn => match id {
            "feed-heater-flash" => std::borrow::Cow::Borrowed("加热 / 冷却 / 阀门"),
            "feed-valve-flash" => std::borrow::Cow::Borrowed("阀门"),
            "feed-cooler-flash" => std::borrow::Cow::Borrowed("冷却器"),
            "feed-mixer-flash" => std::borrow::Cow::Borrowed("混合器"),
            "feed-mixer-heater-flash" => std::borrow::Cow::Borrowed("混合后加热"),
            "water-ethanol-heater-flash" => std::borrow::Cow::Borrowed("PME 样例"),
            _ => std::borrow::Cow::Borrowed(fallback),
        },
    }
}

fn example_case_detail<'a>(
    locale: StudioShellLocale,
    id: &str,
    fallback: &'a str,
) -> std::borrow::Cow<'a, str> {
    match locale {
        StudioShellLocale::En => std::borrow::Cow::Borrowed(fallback),
        StudioShellLocale::ZhCn => match id {
            "feed-valve-flash" => std::borrow::Cow::Borrowed("单股进料，经阀门降压后进入闪蒸罐。"),
            "feed-cooler-flash" => std::borrow::Cow::Borrowed("单股进料，经冷却器后进入闪蒸罐。"),
            "feed-mixer-flash" => std::borrow::Cow::Borrowed("两股进料混合后进入闪蒸罐。"),
            "feed-mixer-heater-flash" => {
                std::borrow::Cow::Borrowed("两股进料混合并加热后进入闪蒸罐。")
            }
            "water-ethanol-heater-flash" => {
                std::borrow::Cow::Borrowed("水 / 乙醇 PME 样例，进料加热后进入闪蒸罐。")
            }
            _ => std::borrow::Cow::Borrowed("单股进料，经加热器后进入闪蒸罐。"),
        },
    }
}

fn example_case_flow_summary(locale: StudioShellLocale, id: &str) -> &'static str {
    match locale {
        StudioShellLocale::En => match id {
            "feed-mixer-flash" | "feed-mixer-heater-flash" => "Feed + Feed -> Mixer -> Flash Drum",
            "feed-valve-flash" => "Feed -> Valve -> Flash Drum",
            "feed-cooler-flash" => "Feed -> Cooler -> Flash Drum",
            "water-ethanol-heater-flash" => "Feed -> Heater -> Flash Drum (water / ethanol)",
            _ => "Feed -> Heater -> Flash Drum",
        },
        StudioShellLocale::ZhCn => match id {
            "feed-mixer-flash" | "feed-mixer-heater-flash" => "进料 + 进料 -> 混合器 -> 闪蒸罐",
            "feed-valve-flash" => "进料 -> 阀门 -> 闪蒸罐",
            "feed-cooler-flash" => "进料 -> 冷却器 -> 闪蒸罐",
            "water-ethanol-heater-flash" => "进料 -> 加热器 -> 闪蒸罐（水 / 乙醇）",
            _ => "进料 -> 加热器 -> 闪蒸罐",
        },
    }
}

fn example_case_components(locale: StudioShellLocale, id: &str) -> &'static str {
    match locale {
        StudioShellLocale::En => match id {
            "water-ethanol-heater-flash" => "Water, Ethanol",
            "feed-mixer-heater-flash" => "Methane, Ethane, Nitrogen",
            _ => "Methane, Ethane",
        },
        StudioShellLocale::ZhCn => match id {
            "water-ethanol-heater-flash" => "水, 乙醇",
            "feed-mixer-heater-flash" => "甲烷, 乙烷, 氮气",
            _ => "甲烷, 乙烷",
        },
    }
}

fn example_case_property_package(id: &str) -> &'static str {
    match id {
        "water-ethanol-heater-flash" => "NRTL / PME sample",
        _ => "binary-hydrocarbon-lite-v1",
    }
}

fn home_text(locale: StudioShellLocale, key: HomeText) -> &'static str {
    match locale {
        StudioShellLocale::En => match key {
            HomeText::Subtitle => "Steady-State Process Simulation",
            HomeText::LocalReady => "Local ready",
            HomeText::ServerOffline => "Server offline",
            HomeText::SignedOut => "Signed out",
            HomeText::SignIn => "Sign in",
            HomeText::SignInUnavailableTitle => "Sign in unavailable",
            HomeText::SignInUnavailableDetail => {
                "OIDC / PKCE browser sign-in is not attached to this internal build yet."
            }
            HomeText::Start => "Start",
            HomeText::ContinueLastCase => "Continue Last Case",
            HomeText::OpenFirstExampleCase => "Open Example Case",
            HomeText::OpenFirstExampleHover => "Open the first bundled example case.",
            HomeText::NewBlankCase => "New Blank Case",
            HomeText::OpenCase => "Open Case",
            HomeText::OpenExampleCase => "Open Example Case",
            HomeText::RecentCases => "Recent Cases",
            HomeText::ExampleCases => "Example Cases",
            HomeText::NoRecentCases => "No recent cases yet.",
            HomeText::ChooseExampleTitle => "Choose an example",
            HomeText::ChooseExampleDetail => {
                "Use the Example Cases section to open a bundled case."
            }
            HomeText::LastOpenedMru => "Last opened: MRU",
            HomeText::PropertyPackage => "Property Package:",
            HomeText::Components => "Components:",
            HomeText::Environment => "Environment",
            HomeText::Client => "Client",
            HomeText::Studio => "Studio",
            HomeText::Mode => "Mode",
            HomeText::Examples => "Examples",
            HomeText::PortableInternal => "Portable / internal",
            HomeText::Missing => "Missing",
            HomeText::Ready => "Ready",
            HomeText::Server => "Server",
            HomeText::Auth => "Auth",
            HomeText::ControlPlane => "Control Plane",
            HomeText::PackageSync => "Package Sync",
            HomeText::Offline => "Offline",
            HomeText::LocalOnly => "Local only",
            HomeText::Device => "Device",
            HomeText::LocalCache => "Local Cache",
            HomeText::Runtime => "Runtime",
            HomeText::Os => "OS",
            HomeText::ExamplesMissing => "Examples missing",
            HomeText::ExamplesMissingDetail => "The bundled examples directory was not discovered.",
            HomeText::Messages => "Messages",
            HomeText::AuthMessage => {
                "You are not signed in. Cloud packages and team cases are unavailable."
            }
            HomeText::ExamplesReadyMessage => "Built-in examples are available locally.",
            HomeText::ExamplesMissingMessage => "Built-in examples were not discovered.",
            HomeText::CacheReadyMessage => "Local property package cache is ready.",
        },
        StudioShellLocale::ZhCn => match key {
            HomeText::Subtitle => "稳态流程模拟",
            HomeText::LocalReady => "本地就绪",
            HomeText::ServerOffline => "服务端离线",
            HomeText::SignedOut => "未登录",
            HomeText::SignIn => "登录",
            HomeText::SignInUnavailableTitle => "登录暂不可用",
            HomeText::SignInUnavailableDetail => "当前内部构建尚未接入 OIDC / PKCE 浏览器登录。",
            HomeText::Start => "开始",
            HomeText::ContinueLastCase => "继续上次 Case",
            HomeText::OpenFirstExampleCase => "打开示例 Case",
            HomeText::OpenFirstExampleHover => "打开第一个内置示例 Case。",
            HomeText::NewBlankCase => "新建空白 Case",
            HomeText::OpenCase => "打开 Case",
            HomeText::OpenExampleCase => "打开示例 Case",
            HomeText::RecentCases => "最近 Case",
            HomeText::ExampleCases => "示例 Case",
            HomeText::NoRecentCases => "还没有最近 Case。",
            HomeText::ChooseExampleTitle => "请选择示例",
            HomeText::ChooseExampleDetail => "从示例 Case 区域打开一个内置 Case。",
            HomeText::LastOpenedMru => "上次打开: MRU",
            HomeText::PropertyPackage => "物性包:",
            HomeText::Components => "组分:",
            HomeText::Environment => "环境",
            HomeText::Client => "客户端",
            HomeText::Studio => "Studio",
            HomeText::Mode => "模式",
            HomeText::Examples => "示例",
            HomeText::PortableInternal => "便携 / 内部",
            HomeText::Missing => "缺失",
            HomeText::Ready => "就绪",
            HomeText::Server => "服务端",
            HomeText::Auth => "认证",
            HomeText::ControlPlane => "控制面",
            HomeText::PackageSync => "物性包同步",
            HomeText::Offline => "离线",
            HomeText::LocalOnly => "仅本地",
            HomeText::Device => "设备",
            HomeText::LocalCache => "本地缓存",
            HomeText::Runtime => "运行时",
            HomeText::Os => "操作系统",
            HomeText::ExamplesMissing => "示例缺失",
            HomeText::ExamplesMissingDetail => "未发现内置示例目录。",
            HomeText::Messages => "消息",
            HomeText::AuthMessage => "尚未登录。云端物性包和团队 Case 暂不可用。",
            HomeText::ExamplesReadyMessage => "内置示例已在本地可用。",
            HomeText::ExamplesMissingMessage => "未发现内置示例。",
            HomeText::CacheReadyMessage => "本地物性包缓存已就绪。",
        },
    }
}
