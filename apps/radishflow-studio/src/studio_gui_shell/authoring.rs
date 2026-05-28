use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthoringTaskKey {
    PropertyPackageSelected,
    ProjectComponentsSelected,
    OneFeed,
    TwoFeeds,
    FeedOutlet,
    FeedOutlets,
    FeedComposition,
    FeedCompositions,
    FeedParameters,
    FeedParametersForBoth,
    HeaterPlaced,
    FeedToHeater,
    HeaterOutlet,
    HeaterParameters,
    MixerPlaced,
    FeedToMixer,
    MixerOutlet,
    MixerParameters,
    FlashPlaced,
    HeaterToFlash,
    MixerToFlash,
    FlashOutlets,
    FlashParameters,
    RunCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AuthoringTask {
    key: AuthoringTaskKey,
    complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthoringRunBlocker {
    case: AuthoringCaseKind,
    task_key: AuthoringTaskKey,
    focus_target: AuthoringFocusTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AuthoringFocusTarget {
    Package,
    Palette,
    Stream(String),
    Unit(String),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct AuthoringProgress {
    feed_units: usize,
    feed_outlet_streams: usize,
    has_property_package: bool,
    selected_components: usize,
    feed_outlet_streams_with_composition: usize,
    feed_units_with_source_parameters: usize,
    has_heater: bool,
    feed_to_heater_streams: usize,
    has_heater_outlet_stream: bool,
    has_heater_parameters: bool,
    has_mixer: bool,
    feed_to_mixer_streams: usize,
    has_mixer_outlet_stream: bool,
    has_mixer_parameters: bool,
    has_flash: bool,
    has_heater_to_flash_stream: bool,
    has_mixer_to_flash_stream: bool,
    flash_outlet_streams: usize,
    has_flash_parameters: bool,
    has_solve_snapshot: bool,
}

impl ReadyAppState {
    pub(super) fn render_authoring_checklists(
        &self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        let Some(case) = self.active_authoring_case else {
            return;
        };

        let document = self.platform_host.document();
        ui.separator();
        self.render_authoring_checklist(ui, window, document, case);
    }

    fn render_authoring_checklist(
        &self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        document: &rf_ui::FlowsheetDocument,
        case: AuthoringCaseKind,
    ) {
        ui.label(egui::RichText::new(authoring_case_title(self.locale, case)).strong());
        render_wrapped_small(ui, authoring_case_detail(self.locale, case));
        ui.add_space(4.0);

        for task in authoring_tasks(case, window, document) {
            let (status, color) = if task.complete {
                (
                    authoring_status_text(self.locale, true),
                    egui::Color32::from_rgb(52, 128, 89),
                )
            } else {
                (
                    authoring_status_text(self.locale, false),
                    egui::Color32::from_rgb(86, 96, 108),
                )
            };
            ui.horizontal_wrapped(|ui| {
                render_status_chip(ui, status, color);
                ui.label(authoring_task_label(self.locale, task.key));
            });
        }
    }

    pub(super) fn intercept_authoring_run_if_needed(&mut self, command_id: &str) -> bool {
        if command_id != "run_panel.run_manual" {
            return false;
        }

        let Some(case) = self.active_authoring_case else {
            return false;
        };

        let window = self.platform_host.snapshot().window_model();
        let blocker = {
            let document = self.platform_host.document();
            authoring_run_blocker(case, &window, document)
        };
        let Some(blocker) = blocker else {
            return false;
        };

        self.focus_authoring_run_blocker(&blocker);
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Warning,
            title: authoring_run_blocked_title(self.locale).to_string(),
            detail: authoring_run_blocked_detail(self.locale, blocker.case, blocker.task_key),
        });
        self.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
        self.platform_host.record_activity_line(format!(
            "blocked authoring run before solve: {:?} {:?}",
            blocker.case, blocker.task_key
        ));
        true
    }

    fn focus_authoring_run_blocker(&mut self, blocker: &AuthoringRunBlocker) {
        match &blocker.focus_target {
            AuthoringFocusTarget::Package => {
                self.left_sidebar_tab = StudioShellLeftSidebarTab::Project;
                self.right_sidebar_tab = StudioShellRightSidebarTab::Package;
            }
            AuthoringFocusTarget::Palette => {
                self.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            }
            AuthoringFocusTarget::Stream(stream_id) => {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                self.dispatch_ui_command(format!("inspector.focus_stream:{stream_id}"));
            }
            AuthoringFocusTarget::Unit(unit_id) => {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                self.dispatch_ui_command(format!("inspector.focus_unit:{unit_id}"));
            }
        }
    }
}

fn authoring_run_blocker(
    case: AuthoringCaseKind,
    window: &StudioGuiWindowModel,
    document: &rf_ui::FlowsheetDocument,
) -> Option<AuthoringRunBlocker> {
    let progress = authoring_progress(window, document);

    authoring_tasks_with_progress(case, &progress)
        .into_iter()
        .find(|task| task.key != AuthoringTaskKey::RunCase && !task.complete)
        .map(|task| AuthoringRunBlocker {
            case,
            task_key: task.key,
            focus_target: authoring_task_focus_target(task.key, window, document),
        })
}

fn authoring_tasks(
    case: AuthoringCaseKind,
    window: &StudioGuiWindowModel,
    document: &rf_ui::FlowsheetDocument,
) -> Vec<AuthoringTask> {
    let progress = authoring_progress(window, document);
    authoring_tasks_with_progress(case, &progress)
}

fn authoring_tasks_with_progress(
    case: AuthoringCaseKind,
    progress: &AuthoringProgress,
) -> Vec<AuthoringTask> {
    match case {
        AuthoringCaseKind::MixerFlash => vec![
            AuthoringTask {
                key: AuthoringTaskKey::PropertyPackageSelected,
                complete: progress.has_property_package,
            },
            AuthoringTask {
                key: AuthoringTaskKey::ProjectComponentsSelected,
                complete: progress.selected_components >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::TwoFeeds,
                complete: progress.feed_units >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FeedOutlets,
                complete: progress.feed_outlet_streams >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FeedCompositions,
                complete: progress.feed_outlet_streams_with_composition >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FeedParametersForBoth,
                complete: progress.feed_units_with_source_parameters >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::MixerPlaced,
                complete: progress.has_mixer,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FeedToMixer,
                complete: progress.feed_to_mixer_streams >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::MixerOutlet,
                complete: progress.has_mixer_outlet_stream,
            },
            AuthoringTask {
                key: AuthoringTaskKey::MixerParameters,
                complete: progress.has_mixer_parameters,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FlashPlaced,
                complete: progress.has_flash,
            },
            AuthoringTask {
                key: AuthoringTaskKey::MixerToFlash,
                complete: progress.has_mixer_to_flash_stream,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FlashOutlets,
                complete: progress.flash_outlet_streams >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FlashParameters,
                complete: progress.has_flash_parameters,
            },
            AuthoringTask {
                key: AuthoringTaskKey::RunCase,
                complete: progress.has_solve_snapshot,
            },
        ],
        AuthoringCaseKind::HeaterFlash => vec![
            AuthoringTask {
                key: AuthoringTaskKey::PropertyPackageSelected,
                complete: progress.has_property_package,
            },
            AuthoringTask {
                key: AuthoringTaskKey::ProjectComponentsSelected,
                complete: progress.selected_components >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::OneFeed,
                complete: progress.feed_units >= 1,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FeedOutlet,
                complete: progress.feed_outlet_streams >= 1,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FeedComposition,
                complete: progress.feed_outlet_streams_with_composition >= 1,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FeedParameters,
                complete: progress.feed_units_with_source_parameters >= 1,
            },
            AuthoringTask {
                key: AuthoringTaskKey::HeaterPlaced,
                complete: progress.has_heater,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FeedToHeater,
                complete: progress.feed_to_heater_streams >= 1,
            },
            AuthoringTask {
                key: AuthoringTaskKey::HeaterOutlet,
                complete: progress.has_heater_outlet_stream,
            },
            AuthoringTask {
                key: AuthoringTaskKey::HeaterParameters,
                complete: progress.has_heater_parameters,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FlashPlaced,
                complete: progress.has_flash,
            },
            AuthoringTask {
                key: AuthoringTaskKey::HeaterToFlash,
                complete: progress.has_heater_to_flash_stream,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FlashOutlets,
                complete: progress.flash_outlet_streams >= 2,
            },
            AuthoringTask {
                key: AuthoringTaskKey::FlashParameters,
                complete: progress.has_flash_parameters,
            },
            AuthoringTask {
                key: AuthoringTaskKey::RunCase,
                complete: progress.has_solve_snapshot,
            },
        ],
    }
}

fn authoring_task_focus_target(
    key: AuthoringTaskKey,
    window: &StudioGuiWindowModel,
    document: &rf_ui::FlowsheetDocument,
) -> AuthoringFocusTarget {
    match key {
        AuthoringTaskKey::PropertyPackageSelected | AuthoringTaskKey::ProjectComponentsSelected => {
            AuthoringFocusTarget::Package
        }
        AuthoringTaskKey::FeedComposition | AuthoringTaskKey::FeedCompositions => {
            first_feed_outlet_stream_missing_composition(window, document)
                .map(AuthoringFocusTarget::Stream)
                .unwrap_or(AuthoringFocusTarget::Palette)
        }
        AuthoringTaskKey::FeedParameters | AuthoringTaskKey::FeedParametersForBoth => {
            first_feed_missing_source_parameters(window, document)
                .map(AuthoringFocusTarget::Unit)
                .unwrap_or(AuthoringFocusTarget::Palette)
        }
        AuthoringTaskKey::HeaterParameters => {
            first_unit_id_by_kind(window.canvas.widget.view(), "heater")
                .map(AuthoringFocusTarget::Unit)
                .unwrap_or(AuthoringFocusTarget::Palette)
        }
        AuthoringTaskKey::MixerParameters => {
            first_unit_id_by_kind(window.canvas.widget.view(), "mixer")
                .map(AuthoringFocusTarget::Unit)
                .unwrap_or(AuthoringFocusTarget::Palette)
        }
        AuthoringTaskKey::FlashParameters => {
            first_unit_id_by_kind(window.canvas.widget.view(), "flash_drum")
                .map(AuthoringFocusTarget::Unit)
                .unwrap_or(AuthoringFocusTarget::Palette)
        }
        AuthoringTaskKey::OneFeed
        | AuthoringTaskKey::TwoFeeds
        | AuthoringTaskKey::FeedOutlet
        | AuthoringTaskKey::FeedOutlets
        | AuthoringTaskKey::HeaterPlaced
        | AuthoringTaskKey::FeedToHeater
        | AuthoringTaskKey::HeaterOutlet
        | AuthoringTaskKey::MixerPlaced
        | AuthoringTaskKey::FeedToMixer
        | AuthoringTaskKey::MixerOutlet
        | AuthoringTaskKey::FlashPlaced
        | AuthoringTaskKey::HeaterToFlash
        | AuthoringTaskKey::MixerToFlash
        | AuthoringTaskKey::FlashOutlets
        | AuthoringTaskKey::RunCase => AuthoringFocusTarget::Palette,
    }
}

fn authoring_progress(
    window: &StudioGuiWindowModel,
    document: &rf_ui::FlowsheetDocument,
) -> AuthoringProgress {
    let view = window.canvas.widget.view();
    let feed_unit_ids = unit_ids_by_kind(view, "feed");
    let heater_unit_id = first_unit_id_by_kind(view, "heater");
    let mixer_unit_id = first_unit_id_by_kind(view, "mixer");
    let flash_unit_id = first_unit_id_by_kind(view, "flash_drum");

    let feed_outlet_streams = view
        .stream_lines
        .iter()
        .filter(|stream| {
            stream.source.as_ref().is_some_and(|source| {
                feed_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == &source.unit_id)
            })
        })
        .count();
    let feed_outlet_stream_ids = feed_outlet_stream_ids(view, &feed_unit_ids);
    let feed_outlet_streams_with_composition =
        count_streams_with_composition(document, &feed_outlet_stream_ids);

    let feed_to_heater_streams = heater_unit_id
        .as_ref()
        .map(|heater_id| count_streams_from_any_to_unit(view, &feed_unit_ids, heater_id))
        .unwrap_or(0);

    let feed_to_mixer_streams = mixer_unit_id
        .as_ref()
        .map(|mixer_id| count_streams_from_any_to_unit(view, &feed_unit_ids, mixer_id))
        .unwrap_or(0);

    let has_heater_outlet_stream = heater_unit_id
        .as_ref()
        .is_some_and(|heater_id| has_stream_from_unit(view, heater_id));

    let has_mixer_outlet_stream = mixer_unit_id
        .as_ref()
        .is_some_and(|mixer_id| has_stream_from_unit(view, mixer_id));

    let has_heater_to_flash_stream = heater_unit_id
        .as_ref()
        .zip(flash_unit_id.as_ref())
        .is_some_and(|(heater_id, flash_id)| has_stream_between_units(view, heater_id, flash_id));

    let has_mixer_to_flash_stream = mixer_unit_id
        .as_ref()
        .zip(flash_unit_id.as_ref())
        .is_some_and(|(mixer_id, flash_id)| has_stream_between_units(view, mixer_id, flash_id));

    let flash_outlet_streams = flash_unit_id
        .as_ref()
        .map(|flash_id| {
            view.stream_lines
                .iter()
                .filter(|stream| {
                    stream
                        .source
                        .as_ref()
                        .is_some_and(|source| &source.unit_id == flash_id)
                })
                .count()
        })
        .unwrap_or(0);

    AuthoringProgress {
        feed_units: feed_unit_ids.len(),
        feed_outlet_streams,
        has_property_package: document.flowsheet.property_package_id().is_some(),
        selected_components: document.flowsheet.components.len(),
        feed_outlet_streams_with_composition,
        feed_units_with_source_parameters: count_units_with_source_parameters(
            document,
            &feed_unit_ids,
        ),
        has_heater: heater_unit_id.is_some(),
        feed_to_heater_streams,
        has_heater_outlet_stream,
        has_heater_parameters: heater_unit_id
            .as_ref()
            .is_some_and(|unit_id| unit_has_temperature_and_pressure_parameters(document, unit_id)),
        has_mixer: mixer_unit_id.is_some(),
        feed_to_mixer_streams,
        has_mixer_outlet_stream,
        has_mixer_parameters: mixer_unit_id
            .as_ref()
            .is_some_and(|unit_id| unit_has_pressure_parameter(document, unit_id)),
        has_flash: flash_unit_id.is_some(),
        has_heater_to_flash_stream,
        has_mixer_to_flash_stream,
        flash_outlet_streams,
        has_flash_parameters: flash_unit_id
            .as_ref()
            .is_some_and(|unit_id| unit_has_temperature_and_pressure_parameters(document, unit_id)),
        has_solve_snapshot: window.runtime.latest_solve_snapshot.is_some(),
    }
}

fn first_feed_outlet_stream_missing_composition(
    window: &StudioGuiWindowModel,
    document: &rf_ui::FlowsheetDocument,
) -> Option<String> {
    let view = window.canvas.widget.view();
    let feed_unit_ids = unit_ids_by_kind(view, "feed");
    feed_outlet_stream_ids(view, &feed_unit_ids)
        .into_iter()
        .find(|stream_id| {
            document
                .flowsheet
                .streams
                .get(&rf_types::StreamId::new(stream_id.as_str()))
                .is_none_or(|stream| stream.overall_mole_fractions.is_empty())
        })
}

fn first_feed_missing_source_parameters(
    window: &StudioGuiWindowModel,
    document: &rf_ui::FlowsheetDocument,
) -> Option<String> {
    unit_ids_by_kind(window.canvas.widget.view(), "feed")
        .into_iter()
        .find(|unit_id| !unit_has_temperature_and_pressure_parameters(document, unit_id))
}

fn count_streams_from_any_to_unit(
    view: &radishflow_studio::StudioGuiCanvasViewModel,
    source_unit_ids: &[String],
    sink_unit_id: &str,
) -> usize {
    view.stream_lines
        .iter()
        .filter(|stream| {
            let from_source = stream.source.as_ref().is_some_and(|source| {
                source_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == &source.unit_id)
            });
            let to_sink = stream
                .sink
                .as_ref()
                .is_some_and(|sink| sink.unit_id.as_str() == sink_unit_id);
            from_source && to_sink
        })
        .count()
}

fn feed_outlet_stream_ids(
    view: &radishflow_studio::StudioGuiCanvasViewModel,
    feed_unit_ids: &[String],
) -> Vec<String> {
    view.stream_lines
        .iter()
        .filter(|stream| {
            stream.source.as_ref().is_some_and(|source| {
                feed_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == &source.unit_id)
            })
        })
        .map(|stream| stream.stream_id.clone())
        .collect()
}

fn count_streams_with_composition(
    document: &rf_ui::FlowsheetDocument,
    stream_ids: &[String],
) -> usize {
    stream_ids
        .iter()
        .filter(|stream_id| {
            document
                .flowsheet
                .streams
                .get(&rf_types::StreamId::new(stream_id.as_str()))
                .is_some_and(|stream| !stream.overall_mole_fractions.is_empty())
        })
        .count()
}

fn count_units_with_source_parameters(
    document: &rf_ui::FlowsheetDocument,
    unit_ids: &[String],
) -> usize {
    unit_ids
        .iter()
        .filter(|unit_id| unit_has_temperature_and_pressure_parameters(document, unit_id))
        .count()
}

fn unit_has_temperature_and_pressure_parameters(
    document: &rf_ui::FlowsheetDocument,
    unit_id: &str,
) -> bool {
    document
        .flowsheet
        .units
        .get(&rf_types::UnitId::new(unit_id))
        .is_some_and(|unit| {
            unit.parameters.outlet_temperature_k.is_some()
                && unit.parameters.outlet_pressure_pa.is_some()
        })
}

fn unit_has_pressure_parameter(document: &rf_ui::FlowsheetDocument, unit_id: &str) -> bool {
    document
        .flowsheet
        .units
        .get(&rf_types::UnitId::new(unit_id))
        .is_some_and(|unit| unit.parameters.outlet_pressure_pa.is_some())
}

fn unit_ids_by_kind(view: &radishflow_studio::StudioGuiCanvasViewModel, kind: &str) -> Vec<String> {
    view.unit_blocks
        .iter()
        .filter(|unit| unit.kind == kind)
        .map(|unit| unit.unit_id.clone())
        .collect()
}

fn first_unit_id_by_kind(
    view: &radishflow_studio::StudioGuiCanvasViewModel,
    kind: &str,
) -> Option<String> {
    view.unit_blocks
        .iter()
        .find(|unit| unit.kind == kind)
        .map(|unit| unit.unit_id.clone())
}

fn has_stream_from_unit(view: &radishflow_studio::StudioGuiCanvasViewModel, unit_id: &str) -> bool {
    view.stream_lines.iter().any(|stream| {
        stream
            .source
            .as_ref()
            .is_some_and(|source| source.unit_id.as_str() == unit_id)
    })
}

fn has_stream_between_units(
    view: &radishflow_studio::StudioGuiCanvasViewModel,
    source_unit_id: &str,
    sink_unit_id: &str,
) -> bool {
    view.stream_lines.iter().any(|stream| {
        let source_matches = stream
            .source
            .as_ref()
            .is_some_and(|source| source.unit_id.as_str() == source_unit_id);
        let sink_matches = stream
            .sink
            .as_ref()
            .is_some_and(|sink| sink.unit_id.as_str() == sink_unit_id);
        source_matches && sink_matches
    })
}

fn authoring_case_title(locale: StudioShellLocale, case: AuthoringCaseKind) -> &'static str {
    match (locale, case) {
        (StudioShellLocale::En, AuthoringCaseKind::MixerFlash) => "Mixer-Flash Case",
        (StudioShellLocale::En, AuthoringCaseKind::HeaterFlash) => "Heater-Flash Case",
        (StudioShellLocale::ZhCn, AuthoringCaseKind::MixerFlash) => "Mixer-Flash 小案例",
        (StudioShellLocale::ZhCn, AuthoringCaseKind::HeaterFlash) => "Heater-Flash 小案例",
    }
}

fn authoring_case_detail(locale: StudioShellLocale, case: AuthoringCaseKind) -> &'static str {
    match (locale, case) {
        (StudioShellLocale::En, AuthoringCaseKind::MixerFlash) => {
            "Follow this checklist to build Feed + Feed -> Mixer -> Flash Drum from a blank project."
        }
        (StudioShellLocale::En, AuthoringCaseKind::HeaterFlash) => {
            "Follow this checklist to build Feed -> Heater -> Flash Drum from a blank project."
        }
        (StudioShellLocale::ZhCn, AuthoringCaseKind::MixerFlash) => {
            "从空白项目开始，按清单完成 Feed + Feed -> Mixer -> Flash Drum 建模。"
        }
        (StudioShellLocale::ZhCn, AuthoringCaseKind::HeaterFlash) => {
            "从空白项目开始，按清单完成 Feed -> Heater -> Flash Drum 建模。"
        }
    }
}

fn authoring_task_label(locale: StudioShellLocale, key: AuthoringTaskKey) -> &'static str {
    match locale {
        StudioShellLocale::En => match key {
            AuthoringTaskKey::PropertyPackageSelected => "Select the property package",
            AuthoringTaskKey::ProjectComponentsSelected => "Select methane and ethane",
            AuthoringTaskKey::OneFeed => "Place one Feed",
            AuthoringTaskKey::TwoFeeds => "Place two Feed units",
            AuthoringTaskKey::FeedOutlet => "Create the Feed outlet stream",
            AuthoringTaskKey::FeedOutlets => "Create both Feed outlet streams",
            AuthoringTaskKey::FeedComposition => "Commit the Feed composition",
            AuthoringTaskKey::FeedCompositions => "Commit both Feed compositions",
            AuthoringTaskKey::FeedParameters => "Commit Feed temperature and pressure",
            AuthoringTaskKey::FeedParametersForBoth => {
                "Commit both Feed temperatures and pressures"
            }
            AuthoringTaskKey::HeaterPlaced => "Place a Heater",
            AuthoringTaskKey::FeedToHeater => "Connect the Feed outlet into the Heater",
            AuthoringTaskKey::HeaterOutlet => "Create the Heater outlet stream",
            AuthoringTaskKey::HeaterParameters => "Commit Heater outlet temperature and pressure",
            AuthoringTaskKey::MixerPlaced => "Place a Mixer",
            AuthoringTaskKey::FeedToMixer => "Connect both Feed outlets into the Mixer",
            AuthoringTaskKey::MixerOutlet => "Create the Mixer outlet stream",
            AuthoringTaskKey::MixerParameters => "Commit Mixer outlet pressure",
            AuthoringTaskKey::FlashPlaced => "Place a Flash Drum",
            AuthoringTaskKey::HeaterToFlash => "Connect the Heater outlet into the Flash Drum",
            AuthoringTaskKey::MixerToFlash => "Connect the Mixer outlet into the Flash Drum",
            AuthoringTaskKey::FlashOutlets => "Create Flash Drum liquid and vapor outlets",
            AuthoringTaskKey::FlashParameters => "Commit Flash Drum temperature and pressure",
            AuthoringTaskKey::RunCase => "Run the case and review results",
        },
        StudioShellLocale::ZhCn => match key {
            AuthoringTaskKey::PropertyPackageSelected => "选择物性包",
            AuthoringTaskKey::ProjectComponentsSelected => "选择 methane 和 ethane",
            AuthoringTaskKey::OneFeed => "放置一个 Feed",
            AuthoringTaskKey::TwoFeeds => "放置两个 Feed",
            AuthoringTaskKey::FeedOutlet => "创建 Feed 出口流股",
            AuthoringTaskKey::FeedOutlets => "创建两个 Feed 出口流股",
            AuthoringTaskKey::FeedComposition => "提交 Feed 组成",
            AuthoringTaskKey::FeedCompositions => "提交两个 Feed 组成",
            AuthoringTaskKey::FeedParameters => "提交 Feed 温度和压力",
            AuthoringTaskKey::FeedParametersForBoth => "提交两个 Feed 温度和压力",
            AuthoringTaskKey::HeaterPlaced => "放置 Heater",
            AuthoringTaskKey::FeedToHeater => "连接 Feed 出口到 Heater",
            AuthoringTaskKey::HeaterOutlet => "创建 Heater 出口流股",
            AuthoringTaskKey::HeaterParameters => "提交 Heater 出口温度和压力",
            AuthoringTaskKey::MixerPlaced => "放置 Mixer",
            AuthoringTaskKey::FeedToMixer => "连接两个 Feed 出口到 Mixer",
            AuthoringTaskKey::MixerOutlet => "创建 Mixer 出口流股",
            AuthoringTaskKey::MixerParameters => "提交 Mixer 出口压力",
            AuthoringTaskKey::FlashPlaced => "放置 Flash Drum",
            AuthoringTaskKey::HeaterToFlash => "连接 Heater 出口到 Flash Drum",
            AuthoringTaskKey::MixerToFlash => "连接 Mixer 出口到 Flash Drum",
            AuthoringTaskKey::FlashOutlets => "创建 Flash Drum 液相和气相出口",
            AuthoringTaskKey::FlashParameters => "提交 Flash Drum 温度和压力",
            AuthoringTaskKey::RunCase => "运行案例并检查结果",
        },
    }
}

fn authoring_status_text(locale: StudioShellLocale, complete: bool) -> &'static str {
    match (locale, complete) {
        (StudioShellLocale::En, true) => "done",
        (StudioShellLocale::En, false) => "todo",
        (StudioShellLocale::ZhCn, true) => "完成",
        (StudioShellLocale::ZhCn, false) => "待做",
    }
}

fn authoring_run_blocked_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Case inputs are not ready",
        StudioShellLocale::ZhCn => "小案例输入未完成",
    }
}

fn authoring_run_blocked_detail(
    locale: StudioShellLocale,
    case: AuthoringCaseKind,
    task_key: AuthoringTaskKey,
) -> String {
    match locale {
        StudioShellLocale::En => format!(
            "Finish `{}` before running the {} checklist.",
            authoring_task_label(locale, task_key),
            authoring_case_title(locale, case)
        ),
        StudioShellLocale::ZhCn => format!(
            "先完成“{}”，再运行 {}。",
            authoring_task_label(locale, task_key),
            authoring_case_title(locale, case)
        ),
    }
}
