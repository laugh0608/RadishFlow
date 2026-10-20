use super::{egui, state_presentation::StudioStateTokens};

/// Keep keyboard navigation inside a unit menu instead of egui's spatial focus search.
/// The selector owns focus; the highlighted option is its accessible active descendant.
pub(super) fn unit_selector<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash,
    label: &str,
    selected: T,
    choices: &[(T, &str)],
) -> egui::InnerResponse<Option<T>> {
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        for widget in [&mut widgets.inactive, &mut widgets.hovered] {
            widget.bg_stroke = egui::Stroke::new(1.0, StudioStateTokens::BORDER);
        }
        for widget in [&mut widgets.active, &mut widgets.open] {
            widget.bg_fill = StudioStateTokens::EDIT;
            widget.weak_bg_fill = StudioStateTokens::EDIT;
            widget.bg_stroke = egui::Stroke::new(1.0, StudioStateTokens::FOCUS);
        }
        render_unit_selector(ui, id_salt, label, selected, choices)
    })
    .inner
}

fn render_unit_selector<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash,
    label: &str,
    selected: T,
    choices: &[(T, &str)],
) -> egui::InnerResponse<Option<T>> {
    let selected_index = choices
        .iter()
        .position(|(value, _)| *value == selected)
        .expect("selected unit must be an allowed choice");
    // ComboBox hashes its salt before combining it with the parent UI identity.
    let button_id = ui.make_persistent_id(egui::Id::new(&id_salt));
    let cursor_id = button_id.with("unit-menu-cursor");
    let was_open = egui::ComboBox::is_open(ui.ctx(), button_id);
    let mut cursor = if was_open {
        ui.data_mut(|data| data.get_temp::<usize>(cursor_id))
            .unwrap_or(selected_index)
    } else {
        selected_index
    };
    let mut accept = false;
    let mut cancel = false;
    if was_open {
        ui.input_mut(|input| {
            let previous = input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
                || input.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
            let next = input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                || input.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
            if previous {
                cursor = (cursor + choices.len() - 1) % choices.len();
            } else if next {
                cursor = (cursor + 1) % choices.len();
            }
            if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
                cursor = 0;
            }
            if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
                cursor = choices.len() - 1;
            }
            accept = input.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                || input.consume_key(egui::Modifiers::NONE, egui::Key::Space);
            cancel = input.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
        });
    }
    let mut chosen = None;
    let mut active_option = None;
    let mut popup = None;
    let response = egui::ComboBox::from_id_salt(id_salt)
        .selected_text(choices[selected_index].1)
        .show_ui(ui, |ui| {
            let popup_id = button_id.with("unit-options");
            popup = Some(popup_id);
            let ctx = ui.ctx().clone();
            ctx.accesskit_node_builder(popup_id, |node| {
                node.set_role(egui::accesskit::Role::ListBox);
                node.set_label(label);
            });
            if !was_open {
                // Opening Enter/Space must not also choose an option.
                ui.input_mut(|input| {
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Space);
                });
            }
            ctx.with_accessibility_parent(popup_id, || {
                for (index, (value, text)) in choices.iter().enumerate() {
                    let option = ui.selectable_label(*value == selected, *text);
                    ctx.accesskit_node_builder(option.id, |node| {
                        node.set_role(egui::accesskit::Role::ListBoxOption);
                        node.set_selected(*value == selected);
                    });
                    if option.has_focus() {
                        cursor = index;
                    }
                    if index == cursor {
                        active_option = Some(option.id);
                        ui.painter().rect_stroke(
                            option.rect,
                            2.0,
                            ui.visuals().selection.stroke,
                            egui::StrokeKind::Inside,
                        );
                        if was_open {
                            option.scroll_to_me(None);
                        }
                    }
                    if option.clicked() || (accept && index == cursor && !cancel) {
                        chosen = Some(*value);
                    }
                }
            });
        })
        .response;
    debug_assert_eq!(button_id, response.id);
    if chosen.is_some() || cancel {
        ui.memory_mut(|memory| memory.close_popup());
        response.request_focus();
    }
    let is_open = egui::ComboBox::is_open(ui.ctx(), response.id);
    if is_open {
        ui.data_mut(|data| data.insert_temp(cursor_id, cursor));
        if !response.has_focus() {
            response.request_focus();
            ui.ctx().request_repaint();
        }
    }
    ui.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            response.id,
            egui::EventFilter {
                horizontal_arrows: is_open,
                vertical_arrows: is_open,
                tab: is_open,
                escape: is_open,
            },
        );
    });
    response.widget_info(|| {
        let mut info =
            egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ui.is_enabled(), label);
        info.current_text_value = Some(choices[selected_index].1.into());
        info
    });
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_expanded(is_open);
        if is_open {
            if let Some(option) = active_option {
                node.set_active_descendant(option.value().into());
            }
            if let Some(popup) = popup {
                node.set_controls(vec![popup.value().into()]);
            }
        }
    });
    egui::InnerResponse::new(chosen, response)
}
