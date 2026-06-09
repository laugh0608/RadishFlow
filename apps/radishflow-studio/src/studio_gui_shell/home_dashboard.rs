use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::*;

const HOME_START_WIDTH: f32 = 220.0;
const HOME_ENVIRONMENT_WIDTH: f32 = 280.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::studio_gui_shell) enum HomeCaseRowAction {
    None,
    Select,
    Open,
}

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
    ReturnWorkspace,
    ReturnWorkspaceDetail,
    NewBlankCase,
    AuthorMixerFlashCase,
    AuthorMixerFlashDetail,
    AuthorHeaterFlashCase,
    AuthorHeaterFlashDetail,
    OpenCase,
    OpenExampleCase,
    RecentCases,
    ExampleCases,
    NoRecentCases,
    ChooseExampleTitle,
    ChooseExampleDetail,
    LastOpenedMru,
    CurrentWorkspace,
    PropertyPackage,
    Components,
    Environment,
    Client,
    Studio,
    DevelopmentBuild,
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
    pub(super) fn window_model_with_shell_home(
        &mut self,
        snapshot: &radishflow_studio::StudioGuiSnapshot,
    ) -> StudioGuiWindowModel {
        self.project_open.sync_recent_case_tiles();
        let mut window = snapshot.window_model();
        window.home.recent_case_tiles = self
            .project_open
            .recent_case_tiles_for_current(&window.runtime.workspace_document);
        window
    }

    pub(super) fn render_home_dashboard(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
    ) {
        self.reconcile_home_case_selection(window);
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
                        home_text(self.locale, HomeText::DevelopmentBuild),
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

        if self.home_current_workspace_is_available() {
            if ui
                .add(
                    egui::Button::new(home_text(self.locale, HomeText::ReturnWorkspace))
                        .fill(egui::Color32::from_rgb(226, 244, 236))
                        .min_size(egui::vec2(ui.available_width(), 42.0)),
                )
                .clicked()
            {
                self.open_current_workspace_from_home();
            }
            render_wrapped_small(ui, home_text(self.locale, HomeText::ReturnWorkspaceDetail));
            ui.add_space(8.0);
        }

        if ui
            .add(
                egui::Button::new(home_text(self.locale, HomeText::NewBlankCase))
                    .fill(egui::Color32::from_rgb(230, 239, 252))
                    .min_size(egui::vec2(ui.available_width(), 44.0)),
            )
            .clicked()
        {
            self.create_blank_project();
        }
        ui.add_space(5.0);
        if ui
            .add(
                egui::Button::new(home_text(self.locale, HomeText::AuthorMixerFlashCase))
                    .min_size(egui::vec2(ui.available_width(), 40.0)),
            )
            .clicked()
        {
            self.start_mixer_flash_authoring_case();
        }
        render_wrapped_small(ui, home_text(self.locale, HomeText::AuthorMixerFlashDetail));
        ui.add_space(5.0);
        if ui
            .add(
                egui::Button::new(home_text(self.locale, HomeText::AuthorHeaterFlashCase))
                    .min_size(egui::vec2(ui.available_width(), 40.0)),
            )
            .clicked()
        {
            self.start_heater_flash_authoring_case();
        }
        render_wrapped_small(
            ui,
            home_text(self.locale, HomeText::AuthorHeaterFlashDetail),
        );
        ui.add_space(5.0);
        if ui
            .add(
                egui::Button::new(home_text(self.locale, HomeText::OpenCase))
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            self.open_selected_recent_project_or_picker();
        }
        ui.add_space(5.0);
        if ui
            .add(
                egui::Button::new(home_text(self.locale, HomeText::OpenExampleCase))
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            self.open_selected_example_project(window);
        }
    }

    fn render_home_cases(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.horizontal(|ui| {
            ui.heading(home_text(self.locale, HomeText::RecentCases));
        });
        self.render_home_recent_cases(ui, window);
        ui.add_space(18.0);
        ui.horizontal(|ui| {
            ui.heading(home_text(self.locale, HomeText::ExampleCases));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.small("examples/flowsheets");
            });
        });
        self.render_home_example_cases(ui, window);
    }

    fn render_home_recent_cases(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        if window.home.recent_case_tiles.is_empty() {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.small(home_text(self.locale, HomeText::NoRecentCases));
            });
            return;
        }

        for tile in window.home.recent_case_tiles.iter().take(5) {
            let current_workspace_tile = home_tile_targets_current_workspace(tile, window);
            let project_path = if tile.source
                == radishflow_studio::StudioGuiWindowHomeCaseTileSource::Current
                && window.runtime.workspace_document.project_path.is_none()
            {
                None
            } else {
                Some(PathBuf::from(&tile.path_text))
            };
            let is_selected = if current_workspace_tile {
                self.home_selected_current_workspace
                    || self
                        .home_selected_recent_project
                        .as_ref()
                        .is_some_and(|selected| paths_match(selected, Path::new(&tile.path_text)))
            } else {
                !self.home_selected_current_workspace
                    && self
                        .home_selected_recent_project
                        .as_ref()
                        .zip(project_path.as_ref())
                        .is_some_and(|(selected, project_path)| paths_match(selected, project_path))
            };
            let fill = if is_selected {
                egui::Color32::from_rgb(230, 239, 252)
            } else {
                ui.visuals().widgets.noninteractive.bg_fill
            };
            let frame_response = egui::Frame::group(ui.style()).fill(fill).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(truncate_middle(&tile.title, 34)).strong());
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(tile.status_label).as_ref(),
                        home_case_tile_status_color(tile.status),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small(home_text(self.locale, HomeText::LastOpenedMru));
                    });
                });
                render_wrapped_small(ui, &tile.detail);
                render_home_thumbnail(ui, &tile.thumbnail);
                let location = if tile.source
                    == radishflow_studio::StudioGuiWindowHomeCaseTileSource::Current
                    && window.runtime.workspace_document.project_path.is_none()
                {
                    home_text(self.locale, HomeText::CurrentWorkspace).to_string()
                } else {
                    project_path
                        .as_deref()
                        .map(parent_display)
                        .unwrap_or_else(|| {
                            home_text(self.locale, HomeText::CurrentWorkspace).to_string()
                        })
                };
                render_muted_small(ui, truncate_middle(&location, 72));
                ui.horizontal_wrapped(|ui| {
                    ui.small(home_text(self.locale, HomeText::Components));
                    ui.small(&tile.component_summary);
                    ui.separator();
                    ui.small(home_text(self.locale, HomeText::PropertyPackage));
                    ui.small(self.locale.runtime_label(&tile.package_summary).as_ref());
                });
            });
            let row_response = ui
                .interact(
                    frame_response.response.rect,
                    ui.make_persistent_id(("home-recent-case-row", tile.source_id.clone())),
                    egui::Sense::click(),
                )
                .on_hover_text(&tile.path_text)
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            match home_case_row_action(row_response.clicked(), row_response.double_clicked()) {
                HomeCaseRowAction::Open => {
                    if current_workspace_tile {
                        self.open_current_workspace_from_home();
                    } else if let Some(project_path) = project_path.clone() {
                        self.open_recent_project(project_path);
                    }
                }
                HomeCaseRowAction::Select => {
                    self.home_selected_current_workspace = current_workspace_tile;
                    self.home_selected_recent_project = project_path.clone();
                }
                HomeCaseRowAction::None => {}
            }
            ui.add_space(6.0);
        }
    }

    fn render_home_example_cases(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        if window.home.example_case_tiles.is_empty() {
            ui.group(|ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(160, 120, 40),
                    home_text(self.locale, HomeText::ExamplesMissing),
                );
                ui.small(home_text(self.locale, HomeText::ExamplesMissingDetail));
            });
            return;
        }

        for tile in window.home.example_case_tiles.iter().take(6) {
            let example_path = window
                .runtime
                .example_projects
                .iter()
                .find(|example| example.id == tile.source_id)
                .map(|example| example.project_path.clone());
            let is_selected = self
                .home_selected_example_project
                .as_ref()
                .zip(example_path.as_ref())
                .is_some_and(|(selected, example_path)| paths_match(selected, example_path));
            let fill = if is_selected {
                egui::Color32::from_rgb(230, 239, 252)
            } else {
                ui.visuals().widgets.noninteractive.bg_fill
            };
            let frame_response = egui::Frame::group(ui.style()).fill(fill).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(example_case_title(
                            self.locale,
                            &tile.source_id,
                            &tile.title,
                        ))
                        .strong(),
                    );
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(tile.status_label).as_ref(),
                        home_case_tile_status_color(tile.status),
                    );
                });
                render_wrapped_small(
                    ui,
                    example_case_detail(self.locale, &tile.source_id, &tile.detail),
                );
                ui.add_space(4.0);
                render_home_thumbnail(ui, &tile.thumbnail);
                render_muted_small(ui, example_case_flow_summary(self.locale, &tile.source_id));
                ui.horizontal_wrapped(|ui| {
                    ui.small(home_text(self.locale, HomeText::Components));
                    ui.small(&tile.component_summary);
                    ui.separator();
                    ui.small(home_text(self.locale, HomeText::PropertyPackage));
                    ui.small(&tile.package_summary);
                });
            });
            let row_response = ui
                .interact(
                    frame_response.response.rect,
                    ui.make_persistent_id(("home-example-case-row", tile.source_id.clone())),
                    egui::Sense::click(),
                )
                .on_hover_text(&tile.path_text)
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if let Some(example_path) = example_path {
                match home_case_row_action(row_response.clicked(), row_response.double_clicked()) {
                    HomeCaseRowAction::Open => self.open_example_project(example_path),
                    HomeCaseRowAction::Select => {
                        self.home_selected_example_project = Some(example_path);
                    }
                    HomeCaseRowAction::None => {}
                }
            }
            ui.add_space(8.0);
        }
    }

    fn reconcile_home_case_selection(&mut self, window: &StudioGuiWindowModel) {
        let has_current_workspace_tile = window
            .home
            .recent_case_tiles
            .iter()
            .any(|tile| home_tile_targets_current_workspace(tile, window));
        let selected_recent_project_is_valid = self
            .home_selected_recent_project
            .as_ref()
            .is_some_and(|selected| {
                window.home.recent_case_tiles.iter().any(|tile| {
                    !home_tile_targets_current_workspace(tile, window)
                        && paths_match(Path::new(&tile.path_text), selected)
                })
            });
        if self.home_selected_current_workspace && !has_current_workspace_tile {
            self.home_selected_current_workspace = false;
        }
        if !self.home_selected_current_workspace && !selected_recent_project_is_valid {
            if has_current_workspace_tile {
                self.home_selected_current_workspace = true;
                self.home_selected_recent_project = None;
            } else {
                self.home_selected_recent_project = window
                    .home
                    .recent_case_tiles
                    .iter()
                    .find(|tile| !home_tile_targets_current_workspace(tile, window))
                    .map(|tile| PathBuf::from(&tile.path_text));
            }
        }

        if self
            .home_selected_example_project
            .as_ref()
            .is_none_or(|selected| {
                !window
                    .home
                    .example_case_tiles
                    .iter()
                    .any(|tile| tile.path_text == selected.display().to_string())
            })
        {
            self.home_selected_example_project =
                window.home.example_case_tiles.first().and_then(|tile| {
                    window
                        .runtime
                        .example_projects
                        .iter()
                        .find(|example| example.id == tile.source_id)
                        .map(|example| example.project_path.clone())
                });
        }
    }

    pub(in crate::studio_gui_shell) fn open_selected_recent_project_or_picker(&mut self) {
        if self.home_selected_current_workspace && self.home_current_workspace_is_available() {
            self.open_current_workspace_from_home();
        } else if let Some(project_path) = self.home_selected_recent_project.clone() {
            if self.selected_recent_project_is_current_workspace(&project_path) {
                self.open_current_workspace_from_home();
            } else {
                self.open_recent_project(project_path);
            }
        } else {
            self.open_project_from_picker();
        }
    }

    fn open_current_workspace_from_home(&mut self) {
        match self.current_workspace_home_entry_screen() {
            StudioShellScreen::Workbench => self.enter_flowsheet_modeling_from_property(),
            screen => self.screen = screen,
        }
    }

    fn current_workspace_home_entry_screen(&self) -> StudioShellScreen {
        let flowsheet_modeling_enabled = self
            .platform_host
            .snapshot()
            .window_model()
            .property_page
            .flowsheet_modeling_enabled;
        if flowsheet_modeling_enabled {
            StudioShellScreen::Workbench
        } else {
            StudioShellScreen::Property
        }
    }

    fn selected_recent_project_is_current_workspace(&self, project_path: &Path) -> bool {
        self.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .as_deref()
            .is_some_and(|current| paths_match(project_path, Path::new(current)))
    }

    fn home_current_workspace_is_available(&self) -> bool {
        !self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .title
            .trim()
            .is_empty()
    }

    pub(in crate::studio_gui_shell) fn open_selected_example_project(
        &mut self,
        window: &StudioGuiWindowModel,
    ) {
        let project_path = self.home_selected_example_project.clone().or_else(|| {
            window
                .runtime
                .example_projects
                .first()
                .map(|example| example.project_path.clone())
        });
        if let Some(project_path) = project_path {
            self.open_example_project(project_path);
        } else {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Info,
                title: home_text(self.locale, HomeText::ChooseExampleTitle).to_string(),
                detail: home_text(self.locale, HomeText::ChooseExampleDetail).to_string(),
            });
        }
    }

    fn render_home_environment(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.heading(home_text(self.locale, HomeText::Environment));
        ui.add_space(8.0);
        self.render_environment_section(
            ui,
            home_text(self.locale, HomeText::Client),
            &[
                (
                    home_text(self.locale, HomeText::Studio),
                    home_text(self.locale, HomeText::DevelopmentBuild),
                ),
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
                        if let Some(notice) = self.project_open.notice.clone() {
                            self.render_home_message_row(
                                ui,
                                HomeMessageTag::Notice,
                                &notice.title,
                                Some(&notice.detail),
                            );
                            self.render_home_project_operation_actions(ui);
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

    fn render_home_project_operation_actions(&mut self, ui: &mut egui::Ui) {
        if self.project_open.pending_confirmation.is_none()
            && !self.project_open.pending_blank_project_confirmation
            && self.project_open.pending_save_as_overwrite.is_none()
            && self
                .project_open
                .pending_close_window_confirmation
                .is_none()
        {
            return;
        }

        ui.horizontal_wrapped(|ui| {
            if self.project_open.pending_confirmation.is_some() {
                if ui
                    .button(self.locale.text(ShellText::ContinueOpenProject))
                    .clicked()
                {
                    self.confirm_pending_project_open();
                }
                if ui
                    .button(self.locale.text(ShellText::CancelOpenProject))
                    .clicked()
                {
                    self.cancel_pending_project_open();
                }
            }
            if self.project_open.pending_blank_project_confirmation {
                if ui
                    .button(self.locale.text(ShellText::ContinueNewBlankProject))
                    .clicked()
                {
                    self.confirm_pending_blank_project();
                }
                if ui
                    .button(self.locale.text(ShellText::CancelNewBlankProject))
                    .clicked()
                {
                    self.cancel_pending_blank_project();
                }
            }
            if self
                .project_open
                .pending_close_window_confirmation
                .is_some()
            {
                if ui
                    .button(self.locale.text(ShellText::SaveAndCloseProject))
                    .clicked()
                    && self.save_pending_close_window()
                {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui
                    .button(self.locale.text(ShellText::DiscardAndCloseProject))
                    .clicked()
                    && self.confirm_pending_close_window()
                {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui
                    .button(self.locale.text(ShellText::CancelCloseProject))
                    .clicked()
                {
                    self.cancel_pending_close_window();
                }
            }
            if self.project_open.pending_save_as_overwrite.is_some() {
                if ui
                    .button(self.locale.text(ShellText::ConfirmSaveAsOverwrite))
                    .clicked()
                {
                    self.confirm_pending_save_as_overwrite();
                }
                if ui
                    .button(self.locale.text(ShellText::CancelSaveAsOverwrite))
                    .clicked()
                {
                    self.cancel_pending_save_as_overwrite();
                }
            }
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

impl ProjectOpenState {
    fn sync_recent_case_tiles(&mut self) {
        if self.recent_case_tile_paths_match_recent_projects() {
            return;
        }

        self.recent_case_tiles = self
            .recent_projects
            .iter()
            .map(|project_path| recent_case_tile_from_path(project_path))
            .collect();
    }

    fn recent_case_tile_paths_match_recent_projects(&self) -> bool {
        self.recent_case_tiles.len() == self.recent_projects.len()
            && self
                .recent_case_tiles
                .iter()
                .zip(&self.recent_projects)
                .all(|(tile, project_path)| {
                    tile.source == radishflow_studio::StudioGuiWindowHomeCaseTileSource::Recent
                        && tile.path_text == project_path.display().to_string()
                })
    }

    fn recent_case_tiles_for_current(
        &self,
        document: &radishflow_studio::StudioGuiWorkspaceDocumentSnapshot,
    ) -> Vec<radishflow_studio::StudioGuiWindowHomeCaseTileModel> {
        let current_project_path = document.project_path.as_deref();
        let mut tiles = self
            .recent_case_tiles
            .iter()
            .cloned()
            .map(|mut tile| {
                let project_path = Path::new(&tile.path_text);
                tile.status = recent_case_tile_status(project_path, current_project_path);
                tile.status_label = home_case_tile_status_label(tile.status);
                tile
            })
            .collect::<Vec<_>>();
        if !tiles.iter().any(|tile| {
            tile.status == radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current
        }) {
            tiles.insert(0, current_workspace_case_tile(document));
        }
        tiles
    }
}

fn render_muted_small(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.small(egui::RichText::new(text.into()).color(egui::Color32::from_rgb(92, 104, 117)));
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

fn recent_case_tile_from_path(
    project_path: &Path,
) -> radishflow_studio::StudioGuiWindowHomeCaseTileModel {
    let project_file = project_path
        .exists()
        .then(|| rf_store::read_project_file(project_path).ok())
        .flatten();
    let title = project_file
        .as_ref()
        .and_then(|project| non_empty_trimmed(&project.document.metadata.title))
        .unwrap_or_else(|| recent_case_title_from_path(project_path));
    let detail = project_file
        .as_ref()
        .map(|project| project.document.flowsheet.name.clone())
        .unwrap_or_else(|| parent_display(project_path));
    let package_summary = project_file
        .as_ref()
        .and_then(|project| project.document.flowsheet.property_package_id())
        .map(str::to_string)
        .unwrap_or_else(|| "Unselected".to_string());
    let component_summary = project_file
        .as_ref()
        .map(|project| component_summary_from_flowsheet(&project.document.flowsheet))
        .unwrap_or_else(|| "Components unavailable".to_string());
    let thumbnail = project_file
        .as_ref()
        .map(|project| thumbnail_from_flowsheet(&project.document.flowsheet))
        .unwrap_or_else(|| radishflow_studio::StudioGuiWindowThumbnailFlowModel {
            nodes: Vec::new(),
            edges: Vec::new(),
        });
    let status = recent_case_tile_status(project_path, None);
    let path_text = project_path.display().to_string();

    radishflow_studio::StudioGuiWindowHomeCaseTileModel {
        source: radishflow_studio::StudioGuiWindowHomeCaseTileSource::Recent,
        source_id: path_text.clone(),
        title,
        detail,
        path_text,
        package_summary,
        component_summary,
        status,
        status_label: home_case_tile_status_label(status),
        thumbnail,
    }
}

fn current_workspace_case_tile(
    document: &radishflow_studio::StudioGuiWorkspaceDocumentSnapshot,
) -> radishflow_studio::StudioGuiWindowHomeCaseTileModel {
    let path_text = document
        .project_path
        .clone()
        .unwrap_or_else(|| "Current workspace".to_string());
    let package_summary = document
        .property_package_choices
        .iter()
        .find(|choice| choice.selected)
        .map(|choice| choice.label.clone())
        .or_else(|| document.property_package_id.clone())
        .unwrap_or_else(|| "Unselected".to_string());
    let component_summary =
        component_summary_from_component_choices(&document.project_component_choices);

    radishflow_studio::StudioGuiWindowHomeCaseTileModel {
        source: radishflow_studio::StudioGuiWindowHomeCaseTileSource::Current,
        source_id: "current-workspace".to_string(),
        title: document.title.clone(),
        detail: document.flowsheet_name.clone(),
        path_text,
        package_summary,
        component_summary,
        status: radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current,
        status_label: home_case_tile_status_label(
            radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current,
        ),
        thumbnail: current_workspace_thumbnail(document),
    }
}

fn non_empty_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn recent_case_title_from_path(project_path: &Path) -> String {
    project_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| project_path.display().to_string())
}

fn component_summary_from_flowsheet(flowsheet: &rf_model::Flowsheet) -> String {
    let component_names = flowsheet
        .components
        .values()
        .filter_map(|component| non_empty_trimmed(&component.name))
        .collect::<Vec<_>>();
    if component_names.is_empty() {
        return "No components".to_string();
    }

    let visible_count = component_names.len().min(3);
    let mut summary = component_names
        .iter()
        .take(visible_count)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let remaining_count = component_names.len().saturating_sub(visible_count);
    if remaining_count > 0 {
        summary.push_str(&format!(" +{remaining_count} more"));
    }
    summary
}

fn component_summary_from_component_choices(
    choices: &[radishflow_studio::StudioGuiProjectComponentChoiceSnapshot],
) -> String {
    let component_names = choices
        .iter()
        .filter(|component| component.selected)
        .filter_map(|component| non_empty_trimmed(&component.name))
        .collect::<Vec<_>>();
    if component_names.is_empty() {
        "No components".to_string()
    } else {
        summarize_visible_names(&component_names)
    }
}

fn summarize_visible_names(names: &[String]) -> String {
    let visible_count = names.len().min(3);
    let mut summary = names
        .iter()
        .take(visible_count)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let remaining_count = names.len().saturating_sub(visible_count);
    if remaining_count > 0 {
        summary.push_str(&format!(" +{remaining_count} more"));
    }
    summary
}

fn current_workspace_thumbnail(
    document: &radishflow_studio::StudioGuiWorkspaceDocumentSnapshot,
) -> radishflow_studio::StudioGuiWindowThumbnailFlowModel {
    let nodes = if document.unit_count > 0 {
        vec![format!("{} units", document.unit_count)]
    } else {
        Vec::new()
    };
    radishflow_studio::StudioGuiWindowThumbnailFlowModel {
        nodes,
        edges: Vec::new(),
    }
}

fn thumbnail_from_flowsheet(
    flowsheet: &rf_model::Flowsheet,
) -> radishflow_studio::StudioGuiWindowThumbnailFlowModel {
    let units = flowsheet.units.values().collect::<Vec<_>>();
    let nodes = units
        .iter()
        .map(|unit| unit_label_for_thumbnail(unit))
        .collect::<Vec<_>>();
    let mut stream_sources: BTreeMap<String, usize> = BTreeMap::new();
    let mut stream_sinks: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for (unit_index, unit) in units.iter().enumerate() {
        for port in &unit.ports {
            if port.kind != rf_types::PortKind::Material {
                continue;
            }
            let Some(stream_id) = port.stream_id.as_ref() else {
                continue;
            };
            match port.direction {
                rf_types::PortDirection::Outlet => {
                    stream_sources.insert(stream_id.to_string(), unit_index);
                }
                rf_types::PortDirection::Inlet => {
                    stream_sinks
                        .entry(stream_id.to_string())
                        .or_default()
                        .push(unit_index);
                }
            }
        }
    }

    let mut edges = Vec::new();
    for (stream_id, source_index) in stream_sources {
        let Some(sink_indices) = stream_sinks.get(&stream_id) else {
            continue;
        };
        for sink_index in sink_indices {
            let edge = (source_index, *sink_index);
            if !edges.contains(&edge) {
                edges.push(edge);
            }
        }
    }

    radishflow_studio::StudioGuiWindowThumbnailFlowModel { nodes, edges }
}

fn unit_label_for_thumbnail(unit: &rf_model::UnitNode) -> String {
    non_empty_trimmed(&unit.name).unwrap_or_else(|| readable_unit_kind(&unit.kind))
}

fn readable_unit_kind(kind: &str) -> String {
    let label = kind
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if label.is_empty() {
        "Unit".to_string()
    } else {
        label
    }
}

fn recent_case_tile_status(
    project_path: &Path,
    current_project_path: Option<&str>,
) -> radishflow_studio::StudioGuiWindowHomeCaseTileStatus {
    if !project_path.exists() {
        return radishflow_studio::StudioGuiWindowHomeCaseTileStatus::MissingFile;
    }

    if current_project_path.is_some_and(|current| paths_match(project_path, Path::new(current))) {
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current
    } else {
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Ready
    }
}

fn home_tile_targets_current_workspace(
    tile: &radishflow_studio::StudioGuiWindowHomeCaseTileModel,
    window: &StudioGuiWindowModel,
) -> bool {
    if tile.source == radishflow_studio::StudioGuiWindowHomeCaseTileSource::Current {
        return true;
    }

    tile.status == radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current
        && window
            .runtime
            .workspace_document
            .project_path
            .as_deref()
            .is_some_and(|current| paths_match(Path::new(&tile.path_text), Path::new(current)))
}

fn home_case_tile_status_label(
    status: radishflow_studio::StudioGuiWindowHomeCaseTileStatus,
) -> &'static str {
    match status {
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Ready => "Ready",
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current => "Current",
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::MissingFile => "Missing file",
    }
}

pub(in crate::studio_gui_shell) fn home_case_row_action(
    clicked: bool,
    double_clicked: bool,
) -> HomeCaseRowAction {
    if double_clicked {
        HomeCaseRowAction::Open
    } else if clicked {
        HomeCaseRowAction::Select
    } else {
        HomeCaseRowAction::None
    }
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
            "feed-heater-flash" => std::borrow::Cow::Borrowed("Heater"),
            "feed-valve-flash" => std::borrow::Cow::Borrowed("Valve"),
            "feed-cooler-flash" => std::borrow::Cow::Borrowed("Cooler"),
            "feed-mixer-flash" => std::borrow::Cow::Borrowed("Mixer"),
            "feed-mixer-heater-flash" => std::borrow::Cow::Borrowed("Synthetic mixer + heater"),
            "water-ethanol-heater-flash" => std::borrow::Cow::Borrowed("PME Sample"),
            _ => std::borrow::Cow::Borrowed(fallback),
        },
        StudioShellLocale::ZhCn => match id {
            "feed-heater-flash" => std::borrow::Cow::Borrowed("加热器"),
            "feed-valve-flash" => std::borrow::Cow::Borrowed("阀门"),
            "feed-cooler-flash" => std::borrow::Cow::Borrowed("冷却器"),
            "feed-mixer-flash" => std::borrow::Cow::Borrowed("混合器"),
            "feed-mixer-heater-flash" => std::borrow::Cow::Borrowed("Synthetic 混合加热"),
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

fn home_case_tile_status_color(
    status: radishflow_studio::StudioGuiWindowHomeCaseTileStatus,
) -> egui::Color32 {
    match status {
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Ready
        | radishflow_studio::StudioGuiWindowHomeCaseTileStatus::Current => {
            egui::Color32::from_rgb(52, 128, 89)
        }
        radishflow_studio::StudioGuiWindowHomeCaseTileStatus::MissingFile => {
            egui::Color32::from_rgb(180, 70, 60)
        }
    }
}

fn render_home_thumbnail(
    ui: &mut egui::Ui,
    thumbnail: &radishflow_studio::StudioGuiWindowThumbnailFlowModel,
) {
    if thumbnail.nodes.is_empty() {
        return;
    }

    let stages = home_thumbnail_stages(thumbnail);
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(244, 247, 251))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(218, 226, 238),
        ))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (stage_index, stage) in stages.iter().enumerate() {
                    if stage_index > 0 {
                        ui.small("->");
                    }
                    for node in stage {
                        render_status_chip(ui, node, egui::Color32::from_rgb(86, 118, 168));
                    }
                }
            });
        });
}

fn home_thumbnail_stages(
    thumbnail: &radishflow_studio::StudioGuiWindowThumbnailFlowModel,
) -> Vec<Vec<&str>> {
    let node_count = thumbnail.nodes.len();
    let mut incoming_counts = vec![0usize; node_count];
    for (_, to) in &thumbnail.edges {
        if *to < node_count {
            incoming_counts[*to] += 1;
        }
    }

    let mut consumed = vec![false; node_count];
    let mut stages = Vec::new();
    while consumed.iter().any(|consumed| !*consumed) {
        let stage_indices = incoming_counts
            .iter()
            .enumerate()
            .filter_map(|(index, incoming_count)| {
                if !consumed[index] && *incoming_count == 0 {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if stage_indices.is_empty() {
            return thumbnail
                .nodes
                .iter()
                .map(|node| vec![node.as_str()])
                .collect();
        }

        for index in &stage_indices {
            consumed[*index] = true;
            for (from, to) in &thumbnail.edges {
                if from == index && *to < node_count {
                    incoming_counts[*to] = incoming_counts[*to].saturating_sub(1);
                }
            }
        }
        stages.push(
            stage_indices
                .into_iter()
                .map(|index| thumbnail.nodes[index].as_str())
                .collect(),
        );
    }

    stages
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
            HomeText::ReturnWorkspace => "Return to Workspace",
            HomeText::ReturnWorkspaceDetail => "Continue the currently open case.",
            HomeText::NewBlankCase => "New Project",
            HomeText::AuthorMixerFlashCase => "Create Mixer-Flash Case",
            HomeText::AuthorMixerFlashDetail => {
                "Start from a blank project with a case authoring checklist."
            }
            HomeText::AuthorHeaterFlashCase => "Create Heater-Flash Case",
            HomeText::AuthorHeaterFlashDetail => {
                "Build a single-feed heater case with the same checklist flow."
            }
            HomeText::OpenCase => "Open Project",
            HomeText::OpenExampleCase => "Open Example Project",
            HomeText::RecentCases => "Recent Cases",
            HomeText::ExampleCases => "Example Cases",
            HomeText::NoRecentCases => "No recent cases yet.",
            HomeText::ChooseExampleTitle => "Choose an example",
            HomeText::ChooseExampleDetail => {
                "Use the Example Cases section to open a bundled case."
            }
            HomeText::LastOpenedMru => "Last opened: MRU",
            HomeText::CurrentWorkspace => "Current workspace",
            HomeText::PropertyPackage => "Property Package:",
            HomeText::Components => "Components:",
            HomeText::Environment => "Environment",
            HomeText::Client => "Client",
            HomeText::Studio => "Studio",
            HomeText::DevelopmentBuild => "development build",
            HomeText::Mode => "Mode",
            HomeText::Examples => "Examples",
            HomeText::PortableInternal => "Portable / development",
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
            HomeText::ReturnWorkspace => "返回工作区",
            HomeText::ReturnWorkspaceDetail => "继续当前已打开项目。",
            HomeText::NewBlankCase => "新建项目",
            HomeText::AuthorMixerFlashCase => "创建 Mixer-Flash 小案例",
            HomeText::AuthorMixerFlashDetail => "从空白项目开始，并打开模块任务清单。",
            HomeText::AuthorHeaterFlashCase => "创建 Heater-Flash 小案例",
            HomeText::AuthorHeaterFlashDetail => "单 Feed 加热后进入 Flash Drum 的作者路径。",
            HomeText::OpenCase => "打开项目",
            HomeText::OpenExampleCase => "打开示例项目",
            HomeText::RecentCases => "最近项目",
            HomeText::ExampleCases => "示例项目",
            HomeText::NoRecentCases => "还没有最近项目。",
            HomeText::ChooseExampleTitle => "请选择示例",
            HomeText::ChooseExampleDetail => "从示例项目区域打开一个内置示例。",
            HomeText::LastOpenedMru => "上次打开",
            HomeText::CurrentWorkspace => "当前工作区",
            HomeText::PropertyPackage => "物性包:",
            HomeText::Components => "组分:",
            HomeText::Environment => "环境",
            HomeText::Client => "客户端",
            HomeText::Studio => "Studio",
            HomeText::DevelopmentBuild => "开发构建",
            HomeText::Mode => "模式",
            HomeText::Examples => "示例",
            HomeText::PortableInternal => "便携 / 开发",
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
            HomeText::AuthMessage => "尚未登录。云端物性包和团队项目暂不可用。",
            HomeText::ExamplesReadyMessage => "内置示例已在本地可用。",
            HomeText::ExamplesMissingMessage => "未发现内置示例。",
            HomeText::CacheReadyMessage => "本地物性包缓存已就绪。",
        },
    }
}
