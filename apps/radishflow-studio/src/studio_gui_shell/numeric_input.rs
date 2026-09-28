use super::*;
use rf_types::units::{ALL_UNITS, from_canonical};
use rf_ui::{
    NumericEditCommand, NumericEditEvent, NumericFieldIssue, NumericFieldPresentation,
    NumericFieldSource,
};

/// The accepted light palette. Numeric controls consume roles, never status strings.
struct StudioStateTokens;
impl StudioStateTokens {
    const SURFACE: egui::Color32 = egui::Color32::WHITE;
    const EDIT: egui::Color32 = egui::Color32::from_rgb(239, 246, 255);
    const TEXT: egui::Color32 = egui::Color32::from_rgb(31, 41, 55);
    const SPECIFIED: egui::Color32 = egui::Color32::from_rgb(29, 78, 216);
    const SECONDARY: egui::Color32 = egui::Color32::from_rgb(71, 85, 105);
    const ERROR: egui::Color32 = egui::Color32::from_rgb(185, 28, 28);
    const BORDER: egui::Color32 = egui::Color32::from_rgb(100, 116, 139);
    const FOCUS: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
}

#[derive(Clone, Default)]
struct TypingGroup {
    token: Option<(u64, u64)>,
    last_time: f64,
    last_cursor: Option<egui::text::CCursorRange>,
    direction: Option<bool>,
}

impl ReadyAppState {
    fn refresh_numeric_field(&self, field: &mut NumericFieldPresentation) {
        if let Some(current) = self
            .platform_host
            .snapshot()
            .runtime
            .active_inspector_detail
            .into_iter()
            .flat_map(|detail| detail.property_fields)
            .filter_map(|field| field.numeric.and_then(Result::ok))
            .find(|candidate| candidate.variable == field.variable)
        {
            *field = current;
        }
    }

    pub(super) fn dispatch_numeric_command(
        &mut self,
        field: &mut NumericFieldPresentation,
        command: NumericEditCommand,
    ) -> bool {
        let Some(window_id) = self.current_window_id() else {
            return false;
        };
        let result = self.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::NumericEdit(command),
        });
        self.refresh_numeric_field(field);
        if let Err(error) = &result {
            self.platform_host.record_activity_line(error.to_string());
        }
        result.is_ok()
    }

    fn begin_numeric_field(&mut self, field: &mut NumericFieldPresentation) -> bool {
        field.generation.is_some()
            || self
                .dispatch_numeric_command(field, NumericEditCommand::Begin(field.variable.clone()))
    }

    pub(super) fn edit_numeric_field(
        &mut self,
        field: &mut NumericFieldPresentation,
        event: NumericEditEvent,
    ) -> bool {
        if !self.begin_numeric_field(field) {
            return false;
        }
        self.dispatch_numeric_command(
            field,
            NumericEditCommand::Edit {
                variable: field.variable.clone(),
                generation: field.generation.expect("begun edit"),
                event,
            },
        )
    }

    fn cancel_numeric_field(&mut self, field: &mut NumericFieldPresentation) {
        if let Some(generation) = field.generation {
            self.dispatch_numeric_command(
                field,
                NumericEditCommand::Cancel {
                    variable: field.variable.clone(),
                    generation,
                },
            );
        }
    }

    pub(super) fn render_numeric_property_field(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        label: &str,
        original: &NumericFieldPresentation,
    ) {
        let id = ui.make_persistent_id(("numeric-input", &original.variable.document, key));
        ui.scope(|ui| {
            // First migration is explicitly light, independent of the OS / surrounding theme.
            ui.set_style({
                let mut style = (**ui.style()).clone();
                style.visuals = egui::Visuals::light();
                style
            });
            egui::Frame::new()
                .fill(StudioStateTokens::SURFACE)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    self.render_numeric_field_contents(ui, id, label, original);
                });
        });
    }

    fn render_numeric_field_contents(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        label: &str,
        original: &NumericFieldPresentation,
    ) {
        let mut field = original.clone();
        self.refresh_numeric_field(&mut field);
        let zh = matches!(self.locale, StudioShellLocale::ZhCn);
        let group_id = id.with("typing");
        let mut group =
            ui.data_mut(|data| data.get_temp::<TypingGroup>(group_id).unwrap_or_default());
        ui.add_space(8.0);
        ui.colored_label(StudioStateTokens::TEXT, egui::RichText::new(label).strong());
        ui.colored_label(
            StudioStateTokens::SECONDARY,
            match (zh, field.source) {
                (true, NumericFieldSource::Specified) => "来源：指定",
                (true, NumericFieldSource::Inherited) => "来源：继承；应用后显式指定",
                (true, NumericFieldSource::Missing) => "来源：未知；尚未指定",
                (false, NumericFieldSource::Specified) => "Source: specified",
                (false, NumericFieldSource::Inherited) => {
                    "Source: inherited; Apply makes it explicit"
                }
                (false, NumericFieldSource::Missing) => "Source: unknown; not specified",
            },
        );
        let focused = ui.memory(|memory| memory.has_focus(id))
            || (ui.memory(|memory| memory.had_focus_last_frame(id))
                && ui.input(|input| input.key_pressed(egui::Key::Escape)));
        let popup_open = ui.memory(|memory| memory.any_popup_open());
        let events = ui.input(|input| input.events.clone());
        let ime_event = focused
            && events
                .iter()
                .any(|event| matches!(event, egui::Event::Ime(_)));
        let ime_commit = focused
            && events
                .iter()
                .any(|event| matches!(event, egui::Event::Ime(egui::ImeEvent::Commit(_))));
        let ime_disabled = focused
            && events
                .iter()
                .any(|event| matches!(event, egui::Event::Ime(egui::ImeEvent::Disabled)));
        let ime_guard = field.composing || ime_event;
        if focused {
            self.begin_numeric_field(&mut field);
            let (undo, redo, escape) = ui.input_mut(|input| {
                let redo = input.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Z,
                ) || input.consume_key(egui::Modifiers::COMMAND, egui::Key::Y);
                let undo = input.consume_key(egui::Modifiers::COMMAND, egui::Key::Z);
                let escape = !popup_open
                    && !ime_guard
                    && input.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
                if ime_guard {
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                }
                (undo, redo, escape)
            });
            if (undo || redo) && !ime_guard {
                self.edit_numeric_field(
                    &mut field,
                    if redo {
                        NumericEditEvent::Redo
                    } else {
                        NumericEditEvent::Undo
                    },
                );
                group = TypingGroup::default();
            }
            if escape {
                self.cancel_numeric_field(&mut field);
                ui.memory_mut(|memory| memory.surrender_focus(id));
                group = TypingGroup::default();
            }
        }
        let previous_cursor =
            egui::TextEdit::load_state(ui.ctx(), id).and_then(|state| state.cursor.char_range());
        let mut raw = field.text.clone();
        let input_width = ui.available_width().clamp(80.0, 240.0);
        let output = ui
            .scope(|ui| {
                let visuals = ui.visuals_mut();
                visuals.extreme_bg_color = if focused {
                    StudioStateTokens::EDIT
                } else {
                    StudioStateTokens::SURFACE
                };
                for widget in [
                    &mut visuals.widgets.inactive,
                    &mut visuals.widgets.hovered,
                    &mut visuals.widgets.active,
                ] {
                    widget.bg_stroke = egui::Stroke::new(1.0, StudioStateTokens::BORDER);
                }
                egui::TextEdit::singleline(&mut raw)
                    .id(id)
                    .desired_width(input_width)
                    .text_color(if field.source == NumericFieldSource::Specified {
                        StudioStateTokens::SPECIFIED
                    } else {
                        StudioStateTokens::TEXT
                    })
                    .show(ui)
            })
            .inner;
        let response = output.response;
        if response.has_focus() {
            ui.data_mut(|data| data.insert_temp(egui::Id::new("numeric-keyboard-owner"), id));
            ui.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                        ..Default::default()
                    },
                )
            });
        }
        let mut edit_state = output.state;
        // The session owns paired text/unit history. Do not leave a second text-only undo stack.
        edit_state.clear_undoer();
        let cursor = edit_state.cursor.char_range();
        edit_state.store(ui.ctx(), id);
        if response.gained_focus() {
            self.begin_numeric_field(&mut field);
            group = TypingGroup::default();
        }
        if ime_commit {
            self.edit_numeric_field(&mut field, NumericEditEvent::CommitComposition(raw));
            group = TypingGroup::default();
        } else if ime_disabled || (field.composing && !focused) {
            self.edit_numeric_field(&mut field, NumericEditEvent::CancelComposition);
            group = TypingGroup::default();
        } else if ime_event || (field.composing && response.changed()) {
            self.edit_numeric_field(&mut field, NumericEditEvent::PreviewComposition(raw));
            group = TypingGroup::default();
        } else if response.changed() {
            let time = ui.input(|input| input.time);
            let direction = plain_typing_direction(&events, previous_cursor);
            let contiguous = direction.is_some()
                && direction == group.direction
                && time - group.last_time < 1.0
                && previous_cursor == group.last_cursor;
            if !contiguous {
                group.token = None;
            }
            self.begin_numeric_field(&mut field);
            let event = if direction.is_some() {
                let token = *group
                    .token
                    .get_or_insert((id.value(), field.generation.unwrap_or(0)));
                NumericEditEvent::ReplaceTextGrouped { raw, group: token }
            } else {
                NumericEditEvent::ReplaceText(raw)
            };
            self.edit_numeric_field(&mut field, event);
            group.last_time = time;
            group.last_cursor = cursor;
            group.direction = direction;
        } else if previous_cursor != cursor || response.lost_focus() {
            group = TypingGroup::default();
        }

        let error = matches!(
            field.issue,
            Some(NumericFieldIssue::Rejected(_) | NumericFieldIssue::Conflict)
        );
        if error {
            ui.painter().rect_stroke(
                response.rect,
                2.0,
                egui::Stroke::new(1.0, StudioStateTokens::ERROR),
                egui::StrokeKind::Inside,
            );
        }
        if response.has_focus() {
            ui.painter().rect_stroke(
                response.rect.expand(2.0),
                3.0,
                egui::Stroke::new(3.0, StudioStateTokens::SURFACE),
                egui::StrokeKind::Outside,
            );
            ui.painter().rect_stroke(
                response.rect.expand(5.0),
                4.0,
                egui::Stroke::new(1.0, StudioStateTokens::FOCUS),
                egui::StrokeKind::Outside,
            );
        }
        let issue_text = field.issue.as_ref().map(|issue| match issue {
            NumericFieldIssue::Incomplete => {
                if zh {
                    "输入未完成".into()
                } else {
                    "Input incomplete".into()
                }
            }
            NumericFieldIssue::Conflict => {
                if zh {
                    "已提交值或来源已改变，请选择如何保留本次编辑".into()
                } else {
                    "Committed value or source changed; choose how to preserve this edit".into()
                }
            }
            NumericFieldIssue::Rejected(reason) => format!(
                "{}: {reason}",
                if zh { "输入错误" } else { "Invalid input" }
            ),
        });
        response.widget_info(|| {
            let mut info = egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                true,
                format!(
                    "{label} {} · {}{}",
                    field.input_unit.definition().symbol,
                    if field.pending {
                        if zh { "未提交" } else { "Uncommitted" }
                    } else {
                        ""
                    },
                    issue_text.as_deref().unwrap_or("")
                ),
            );
            info.current_text_value = Some(field.text.clone());
            info
        });
        if let Some(reason) = &issue_text {
            ui.colored_label(
                if error {
                    StudioStateTokens::ERROR
                } else {
                    StudioStateTokens::SECONDARY
                },
                reason,
            );
        }
        let mut selected_unit = None;
        ui.horizontal_wrapped(|ui| {
            ui.label(if zh { "本次输入" } else { "Input unit" });
            ui.add_enabled_ui(!ime_guard, |ui| {
                egui::ComboBox::from_id_salt(id.with("unit"))
                    .selected_text(field.input_unit.definition().symbol)
                    .show_ui(ui, |ui| {
                        for unit in ALL_UNITS {
                            if from_canonical(1.0, field.quantity, *unit).is_ok()
                                && ui
                                    .selectable_label(
                                        *unit == field.input_unit,
                                        unit.definition().symbol,
                                    )
                                    .clicked()
                            {
                                selected_unit = Some(*unit);
                            }
                        }
                    });
            });
        });
        if let Some(unit) = selected_unit {
            self.edit_numeric_field(&mut field, NumericEditEvent::SelectInputUnit(unit));
            group = TypingGroup::default();
        }
        ui.colored_label(
            StudioStateTokens::SECONDARY,
            format!(
                "{} · {} {}",
                if field.pending {
                    if zh { "未提交" } else { "Uncommitted" }
                } else if zh {
                    "已提交"
                } else {
                    "Committed"
                },
                if zh {
                    "提交后显示"
                } else {
                    "Display after applying"
                },
                field.display_unit.definition().symbol
            ),
        );
        let enter = !ime_guard
            && response.lost_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) && input.modifiers == egui::Modifiers::NONE
            });
        ui.horizontal_wrapped(|ui| {
            let apply = ui
                .add_enabled(
                    !ime_guard && field.can_apply,
                    egui::Button::new(if zh { "应用" } else { "Apply" }),
                )
                .clicked();
            if !ime_guard && field.can_apply && (enter || apply) {
                let command = NumericEditCommand::Commit {
                    variable: field.variable.clone(),
                    generation: field.generation.expect("applicable session"),
                };
                self.dispatch_numeric_command(&mut field, command);
                self.refresh_numeric_field(&mut field);
                group = TypingGroup::default();
            }
            if ui
                .add_enabled(
                    field.generation.is_some(),
                    egui::Button::new(if zh { "取消编辑" } else { "Cancel edit" }),
                )
                .clicked()
            {
                self.cancel_numeric_field(&mut field);
                group = TypingGroup::default();
            }
            for (enabled, label, event) in [
                (
                    field.can_undo,
                    if zh { "撤销输入" } else { "Undo input" },
                    NumericEditEvent::Undo,
                ),
                (
                    field.can_redo,
                    if zh { "重做输入" } else { "Redo input" },
                    NumericEditEvent::Redo,
                ),
            ] {
                if ui
                    .add_enabled(!ime_guard && enabled, egui::Button::new(label))
                    .clicked()
                {
                    self.edit_numeric_field(&mut field, event);
                    group = TypingGroup::default();
                }
            }
        });
        if matches!(field.issue, Some(NumericFieldIssue::Conflict)) {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(if zh {
                        "按最新值重新编辑"
                    } else {
                        "Restart from latest"
                    })
                    .clicked()
                {
                    self.cancel_numeric_field(&mut field);
                    self.begin_numeric_field(&mut field);
                }
                if ui
                    .button(if zh {
                        "保留候选并重验"
                    } else {
                        "Revalidate my candidate"
                    })
                    .clicked()
                {
                    self.edit_numeric_field(&mut field, NumericEditEvent::Rebase);
                }
            });
        }
        // An untouched focus session should not freeze display units after focus moves away.
        if response.lost_focus()
            && !field.pending
            && !field.can_undo
            && !field.can_redo
            && !ui.memory(|memory| memory.any_popup_open())
            && !ime_guard
        {
            self.cancel_numeric_field(&mut field);
        }
        ui.data_mut(|data| data.insert_temp(group_id, group));
    }
}

fn plain_typing_direction(
    events: &[egui::Event],
    cursor: Option<egui::text::CCursorRange>,
) -> Option<bool> {
    if cursor.is_some_and(|range| range.primary != range.secondary) {
        return None;
    }
    let mut direction = None;
    for event in events {
        match event {
            egui::Event::Text(_) => {
                if direction == Some(false) {
                    return None;
                }
                direction = Some(true);
            }
            egui::Event::Key {
                key: egui::Key::Backspace | egui::Key::Delete,
                pressed: true,
                modifiers,
                ..
            } if *modifiers == egui::Modifiers::NONE => {
                if direction == Some(true) {
                    return None;
                }
                direction = Some(false);
            }
            egui::Event::Paste(_)
            | egui::Event::Cut
            | egui::Event::Ime(_)
            | egui::Event::PointerButton { .. } => return None,
            egui::Event::Key {
                key: egui::Key::ArrowLeft | egui::Key::ArrowRight | egui::Key::Home | egui::Key::End,
                pressed: true,
                ..
            } => return None,
            _ => {}
        }
    }
    direction
}

pub(super) fn owns_keyboard(ctx: &egui::Context) -> bool {
    ctx.data_mut(|data| data.get_temp::<egui::Id>(egui::Id::new("numeric-keyboard-owner")))
        .is_some_and(|id| {
            ctx.memory(|memory| memory.has_focus(id))
                || (ctx.memory(|memory| memory.had_focus_last_frame(id))
                    && ctx.input(|input| input.key_pressed(egui::Key::Escape)))
        })
}
