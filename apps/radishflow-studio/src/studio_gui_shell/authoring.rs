use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MixerFlashAuthoringText {
    Title,
    Detail,
    TwoFeeds,
    FeedOutlets,
    MixerPlaced,
    FeedToMixer,
    MixerOutlet,
    FlashPlaced,
    MixerToFlash,
    FlashOutlets,
    RunCase,
    Done,
    Todo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MixerFlashAuthoringTask {
    label: &'static str,
    complete: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct MixerFlashAuthoringProgress {
    feed_units: usize,
    feed_outlet_streams: usize,
    has_mixer: bool,
    feed_to_mixer_streams: usize,
    has_mixer_outlet_stream: bool,
    has_flash: bool,
    has_mixer_to_flash_stream: bool,
    flash_outlet_streams: usize,
    has_solve_snapshot: bool,
}

impl ReadyAppState {
    pub(super) fn render_mixer_flash_authoring_checklist(
        &self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        ui.separator();
        ui.label(
            egui::RichText::new(mixer_flash_authoring_text(
                self.locale,
                MixerFlashAuthoringText::Title,
            ))
            .strong(),
        );
        render_wrapped_small(
            ui,
            mixer_flash_authoring_text(self.locale, MixerFlashAuthoringText::Detail),
        );
        ui.add_space(4.0);

        for task in mixer_flash_authoring_tasks(self.locale, window) {
            let (status, color) = if task.complete {
                (
                    mixer_flash_authoring_text(self.locale, MixerFlashAuthoringText::Done),
                    egui::Color32::from_rgb(52, 128, 89),
                )
            } else {
                (
                    mixer_flash_authoring_text(self.locale, MixerFlashAuthoringText::Todo),
                    egui::Color32::from_rgb(86, 96, 108),
                )
            };
            ui.horizontal_wrapped(|ui| {
                render_status_chip(ui, status, color);
                ui.label(task.label);
            });
        }
    }
}

fn mixer_flash_authoring_tasks(
    locale: StudioShellLocale,
    window: &StudioGuiWindowModel,
) -> Vec<MixerFlashAuthoringTask> {
    let progress = mixer_flash_authoring_progress(window);

    vec![
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::TwoFeeds),
            complete: progress.feed_units >= 2,
        },
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::FeedOutlets),
            complete: progress.feed_outlet_streams >= 2,
        },
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::MixerPlaced),
            complete: progress.has_mixer,
        },
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::FeedToMixer),
            complete: progress.feed_to_mixer_streams >= 2,
        },
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::MixerOutlet),
            complete: progress.has_mixer_outlet_stream,
        },
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::FlashPlaced),
            complete: progress.has_flash,
        },
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::MixerToFlash),
            complete: progress.has_mixer_to_flash_stream,
        },
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::FlashOutlets),
            complete: progress.flash_outlet_streams >= 2,
        },
        MixerFlashAuthoringTask {
            label: mixer_flash_authoring_text(locale, MixerFlashAuthoringText::RunCase),
            complete: progress.has_solve_snapshot,
        },
    ]
}

fn mixer_flash_authoring_progress(window: &StudioGuiWindowModel) -> MixerFlashAuthoringProgress {
    let view = window.canvas.widget.view();
    let feed_unit_ids = unit_ids_by_kind(view, "feed");
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

    let feed_to_mixer_streams = mixer_unit_id
        .as_ref()
        .map(|mixer_id| {
            view.stream_lines
                .iter()
                .filter(|stream| {
                    let from_feed = stream.source.as_ref().is_some_and(|source| {
                        feed_unit_ids
                            .iter()
                            .any(|unit_id| unit_id == &source.unit_id)
                    });
                    let to_mixer = stream
                        .sink
                        .as_ref()
                        .is_some_and(|sink| &sink.unit_id == mixer_id);
                    from_feed && to_mixer
                })
                .count()
        })
        .unwrap_or(0);

    let has_mixer_outlet_stream = mixer_unit_id
        .as_ref()
        .is_some_and(|mixer_id| has_stream_from_unit(view, mixer_id));

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

    MixerFlashAuthoringProgress {
        feed_units: feed_unit_ids.len(),
        feed_outlet_streams,
        has_mixer: mixer_unit_id.is_some(),
        feed_to_mixer_streams,
        has_mixer_outlet_stream,
        has_flash: flash_unit_id.is_some(),
        has_mixer_to_flash_stream,
        flash_outlet_streams,
        has_solve_snapshot: window.runtime.latest_solve_snapshot.is_some(),
    }
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

fn mixer_flash_authoring_text(
    locale: StudioShellLocale,
    key: MixerFlashAuthoringText,
) -> &'static str {
    match locale {
        StudioShellLocale::En => match key {
            MixerFlashAuthoringText::Title => "Mixer-Flash Case",
            MixerFlashAuthoringText::Detail => {
                "Follow this checklist to build Feed + Feed -> Mixer -> Flash Drum from a blank project."
            }
            MixerFlashAuthoringText::TwoFeeds => "Place two Feed units",
            MixerFlashAuthoringText::FeedOutlets => "Create both Feed outlet streams",
            MixerFlashAuthoringText::MixerPlaced => "Place a Mixer",
            MixerFlashAuthoringText::FeedToMixer => "Connect both Feed outlets into the Mixer",
            MixerFlashAuthoringText::MixerOutlet => "Create the Mixer outlet stream",
            MixerFlashAuthoringText::FlashPlaced => "Place a Flash Drum",
            MixerFlashAuthoringText::MixerToFlash => "Connect the Mixer outlet into the Flash Drum",
            MixerFlashAuthoringText::FlashOutlets => "Create Flash Drum liquid and vapor outlets",
            MixerFlashAuthoringText::RunCase => "Run the case and review results",
            MixerFlashAuthoringText::Done => "done",
            MixerFlashAuthoringText::Todo => "todo",
        },
        StudioShellLocale::ZhCn => match key {
            MixerFlashAuthoringText::Title => "Mixer-Flash 小案例",
            MixerFlashAuthoringText::Detail => {
                "从空白项目开始，按清单完成 Feed + Feed -> Mixer -> Flash Drum 建模。"
            }
            MixerFlashAuthoringText::TwoFeeds => "放置两个 Feed",
            MixerFlashAuthoringText::FeedOutlets => "创建两个 Feed 出口流股",
            MixerFlashAuthoringText::MixerPlaced => "放置 Mixer",
            MixerFlashAuthoringText::FeedToMixer => "连接两个 Feed 出口到 Mixer",
            MixerFlashAuthoringText::MixerOutlet => "创建 Mixer 出口流股",
            MixerFlashAuthoringText::FlashPlaced => "放置 Flash Drum",
            MixerFlashAuthoringText::MixerToFlash => "连接 Mixer 出口到 Flash Drum",
            MixerFlashAuthoringText::FlashOutlets => "创建 Flash Drum 液相和气相出口",
            MixerFlashAuthoringText::RunCase => "运行案例并检查结果",
            MixerFlashAuthoringText::Done => "完成",
            MixerFlashAuthoringText::Todo => "待做",
        },
    }
}
