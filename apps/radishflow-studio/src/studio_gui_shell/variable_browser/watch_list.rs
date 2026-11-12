use super::*;
use rf_ui::{
    WorkspaceState,
    project_query::ProjectQuery,
    variable_browser::{
        VariableSection, VariableWatchCommand, VariableWatchDirection, VariableWatchOutcome,
    },
};

#[derive(Default)]
pub(super) struct WatchViewState {
    pub(super) selected: Option<String>,
    pub(super) detail_page: bool,
    notice: Option<String>,
    focus: Option<WatchFocus>,
    reveal_selection: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WatchFocus {
    Row,
    Heading,
    Move(VariableWatchDirection),
}

impl WatchViewState {
    pub(super) fn back_to_list(&mut self) {
        self.detail_page = false;
        self.focus = self.selected.as_ref().map(|_| WatchFocus::Row);
    }
}

impl VariableBrowserState {
    pub(super) fn show_watched(&mut self, reference: String) {
        self.directory = DirectoryView::WatchList;
        self.watch.selected = Some(reference);
        self.watch.detail_page = false;
        self.watch.focus = Some(WatchFocus::Row);
        self.watch.reveal_selection = true;
        self.notice = None;
    }
}

impl ReadyAppState {
    pub(super) fn edit_browser_watch_list(&mut self, command: VariableWatchCommand) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if self.read_only_project.is_some() || !self.variable_browser.belongs_to(workspace) {
            return;
        }
        let Some(window_id) = self.current_window_id() else {
            return;
        };
        let move_direction = match &command {
            VariableWatchCommand::Move { direction, .. } => Some(*direction),
            _ => None,
        };
        let dispatch = match self.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::VariableWatch(command),
        }) {
            Ok(dispatch) => dispatch,
            Err(error) => {
                self.variable_browser.notice = Some(error.to_string());
                return;
            }
        };
        let Some(radishflow_studio::StudioRuntimeDispatch::VariableWatch(outcome)) =
            super::super::failure_recovery::runtime_dispatch_from_gui(&dispatch)
        else {
            return;
        };
        self.variable_browser.notice = None;
        match outcome {
            VariableWatchOutcome::Added => {
                self.variable_browser.notice =
                    Some(tr(self.locale, "已加入关注清单。", "Added to watch list.").into());
            }
            VariableWatchOutcome::AlreadyWatched { reference } => {
                self.variable_browser.show_watched(reference.clone());
                self.variable_browser.watch.notice = Some(
                    tr(
                        self.locale,
                        "已选中原关注项。",
                        "Selected the existing watch list item.",
                    )
                    .into(),
                );
            }
            VariableWatchOutcome::Moved { position } => {
                let watch = &mut self.variable_browser.watch;
                watch.focus = move_direction.map(WatchFocus::Move);
                watch.reveal_selection = true;
                watch.notice = Some(if self.locale == StudioShellLocale::ZhCn {
                    format!("已移动至第 {position} 项。")
                } else {
                    format!("Moved to item {position}.")
                });
            }
            VariableWatchOutcome::Removed { next } => {
                let watch = &mut self.variable_browser.watch;
                watch.selected = next.clone();
                watch.detail_page = false;
                watch.reveal_selection = next.is_some();
                watch.focus = Some(if next.is_some() {
                    WatchFocus::Row
                } else {
                    WatchFocus::Heading
                });
                watch.notice = Some(
                    tr(
                        self.locale,
                        "已从关注清单移除。",
                        "Removed from watch list.",
                    )
                    .into(),
                );
            }
            VariableWatchOutcome::Unchanged => {}
        }
    }
}

pub(super) fn render_add(
    ui: &mut egui::Ui,
    workspace: &WorkspaceState,
    selection: &BrowserSelection,
    locale: StudioShellLocale,
    intent: &mut Option<BrowserIntent>,
) {
    let Some(variable) =
        references::selected_variable(selection).filter(StableVariableReference::supports)
    else {
        return;
    };
    match workspace.watch_reference(variable.clone()) {
        Ok(reference) => {
            let reference = reference.to_string();
            if workspace.variable_watch_list.entries().contains(&reference) {
                if ui
                    .button(tr(locale, "查看已关注项", "View watched item"))
                    .clicked()
                {
                    *intent = Some(BrowserIntent::ShowWatched(reference));
                }
            } else if ui
                .button(tr(locale, "加入关注", "Add to watch list"))
                .clicked()
            {
                *intent = Some(BrowserIntent::EditWatch(VariableWatchCommand::Add(
                    variable,
                )));
            }
        }
        Err(error) => {
            ui.add_enabled(
                false,
                egui::Button::new(tr(locale, "加入关注", "Add to watch list")),
            )
            .on_disabled_hover_text(references::error_label(error, locale));
        }
    }
}

struct WatchRow {
    object: String,
    identity: String,
    field: String,
    section: &'static str,
    value: Result<ResolvedWatch, String>,
}

struct ResolvedWatch {
    reference: StableVariableReference,
    selection: BrowserSelection,
    read: BrowserRead,
}

impl WatchRow {
    fn read(workspace: &WorkspaceState, text: &str, locale: StudioShellLocale) -> Self {
        let reference = match text.parse::<StableVariableReference>() {
            Ok(reference) => reference,
            Err(error) => {
                return Self {
                    object: tr(locale, "原引用", "Original reference").into(),
                    identity: text.into(),
                    field: "—".into(),
                    section: "—",
                    value: Err(references::error_label(error, locale).into()),
                };
            }
        };
        let variable = reference.variable();
        let identity = object_identity(&rf_ui::project_query::ProjectObjectId::Flow(
            variable.object.clone(),
        ));
        let section = match variable.section {
            VariableSection::Inputs => tr(locale, "输入", "Input"),
            VariableSection::Results => tr(locale, "结果", "Result"),
        };
        let selection = references::variable_selection(variable.clone());
        let field = field_label(
            selection
                .field
                .as_ref()
                .expect("watch references always target fields"),
            locale,
        );
        let resolved = reference
            .resolve(workspace)
            .map_err(|error| references::error_label(error, locale).to_owned())
            .and_then(|_| {
                let object = ProjectQuery::new(workspace)
                    .object(&selection.object)
                    .map_err(|error| query_error(&error, locale))?;
                let read = read_selection(workspace, &selection, locale)?;
                Ok((object.label, read))
            });
        let (object, value) = match resolved {
            Ok((name, read)) => (
                name,
                Ok(ResolvedWatch {
                    reference,
                    selection,
                    read,
                }),
            ),
            Err(reason) => (identity.clone(), Err(reason)),
        };
        Self {
            object,
            identity,
            field,
            section,
            value,
        }
    }

    fn display(&self, locale: StudioShellLocale) -> (&str, &str) {
        match &self.value {
            Ok(ResolvedWatch { read, .. }) => (&read.value, &read.status),
            Err(_) => ("—", tr(locale, "引用失效", "Broken reference")),
        }
    }
}

pub(super) fn render(
    ui: &mut egui::Ui,
    state: &mut VariableBrowserState,
    workspace: &WorkspaceState,
    locale: StudioShellLocale,
    intent: &mut Option<BrowserIntent>,
) {
    let entries = workspace.variable_watch_list.entries();
    let watch = &mut state.watch;
    let wide = state.mode == BrowserMode::Expanded
        && ui.available_width()
            >= 850.0 * (ui.text_style_height(&egui::TextStyle::Body) / 18.0).max(1.0);
    if let Some(notice) = state.notice.as_ref().or(watch.notice.as_ref()) {
        ui.add(egui::Label::new(notice).wrap());
    }
    let dirty = workspace.variable_watch_list.is_dirty();
    ui.small(if dirty {
        tr(locale, "关注清单未保存", "Watch list unsaved")
    } else {
        tr(locale, "关注清单已保存", "Watch list saved")
    });
    ui.separator();
    let selected = watch
        .selected
        .as_ref()
        .and_then(|text| entries.iter().position(|entry| entry == text));
    if wide && !entries.is_empty() {
        let width = ui.available_width();
        let height = ui.available_height();
        let detail_width = (width * 0.32).clamp(265.0, 380.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2((width - detail_width - 16.0).max(0.0), height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_list(ui, watch, workspace, locale, true);
                },
            );
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(detail_width, height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("watch-expanded-detail")
                        .show(ui, |ui| {
                            render_detail(ui, watch, workspace, locale, intent);
                        });
                },
            );
        });
    } else if watch.detail_page && selected.is_some() {
        if ui
            .button(tr(locale, "← 返回关注清单", "← Back to watch list"))
            .clicked()
        {
            watch.back_to_list();
        }
        egui::ScrollArea::vertical()
            .id_salt("watch-narrow-detail")
            .show(ui, |ui| {
                render_detail(ui, watch, workspace, locale, intent);
            });
    } else {
        render_list(ui, watch, workspace, locale, false);
    }
}

const WATCH_COLUMNS: [f32; 6] = [0.09, 0.27, 0.10, 0.20, 0.17, 0.17];

fn render_list(
    ui: &mut egui::Ui,
    state: &mut WatchViewState,
    workspace: &WorkspaceState,
    locale: StudioShellLocale,
    wide: bool,
) {
    let entries = workspace.variable_watch_list.entries();
    let heading = ui.add(
        egui::Label::new(egui::RichText::new(tr(locale, "关注清单", "Watch list")).strong())
            .sense(egui::Sense::focusable_noninteractive()),
    );
    if state.focus == Some(WatchFocus::Heading) {
        heading.request_focus();
        state.focus = None;
    }
    if entries.is_empty() {
        ui.add(
            egui::Label::new(tr(
                locale,
                "从工程分类或变量树打开变量详情，选择「加入关注」。",
                "Open a variable in Project or Variable tree and choose Add to watch list.",
            ))
            .wrap(),
        );
        return;
    }
    if wide {
        render::browser_columns(
            ui,
            [
                tr(locale, "顺序", "Order"),
                tr(locale, "对象 / ID", "Object / ID"),
                tr(locale, "分区", "Section"),
                tr(locale, "字段", "Field"),
                tr(locale, "值", "Value"),
                tr(locale, "状态", "Status"),
            ],
            WATCH_COLUMNS,
            true,
        );
    }
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let height = line * if wide { 2.8 } else { 4.4 } + 8.0;
    let mut list = egui::ScrollArea::vertical()
        .id_salt(if wide {
            "watch-expanded-list"
        } else {
            "watch-narrow-list"
        })
        .auto_shrink([false, false]);
    if std::mem::take(&mut state.reveal_selection)
        && let Some(index) = entries
            .iter()
            .position(|entry| Some(entry) == state.selected.as_ref())
    {
        list = list.vertical_scroll_offset(
            index.saturating_sub(1) as f32 * (height + ui.spacing().item_spacing.y),
        );
    }
    list.show_rows(ui, height, entries.len(), |ui, range| {
        for index in range {
            let reference = &entries[index];
            let row = WatchRow::read(workspace, reference, locale);
            let selected = state.selected.as_ref() == Some(reference);
            let response = ui
                .push_id(("watch-row", reference), |ui| {
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), height),
                        egui::Sense::click(),
                    );
                    let (value, status) = row.display(locale);
                    let text = format!(
                        "{} · {} / {} / {} · {value} · {status}",
                        index + 1,
                        row.object,
                        row.section,
                        row.field
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::SelectableLabel,
                            ui.is_enabled(),
                            selected,
                            &text,
                        )
                    });
                    if selected || response.hovered() {
                        ui.painter().rect_filled(
                            rect,
                            3.0,
                            if selected {
                                ui.visuals().selection.bg_fill
                            } else {
                                ui.visuals().widgets.hovered.bg_fill
                            },
                        );
                    }
                    if response.has_focus() {
                        ui.painter().rect_stroke(
                            rect,
                            3.0,
                            ui.visuals().selection.stroke,
                            egui::StrokeKind::Inside,
                        );
                    }
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect.shrink(4.0))
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    child.set_clip_rect(rect.intersect(ui.clip_rect()));
                    child.style_mut().interaction.selectable_labels = false;
                    if wide {
                        render::browser_columns(
                            &mut child,
                            [
                                &(index + 1).to_string(),
                                &row.object,
                                row.section,
                                &row.field,
                                value,
                                status,
                            ],
                            WATCH_COLUMNS,
                            false,
                        );
                        child.add(
                            egui::Label::new(egui::RichText::new(&row.identity).small()).truncate(),
                        );
                    } else {
                        child.add(
                            egui::Label::new(
                                egui::RichText::new(format!("{}  {}", index + 1, row.field))
                                    .strong(),
                            )
                            .truncate(),
                        );
                        let mut object = egui::text::LayoutJob::simple_singleline(
                            format!("{} / {}", row.object, row.section),
                            egui::TextStyle::Body.resolve(child.style()),
                            child.visuals().text_color(),
                        );
                        object.wrap.max_rows = 2;
                        object.wrap.break_anywhere = true;
                        child.add(egui::Label::new(object).wrap());
                        child.add(egui::Label::new(format!("{value} · {status}")).truncate());
                    }
                    response.on_hover_text(format!("{text}\n{}", row.identity))
                })
                .inner;
            if selected && state.focus == Some(WatchFocus::Row) {
                response.request_focus();
                state.focus = None;
            }
            if response.clicked()
                || (response.has_focus()
                    && ui.input_mut(|input| {
                        input.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                    }))
            {
                state.selected = Some(reference.clone());
                state.detail_page = !wide;
            }
        }
    });
}

fn render_detail(
    ui: &mut egui::Ui,
    state: &mut WatchViewState,
    workspace: &WorkspaceState,
    locale: StudioShellLocale,
    intent: &mut Option<BrowserIntent>,
) {
    use VariableWatchDirection::{Down, Up};
    let entries = workspace.variable_watch_list.entries();
    let Some((index, text)) = state.selected.as_ref().and_then(|selected| {
        entries
            .iter()
            .enumerate()
            .find(|(_, entry)| *entry == selected)
    }) else {
        ui.label(tr(
            locale,
            "选择一项查看详情。",
            "Select an item to inspect it.",
        ));
        return;
    };
    let row = WatchRow::read(workspace, text, locale);
    ui.small(if locale == StudioShellLocale::ZhCn {
        format!("第 {} 项 / 共 {} 项", index + 1, entries.len())
    } else {
        format!("Item {} of {}", index + 1, entries.len())
    });
    ui.heading(&row.field);
    ui.add(egui::Label::new(format!("{} / {}", row.object, row.section)).wrap());
    ui.add(egui::Label::new(&row.identity).wrap());
    let (value, status) = row.display(locale);
    ui.heading(value);
    ui.label(status);
    match &row.value {
        Ok(ResolvedWatch { read, .. }) => {
            ui.add(egui::Label::new(&read.source).wrap());
        }
        Err(reason) => {
            ui.add(egui::Label::new(reason).wrap());
        }
    }
    ui.horizontal_wrapped(|ui| {
        let can_move_up = index > 0;
        let can_move_down = index + 1 < entries.len();
        if let Some(WatchFocus::Move(direction)) = state.focus {
            state.focus = match (direction, can_move_up, can_move_down) {
                (Up, true, _) | (Down, _, true) => Some(WatchFocus::Move(direction)),
                (Up, false, true) => Some(WatchFocus::Move(Down)),
                (Down, true, false) => Some(WatchFocus::Move(Up)),
                _ => Some(WatchFocus::Row),
            };
        }
        for (direction, enabled, label) in [
            (Up, can_move_up, tr(locale, "↑ 上移", "↑ Move up")),
            (Down, can_move_down, tr(locale, "↓ 下移", "↓ Move down")),
        ] {
            let response = ui.add_enabled(enabled, egui::Button::new(label));
            if enabled && state.focus == Some(WatchFocus::Move(direction)) {
                response.request_focus();
                state.focus = None;
            }
            if response.clicked() {
                *intent = Some(BrowserIntent::EditWatch(VariableWatchCommand::Move {
                    reference: text.clone(),
                    direction,
                }));
            }
        }
    });
    let locate_label = if row.section == tr(locale, "结果", "Result") {
        tr(locale, "定位结果字段", "Locate result field")
    } else {
        tr(locale, "定位输入字段", "Locate input field")
    };
    let copy_label = tr(locale, "复制当前值", "Copy current value");
    if let Ok(ResolvedWatch {
        reference,
        selection,
        read,
    }) = row.value
    {
        if ui.button(locate_label).clicked() {
            *intent = Some(BrowserIntent::Navigate(BrowserNavigation::StableReference(
                reference,
            )));
        }
        if ui
            .add_enabled(read.copy.is_some(), egui::Button::new(copy_label))
            .clicked()
        {
            *intent = Some(BrowserIntent::Copy(selection));
        }
    } else {
        ui.add_enabled(false, egui::Button::new(locate_label));
        ui.add_enabled(false, egui::Button::new(copy_label));
    }
    if ui
        .button(tr(locale, "移除关注", "Remove from watch list"))
        .clicked()
    {
        *intent = Some(BrowserIntent::EditWatch(VariableWatchCommand::Remove(
            text.clone(),
        )));
    }
}
