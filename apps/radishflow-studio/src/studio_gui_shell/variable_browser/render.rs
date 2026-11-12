use super::*;
use rf_ui::project_query::{
    ProjectActionDescriptor, ProjectObjectId, ProjectObjectKind, ProjectQuery, ProjectRecord,
};
use rf_ui::variable_browser::{ActionExecution, VariableSection};

pub(super) fn browser(
    ui: &mut egui::Ui,
    state: &mut VariableBrowserState,
    workspace: &rf_ui::WorkspaceState,
    locale: StudioShellLocale,
) -> Option<BrowserIntent> {
    let mut intent = None;
    ui.horizontal_wrapped(|ui| {
        ui.heading(tr(locale, "变量浏览器", "Variable browser"));
        if state.mode == BrowserMode::Expanded {
            if ui
                .button(tr(locale, "返回流程图", "Back to flowsheet"))
                .clicked()
            {
                intent = Some(BrowserIntent::Flowsheet);
            }
            if ui
                .button(tr(locale, "收起为侧区", "Dock to side"))
                .clicked()
            {
                intent = Some(BrowserIntent::Dock);
            }
        } else if ui
            .button(tr(locale, "展开工作区", "Expand workspace"))
            .clicked()
        {
            intent = Some(BrowserIntent::Expand);
        }
        if ui.button(tr(locale, "关闭", "Close")).clicked() {
            intent = Some(BrowserIntent::Close);
        }
    });
    ui.label(format!(
        "{} · r{} · {}: {}",
        workspace.document.metadata.title,
        workspace.document.revision,
        tr(locale, "流程结果", "Flow result"),
        result_label(workspace.flow_result().status(), locale)
    ));
    ui.label(if workspace.pending_flow_run().is_some() {
        tr(
            locale,
            "运行任务：进行中（独立于结果资格）",
            "Run task: in progress (independent of result status)",
        )
    } else {
        tr(locale, "运行任务：空闲", "Run task: idle")
    });
    ui.small(tr(
        locale,
        "只读已提交值；模板与结果分开。",
        "Committed values only; templates are separate from results.",
    ));
    references::render_reference_controls(ui, state, workspace, locale, &mut intent);
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(
            &mut state.directory,
            DirectoryView::Project,
            tr(locale, "工程分类", "Project"),
        );
        ui.selectable_value(
            &mut state.directory,
            DirectoryView::Variables,
            tr(locale, "变量树", "Variable tree"),
        );
        ui.selectable_value(
            &mut state.directory,
            DirectoryView::WatchList,
            format!(
                "{} · {}",
                tr(locale, "关注清单", "Watch list"),
                workspace.variable_watch_list.entries().len()
            ),
        );
    });
    if state.directory == DirectoryView::WatchList {
        watch_list::render(ui, state, workspace, locale, &mut intent);
        if consume_browser_escape(ui, state) {
            if state.reference_input.is_some() {
                state.reference_input = None;
            } else if state.watch.detail_page {
                state.watch.back_to_list();
            } else {
                intent = Some(BrowserIntent::Close);
            }
        }
        return intent;
    }
    let search = ui.add(
        egui::TextEdit::singleline(&mut state.query)
            .desired_width(f32::INFINITY)
            .hint_text(tr(
                locale,
                "搜索对象 / 字段 / 单位",
                "Search objects / fields / units",
            )),
    );
    if search.changed() {
        state.detail_page = false;
    }
    ui.horizontal_wrapped(|ui| {
        for (value, label) in [
            (SectionFilter::All, tr(locale, "全部", "All")),
            (SectionFilter::Inputs, tr(locale, "输入", "Inputs")),
            (SectionFilter::Results, tr(locale, "结果", "Results")),
        ] {
            if ui
                .selectable_value(&mut state.section, value, label)
                .changed()
            {
                state.detail_page = false;
            }
        }
        if ui
            .small_button(tr(locale, "清除搜索", "Clear search"))
            .clicked()
        {
            state.query.clear();
            state.detail_page = false;
        }
        if state.scope.is_some()
            && ui
                .small_button(tr(locale, "全部对象", "All objects"))
                .clicked()
        {
            state.scope = None;
        }
    });
    if let Some(notice) = &state.notice {
        ui.add(egui::Label::new(notice).wrap());
    }
    if let Some(scope) = &state.scope {
        match ProjectQuery::new(workspace).object(scope) {
            Ok(object) => {
                ui.small(format!("{}: {}", tr(locale, "范围", "Scope"), object.label));
            }
            Err(error) => {
                ui.colored_label(egui::Color32::DARK_RED, query_error(&error, locale));
            }
        }
    }
    state
        .cache
        .filter(&state.query, state.section, state.scope.as_ref());
    ui.separator();
    let font_scale = (ui.text_style_height(&egui::TextStyle::Body) / 18.0).max(1.0);
    let wide = state.mode == BrowserMode::Expanded && ui.available_width() >= 850.0 * font_scale;
    if wide {
        let width = ui.available_width();
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(230.0 * font_scale, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_width(230.0 * font_scale);
                    directory(ui, state, locale);
                },
            );
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2((width - 250.0 * font_scale).max(0.0), ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    rows(
                        ui,
                        state,
                        workspace,
                        locale,
                        true,
                        (ui.available_height() * 0.45).max(100.0),
                    );
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("browser-expanded-detail")
                        .auto_shrink([false, false])
                        .show(ui, |ui| detail(ui, state, workspace, locale, &mut intent));
                },
            );
        });
    } else if state.detail_page {
        if ui
            .button(tr(locale, "← 返回列表", "← Back to list"))
            .clicked()
        {
            state.detail_page = false;
        }
        egui::ScrollArea::vertical()
            .id_salt("browser-narrow-detail")
            .auto_shrink([false, false])
            .show(ui, |ui| detail(ui, state, workspace, locale, &mut intent));
    } else {
        egui::CollapsingHeader::new(tr(locale, "对象目录", "Object directory"))
            .id_salt("browser-narrow-directory")
            .show(ui, |ui| directory(ui, state, locale));
        rows(ui, state, workspace, locale, false, ui.available_height());
    }
    if consume_browser_escape(ui, state) {
        if state.reference_input.is_some() {
            state.reference_input = None;
        } else if state.detail_page {
            state.detail_page = false;
        } else {
            intent = Some(BrowserIntent::Close);
        }
    }
    intent
}

fn consume_browser_escape(ui: &egui::Ui, state: &VariableBrowserState) -> bool {
    state.owns_shortcuts(ui.ctx())
        && !ui
            .ctx()
            .memory(|m| m.any_popup_open() || m.top_modal_layer().is_some())
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
}

fn category(kind: ProjectObjectKind) -> usize {
    match kind {
        ProjectObjectKind::Unit => 0,
        ProjectObjectKind::Stream => 1,
        ProjectObjectKind::Sizing
        | ProjectObjectKind::Rating
        | ProjectObjectKind::Case
        | ProjectObjectKind::Geometry => 2,
        ProjectObjectKind::Basis => 3,
        ProjectObjectKind::Source => 4,
    }
}

fn directory(ui: &mut egui::Ui, state: &mut VariableBrowserState, locale: StudioShellLocale) {
    egui::ScrollArea::vertical()
        .id_salt("browser-directory")
        .max_height(ui.available_height().min(500.0))
        .show(ui, |ui| {
            for (group, title) in [
                tr(locale, "流程 / 单元", "Flow / Units"),
                tr(locale, "流程 / 流股", "Flow / Streams"),
                tr(locale, "设备 / 工况 / 几何", "Equipment / Cases / Geometry"),
                tr(locale, "工程基础", "Engineering basis"),
                tr(locale, "来源", "Sources"),
            ]
            .into_iter()
            .enumerate()
            {
                egui::CollapsingHeader::new(title)
                    .id_salt(("browser-group", group))
                    .default_open(true)
                    .show(ui, |ui| {
                        for object in state
                            .cache
                            .objects
                            .iter()
                            .filter(|o| category(o.kind) == group)
                        {
                            let id = object_identity(&object.reference.object);
                            let selected = state.scope.as_ref() == Some(&object.reference);
                            let label = format!(
                                "{}{}",
                                if object.parent.is_some() { "↳ " } else { "" },
                                object.label
                            );
                            if ui
                                .add(egui::Button::new(label).selected(selected).wrap())
                                .on_hover_text(&id)
                                .clicked()
                            {
                                state.scope = Some(object.reference.clone());
                                state.selected = Some(BrowserSelection {
                                    object: object.reference.clone(),
                                    field: None,
                                });
                                state.detail_page = false;
                            }
                            if selected && state.directory == DirectoryView::Variables {
                                for section in
                                    [RowSection::Input, RowSection::Template, RowSection::Result]
                                {
                                    let entries: Vec<_> = state
                                        .cache
                                        .entries
                                        .iter()
                                        .filter(|e| {
                                            e.target.object == object.reference
                                                && e.target.field.is_some()
                                                && e.section == section
                                        })
                                        .collect();
                                    if entries.is_empty() {
                                        continue;
                                    }
                                    egui::CollapsingHeader::new(section.label(locale))
                                        .id_salt((&id, section as u8))
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            for entry in entries {
                                                if ui
                                                    .add(
                                                        egui::Button::new(&entry.label)
                                                            .selected(
                                                                state.selected.as_ref()
                                                                    == Some(&entry.target),
                                                            )
                                                            .wrap(),
                                                    )
                                                    .clicked()
                                                {
                                                    state.selected = Some(entry.target.clone());
                                                    state.detail_page = true;
                                                }
                                            }
                                        });
                                }
                            }
                        }
                    });
            }
        });
}

/// Cache wrapped metadata heights, never displayed values. The viewport only reads
/// visible values, including when long names require different row heights.
#[derive(Default)]
pub(super) struct RowLayout {
    key: Option<(u64, f32, egui::FontId, bool)>,
    offsets: Vec<f32>,
}
impl RowLayout {
    fn prepare(
        &mut self,
        ui: &egui::Ui,
        index: &BrowserIndex,
        wide: bool,
        locale: StudioShellLocale,
    ) {
        let width = (ui.available_width() - ui.spacing().scroll.bar_width - 16.0).max(40.0);
        let font = egui::TextStyle::Body.resolve(ui.style());
        let key = (index.generation, width, font.clone(), wide);
        if self.key.as_ref() == Some(&key) {
            return;
        }
        let line = ui.text_style_height(&egui::TextStyle::Body);
        self.offsets.clear();
        self.offsets.push(0.0);
        for &i in &index.visible {
            let entry = &index.entries[i];
            let height = if wide {
                line * 2.5 + 14.0
            } else {
                let name = index.objects[entry.object_index].label.clone();
                let field = format!("{} / {}", entry.section.label(locale), entry.label);
                ui.fonts(|f| {
                    f.layout(name, font.clone(), egui::Color32::WHITE, width)
                        .size()
                        .y
                        + f.layout(field, font.clone(), egui::Color32::WHITE, width)
                            .size()
                            .y
                }) + line * 2.0
                    + ui.spacing().item_spacing.y * 4.0
                    + 12.0
            };
            self.offsets
                .push(self.offsets.last().copied().unwrap() + height + ui.spacing().item_spacing.y);
        }
        self.key = Some(key);
    }
}

fn rows(
    ui: &mut egui::Ui,
    state: &mut VariableBrowserState,
    workspace: &rf_ui::WorkspaceState,
    locale: StudioShellLocale,
    wide: bool,
    height: f32,
) {
    state
        .cache
        .filter(&state.query, state.section, state.scope.as_ref());
    if wide {
        table_cells(
            ui,
            [
                tr(locale, "对象", "Object"),
                tr(locale, "分区", "Section"),
                tr(locale, "字段", "Field"),
                tr(locale, "值 / 单位", "Value / unit"),
                tr(locale, "状态", "Status"),
            ],
            true,
        );
    }
    ui.small(format!(
        "{} {}",
        state.cache.visible.len(),
        tr(locale, "项", "items")
    ));
    if state.cache.visible.is_empty() {
        ui.label(tr(
            locale,
            "未找到匹配项。清除搜索或返回全部对象。",
            "No matches. Clear search or return to all objects.",
        ));
        return;
    }
    state.row_layout.prepare(ui, &state.cache, wide, locale);
    egui::ScrollArea::vertical()
        .id_salt("browser-variable-list")
        .max_height(height.max(60.0))
        .auto_shrink([false, false])
        .show_viewport(ui, |ui, viewport| {
            let offsets = &state.row_layout.offsets;
            ui.set_height(*offsets.last().unwrap_or(&0.0));
            let first = offsets
                .partition_point(|y| *y <= viewport.min.y)
                .saturating_sub(1)
                .min(state.cache.visible.len());
            let end = offsets
                .partition_point(|y| *y < viewport.max.y)
                .min(state.cache.visible.len());
            ui.add_space(offsets[first]);
            for visible in first..end {
                let row_height =
                    offsets[visible + 1] - offsets[visible] - ui.spacing().item_spacing.y;
                let entry = &state.cache.entries[state.cache.visible[visible]];
                let object = &state.cache.objects[entry.object_index];
                let read = entry
                    .target
                    .field
                    .as_ref()
                    .map(|_| read_selection(workspace, &entry.target, locale));
                let (value, status) = match &read {
                    Some(Ok(read)) => (read.value.as_str(), read.status.as_str()),
                    Some(Err(reason)) => ("—", reason.as_str()),
                    None => ("", tr(locale, "只读记录", "Read-only record")),
                };
                let selected = state.selected.as_ref() == Some(&entry.target);
                let response = ui
                    .push_id(("browser-row", format!("{:?}", entry.target)), |ui| {
                        let size = egui::vec2(ui.available_width(), row_height);
                        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                ui.is_enabled(),
                                selected,
                                format!(
                                    "{} / {} / {}: {value}, {status}",
                                    object.label,
                                    entry.section.label(locale),
                                    entry.label
                                ),
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
                        let mut child = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(rect.shrink(4.0))
                                .layout(egui::Layout::top_down(egui::Align::Min)),
                        );
                        child.set_clip_rect(rect.intersect(ui.clip_rect()));
                        child.style_mut().interaction.selectable_labels = false;
                        if wide {
                            table_cells(
                                &mut child,
                                [
                                    object.label.as_str(),
                                    entry.section.label(locale),
                                    entry.label.as_str(),
                                    value,
                                    status,
                                ],
                                false,
                            );
                            child.add(
                                egui::Label::new(
                                    egui::RichText::new(short_identity(&object.reference.object))
                                        .small(),
                                )
                                .truncate(),
                            );
                        } else {
                            child
                                .add(
                                    egui::Label::new(egui::RichText::new(&object.label).strong())
                                        .wrap(),
                                )
                                .on_hover_text(&object.label);
                            child.add(
                                egui::Label::new(format!(
                                    "{} / {}",
                                    entry.section.label(locale),
                                    entry.label
                                ))
                                .wrap(),
                            );
                            child
                                .add(egui::Label::new(format!("{value} · {status}")).truncate())
                                .on_hover_text(format!("{value} · {status}"));
                            child.add(
                                egui::Label::new(
                                    egui::RichText::new(short_identity(&object.reference.object))
                                        .small(),
                                )
                                .truncate(),
                            );
                        }
                        response.on_hover_text(format!(
                            "{}\n{}",
                            object.label,
                            object_identity(&object.reference.object)
                        ))
                    })
                    .inner;
                if response.clicked()
                    || (response.has_focus()
                        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)))
                {
                    state.selected = Some(entry.target.clone());
                    state.detail_page = true;
                    state.notice = None;
                }
            }
        });
}

fn table_cells(ui: &mut egui::Ui, texts: [&str; 5], heading: bool) {
    browser_columns(ui, texts, [0.25, 0.10, 0.23, 0.20, 0.22], heading);
}

pub(super) fn browser_columns<const N: usize>(
    ui: &mut egui::Ui,
    texts: [&str; N],
    fractions: [f32; N],
    heading: bool,
) {
    let height = ui.text_style_height(&egui::TextStyle::Body) * 1.5;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let spacing = ui.spacing().item_spacing.x;
    let usable = (rect.width() - N.saturating_sub(1) as f32 * spacing).max(0.0);
    let mut x = rect.left();
    for (text, fraction) in texts.into_iter().zip(fractions) {
        let cell = egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(usable * fraction, height),
        );
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(cell)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let content = if heading {
            egui::RichText::new(text).strong()
        } else {
            egui::RichText::new(text)
        };
        child
            .add(egui::Label::new(content).truncate())
            .on_hover_text(text);
        x += cell.width() + spacing;
    }
}

fn short_identity(id: &ProjectObjectId) -> String {
    let identity = object_identity(id);
    if identity.chars().count() <= 42 {
        identity
    } else {
        format!(
            "…{}",
            identity
                .chars()
                .rev()
                .take(38)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>()
        )
    }
}

fn detail(
    ui: &mut egui::Ui,
    state: &VariableBrowserState,
    workspace: &rf_ui::WorkspaceState,
    locale: StudioShellLocale,
    intent: &mut Option<BrowserIntent>,
) {
    let Some(selected) = &state.selected else {
        ui.label(tr(
            locale,
            "选择对象或字段查看详情。",
            "Select an object or field to inspect it.",
        ));
        return;
    };
    let query = ProjectQuery::new(workspace);
    let object = match query.object(&selected.object) {
        Ok(object) => object,
        Err(error) => {
            ui.colored_label(egui::Color32::DARK_RED, query_error(&error, locale));
            return;
        }
    };
    ui.add(egui::Label::new(egui::RichText::new(&object.label).heading()).wrap());
    if let Some(parent) = &object.parent
        && let Ok(parent) = query.object(parent)
    {
        ui.label(format!("{} / {}", parent.label, object.label));
    }
    ui.add(egui::Label::new(object_identity(&object.reference.object)).wrap());
    if let Some(field) = &selected.field {
        ui.strong(field_label(field, locale));
        watch_list::render_add(ui, workspace, selected, locale, intent);
        references::render_copy_reference(ui, workspace, selected, locale, intent);
        match read_selection(workspace, selected, locale) {
            Ok(read) => {
                ui.add(egui::Label::new(egui::RichText::new(&read.value).heading()).wrap());
                ui.colored_label(
                    if read.error {
                        egui::Color32::from_rgb(185, 28, 28)
                    } else {
                        ui.visuals().text_color()
                    },
                    &read.status,
                );
                ui.add(egui::Label::new(&read.source).wrap());
                ui.add(egui::Label::new(&read.explanation).wrap());
                if let Some(canonical) = &read.canonical {
                    ui.add(
                        egui::Label::new(format!(
                            "{}: {canonical}",
                            tr(locale, "规范值 / SI", "Canonical / SI")
                        ))
                        .wrap(),
                    );
                }
                if ui
                    .add_enabled(
                        read.copy.is_some(),
                        egui::Button::new(tr(
                            locale,
                            "复制当前值与单位",
                            "Copy current value and unit",
                        ))
                        .wrap(),
                    )
                    .clicked()
                {
                    *intent = Some(BrowserIntent::Copy(selected.clone()));
                }
            }
            Err(reason) => {
                ui.colored_label(egui::Color32::DARK_RED, reason);
            }
        }
    }
    let label = if matches!(
        &selected.field,
        Some(ProjectField::Flow {
            section: VariableSection::Inputs,
            ..
        })
    ) {
        tr(locale, "定位输入字段", "Locate input field")
    } else if matches!(
        &selected.field,
        Some(ProjectField::Flow {
            section: VariableSection::Results,
            ..
        })
    ) {
        tr(locale, "查看流股结果", "View stream results")
    } else {
        tr(
            locale,
            "定位对象 / 打开正式页面",
            "Locate object / open editor",
        )
    };
    if ui.button(label).clicked() {
        *intent = Some(BrowserIntent::Navigate(BrowserNavigation::Selection(
            selected.clone(),
        )));
    }
    if selected.field.is_some() && !matches!(selected.field, Some(ProjectField::Flow { .. })) {
        ui.small(tr(
            locale,
            "此项定位到对象页面，不提供字段级聚焦。",
            "This opens the object page; exact field focus is unavailable.",
        ));
    }
    if matches!(selected.object.object, ProjectObjectId::Case(_)) {
        if let Ok(view) = query.equipment_result(&selected.object) {
            ui.separator();
            ui.strong(tr(locale, "设备结果", "Equipment result"));
            ui.label(equipment::status_label(locale, view.status()));
            super::super::result_review::equipment(ui, locale, &view);
        }
        if ui
            .button(tr(locale, "查看设备结果", "View equipment results"))
            .clicked()
        {
            *intent = Some(BrowserIntent::Navigate(BrowserNavigation::EquipmentResult(
                selected.object.clone(),
            )));
        }
    }
    if let ProjectObjectId::Flow(id) = &selected.object.object {
        if matches!(
            selected.field,
            Some(ProjectField::Flow {
                section: VariableSection::Results,
                ..
            })
        ) {
            super::super::result_review::flow(
                ui,
                locale,
                &radishflow_studio::FlowResultReview::capture(workspace),
            );
        }
        if let Ok(ports) = query.flow_variables().stream_references(id) {
            for (port, stream) in ports {
                let reference = query.reference(ProjectObjectId::Flow(stream));
                let label = query
                    .object(&reference)
                    .map(|o| o.label)
                    .unwrap_or_else(|_| tr(locale, "流股已不存在", "Stream removed").into());
                if ui.button(format!("{port} → {label}")).clicked() {
                    *intent = Some(BrowserIntent::Navigate(BrowserNavigation::FlowResults(
                        reference,
                    )));
                }
            }
        }
    }
    if let Ok(ProjectRecord::Source(source)) = query.record(&selected.object) {
        for (label, value) in [
            (tr(locale, "出处", "Reference"), source.reference.as_str()),
            (tr(locale, "位置", "Locator"), source.locator.as_str()),
            (
                tr(locale, "版本", "Version"),
                source.publication_version.as_str(),
            ),
            (
                tr(locale, "采纳说明", "Adoption note"),
                source.adoption_note.as_str(),
            ),
        ] {
            ui.add(
                egui::Label::new(format!(
                    "{label}: {}",
                    if value.is_empty() {
                        tr(locale, "未记录", "Unrecorded")
                    } else {
                        value
                    }
                ))
                .wrap(),
            );
        }
        ui.label(format!(
            "{} · {}",
            equipment::source_kind(locale, source.kind),
            equipment::source_review(locale, source.review)
        ));
        ui.label(format!(
            "{}: r{} · {}: {}",
            tr(locale, "修订", "Revision"),
            source.revision,
            tr(locale, "访问日期", "Access date"),
            source
                .accessed_on
                .map(|d| d.to_string())
                .unwrap_or_else(|| tr(locale, "未记录", "Unrecorded").into())
        ));
        ui.add(
            egui::Label::new(format!(
                "{}: {}\n{}",
                tr(locale, "适用范围", "Applicability"),
                source.applicability.fluid_system,
                source.applicability.scope
            ))
            .wrap(),
        );
        if let Some(conditions) = source.applicability.operating_conditions {
            ui.label(format!(
                "{} K · {} Pa",
                conditions.temperature_k, conditions.pressure_pa
            ));
        }
        ui.collapsing(
            tr(
                locale,
                "原始观测与换算记录",
                "Original observations and conversions",
            ),
            |ui| {
                for observation in &source.observations {
                    ui.add(
                        egui::Label::new(format!(
                            "{}: {} {} → {} {}\n{}",
                            observation.id,
                            observation.original_value,
                            observation.original_unit,
                            observation.canonical_value,
                            observation
                                .quantity
                                .definition()
                                .canonical_unit
                                .definition()
                                .symbol,
                            observation.conversion_note
                        ))
                        .wrap(),
                    );
                    for dependency in &observation.depends_on {
                        ui.label(format!(
                            "{} / {}",
                            dependency.source_id, dependency.observation_id
                        ));
                    }
                }
            },
        );
    }
    ui.collapsing(tr(locale,"动作描述（只读）","Action descriptions (read only)"),|ui| {
        if let Ok(actions) = query.actions(Some(&selected.object),ActionExecution::Background) {
            for action in actions {
                match action {
                    ProjectActionDescriptor::Flow(a) => {
                        ui.strong(a.label); ui.add(egui::Label::new(a.preconditions).wrap());
                        for field in a.fields { ui.label(field_label(&ProjectField::Flow { section: VariableSection::Inputs, field },locale)); }
                        for p in a.parameters { ui.label(format!("{} · {:?}{}",p.name,p.kind,if p.optional {" ?"}else{""})); }
                        ui.label(format!("{}: {} · {}: {}",tr(locale,"修改输入","Changes inputs"),a.modifies_document,tr(locale,"异步","Asynchronous"),a.asynchronous));
                        ui.add(egui::Label::new(a.returns).wrap());
                    }
                    ProjectActionDescriptor::Equipment(a) => {
                        use rf_ui::project_query::EquipmentActionKind;
                        ui.label(match a.kind { EquipmentActionKind::Edit(_) => tr(locale,"编辑对象","Edit object"), EquipmentActionKind::Check(_) => tr(locale,"检查工况","Check case"), EquipmentActionKind::Size(_) => tr(locale,"尺寸计算","Calculate sizing"), EquipmentActionKind::Rate(_) => tr(locale,"校核工况","Rate case") });
                        ui.label(format!("{}: {} · {}: {}",tr(locale,"未提交输入","Pending drafts"),a.pending_drafts,tr(locale,"修改输入","Changes inputs"),a.modifies_inputs));
                    }
                }
                ui.separator();
            }
        }
        ui.small(tr(locale,"前置条件与可用性由正式入口重验；此处不执行动作。","The editor rechecks preconditions and availability; actions cannot be executed here."));
    });
    if ui
        .small_button(tr(locale, "打开物性页", "Open property page"))
        .clicked()
    {
        *intent = Some(BrowserIntent::Navigate(BrowserNavigation::Property));
    }
}
