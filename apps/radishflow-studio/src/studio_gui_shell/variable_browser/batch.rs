use super::*;
use crate::studio_gui_shell::numeric_input::NumericEditSurface;
use rf_types::RfError;
use rf_ui::{CheckedFeedMolarFlowBatch, FeedMolarFlowBatchError, FeedMolarFlowBatchReceipt};
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct FeedMolarFlowBatchView {
    pub(super) open: bool,
    pub(super) selected: BTreeSet<String>,
    checked: Option<CheckedFeedMolarFlowBatch>,
    rejected: Option<FeedMolarFlowBatchError>,
    applied: Option<FeedMolarFlowBatchReceipt>,
    dispatch_error: Option<String>,
    focus: Option<VariableId>,
}

impl FeedMolarFlowBatchView {
    pub(super) fn close(&mut self) {
        self.open = false;
        self.reset_check();
    }

    pub(super) fn select(&mut self, reference: &str, selected: bool) {
        if selected {
            self.selected.insert(reference.to_owned());
        } else {
            self.selected.remove(reference);
        }
        self.reset_check();
    }

    pub(super) fn retain_watched(&mut self, workspace: &rf_ui::WorkspaceState) {
        let count = self.selected.len();
        self.selected.retain(|reference| {
            workspace.variable_watch_list.entries().contains(reference)
                && reference
                    .parse::<StableVariableReference>()
                    .and_then(|reference| reference.resolve(workspace))
                    .is_ok()
        });
        if self.selected.len() != count {
            self.reset_check();
            self.rejected = Some(FeedMolarFlowBatchError::StaleCheck);
        }
    }

    fn reset_check(&mut self) {
        self.checked = None;
        self.rejected = None;
        self.applied = None;
        self.dispatch_error = None;
        self.focus = None;
    }

    pub(super) fn variables(&self, workspace: &rf_ui::WorkspaceState) -> Vec<VariableId> {
        workspace
            .variable_watch_list
            .entries()
            .iter()
            .filter(|reference| self.selected.contains(*reference))
            .filter_map(|reference| {
                reference
                    .parse::<StableVariableReference>()
                    .ok()
                    .map(|reference| reference.variable().clone())
            })
            .collect()
    }
}

pub(super) fn eligible(workspace: &rf_ui::WorkspaceState, text: &str) -> bool {
    text.parse::<StableVariableReference>()
        .and_then(|reference| reference.resolve(workspace))
        .is_ok_and(|variable| workspace.supports_feed_molar_flow_batch_item(&variable))
}

pub(super) fn render(
    ui: &mut egui::Ui,
    state: &mut FeedMolarFlowBatchView,
    app: &mut ReadyAppState,
) {
    let locale = app.locale;
    let workspace = &app.platform_host.modeling_inputs().workspace;
    state.retain_watched(workspace);
    let variables = state.variables(workspace);
    ui.horizontal_wrapped(|ui| {
        ui.heading(tr(locale, "核对批量草稿", "Review flow drafts"));
        if ui
            .button(tr(locale, "返回关注清单", "Back to watch list"))
            .clicked()
        {
            state.close();
        }
    });
    if !state.open {
        return;
    }

    render_status(ui, state, locale);
    let mut remove = None;
    egui::ScrollArea::vertical()
        .id_salt("feed-molar-flow-batch")
        .show(ui, |ui| {
            for variable in &variables {
                ui.push_id(&variable.object, |ui| {
                    if render_item(ui, state, app, variable) {
                        remove = Some(variable.clone());
                    }
                });
            }
            if let Some(variable) = remove {
                state.selected.retain(|text| {
                    !text
                        .parse::<StableVariableReference>()
                        .is_ok_and(|reference| reference.variable() == &variable)
                });
                state.reset_check();
            }

            let workspace = &app.platform_host.modeling_inputs().workspace;
            let live_variables = state.variables(workspace);
            if state
                .checked
                .as_ref()
                .is_some_and(|checked| !checked.matches(workspace, &live_variables))
            {
                state.checked = None;
                state.rejected = Some(FeedMolarFlowBatchError::StaleCheck);
                ui.ctx().request_repaint();
            }
            let count = live_variables.len();
            let zh = locale == StudioShellLocale::ZhCn;
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(
                        count >= 2,
                        egui::Button::new(if zh {
                            format!("检查 {count} 项草稿")
                        } else {
                            format!("Check {count} drafts")
                        }),
                    )
                    .clicked()
                {
                    state.reset_check();
                    match app
                        .platform_host
                        .modeling_inputs()
                        .workspace
                        .check_feed_molar_flow_batch(&live_variables)
                    {
                        Ok(checked) => state.checked = Some(checked),
                        Err(error) => state.rejected = Some(error),
                    }
                    ui.ctx().request_repaint();
                } else if let Some(checked) = &state.checked
                    && ui
                        .button(if zh {
                            format!("应用 {count} 项")
                        } else {
                            format!("Apply {count} items")
                        })
                        .clicked()
                {
                    let checked = checked.clone();
                    state.reset_check();
                    match apply(app, checked) {
                        Ok(Ok(receipt)) => state.applied = Some(receipt),
                        Ok(Err(error)) => state.rejected = Some(error),
                        Err(error) => state.dispatch_error = Some(error.to_string()),
                    }
                    ui.ctx().request_repaint();
                }
            });
        });
}

fn render_status(ui: &mut egui::Ui, state: &mut FeedMolarFlowBatchView, locale: StudioShellLocale) {
    if let Some(error) = &state.dispatch_error {
        ui.label(error);
    } else if let Some(receipt) = &state.applied {
        if receipt.changed_count == 0 {
            ui.label(tr(locale, "没有工程变更", "No project changes"));
        } else if locale == StudioShellLocale::ZhCn {
            ui.label(format!(
                "已更新 {} 项正式流量 · r{}",
                receipt.changed_count, receipt.revision
            ));
        } else {
            ui.label(format!(
                "Updated {} committed flows · r{}",
                receipt.changed_count, receipt.revision
            ));
        }
    } else if state.checked.is_some() {
        ui.label(tr(locale, "检查通过", "Check passed"));
    } else if let Some(error) = &state.rejected {
        match error {
            FeedMolarFlowBatchError::InvalidDrafts(issues) => {
                ui.label(tr(
                    locale,
                    "批量检查未通过 · 已应用 0 项",
                    "Batch check failed · 0 items applied",
                ));
                let target = issues
                    .iter()
                    .find(|(_, error)| !matches!(error, rf_ui::NumericEditError::UnsupportedField));
                if let Some((id, _)) = target
                    && ui
                        .button(tr(locale, "返回编辑首个失败项", "Edit first failed item"))
                        .clicked()
                {
                    state.focus = Some(id.clone());
                }
            }
            FeedMolarFlowBatchError::StaleCheck => {
                ui.label(tr(
                    locale,
                    "批次输入已变，请重新检查。",
                    "Batch input changed. Check drafts again.",
                ));
            }
            FeedMolarFlowBatchError::TooFewStreams => {
                ui.label(tr(
                    locale,
                    "请选择至少两条进料流股。",
                    "Select at least two feed streams.",
                ));
            }
        }
    }
}

fn render_item(
    ui: &mut egui::Ui,
    state: &mut FeedMolarFlowBatchView,
    app: &mut ReadyAppState,
    variable: &VariableId,
) -> bool {
    let locale = app.locale;
    let ObjectId::Stream(stream) = &variable.object else {
        return false;
    };
    let workspace = &app.platform_host.modeling_inputs().workspace;
    ui.separator();
    let label = workspace
        .document
        .flowsheet
        .streams
        .get(stream)
        .map_or(stream.as_str(), |stream| stream.name.as_str());
    ui.heading(label);
    let remove = ui
        .button(tr(locale, "移出本批", "Remove from batch"))
        .clicked();
    if !workspace.supports_feed_molar_flow_batch_item(variable) {
        ui.label(tr(
            locale,
            "此流股已不适用，请重新选择。",
            "This stream is no longer eligible. Select again.",
        ));
        return remove;
    }
    let Ok(field) = app.platform_host.numeric_field_presentation(variable, None) else {
        return remove;
    };
    ui.label(format!(
        "{} {} {}",
        tr(locale, "已提交值", "Committed value"),
        field.committed_text,
        field.display_unit.definition().symbol
    ));
    let focus = state.focus.as_ref() == Some(variable);
    if focus {
        state.focus = None;
    }
    let label = if locale == StudioShellLocale::ZhCn {
        format!("{label}总摩尔流量")
    } else {
        format!("{label} total molar flow")
    };
    app.render_numeric_editor(
        ui,
        stream.as_str(),
        &label,
        &field,
        NumericEditSurface::BrowserBatch { focus },
    );
    if let Some(FeedMolarFlowBatchError::InvalidDrafts(issues)) = &state.rejected {
        let workspace = &app.platform_host.modeling_inputs().workspace;
        if workspace
            .numeric_edit(variable)
            .ok()
            .map(rf_ui::NumericEditSession::generation)
            != field.generation
        {
            state.rejected = None;
        } else if let Some((_, error)) = issues.iter().find(|(id, _)| id == variable) {
            if field.issue.is_none() {
                ui.label(
                    if matches!(error, rf_ui::NumericEditError::MissingSession) {
                        tr(locale, "请填写本项流量。", "Enter a flow for this item.").to_owned()
                    } else {
                        locale.numeric_error(error)
                    },
                );
            }
        } else if field.issue.is_none() {
            ui.small(tr(locale, "此项通过检查", "This item passed"));
        }
    }
    remove
}

fn apply(
    app: &mut ReadyAppState,
    checked: CheckedFeedMolarFlowBatch,
) -> RfResult<Result<FeedMolarFlowBatchReceipt, FeedMolarFlowBatchError>> {
    let window_id = app
        .current_window_id()
        .ok_or_else(|| RfError::invalid_input("no active window"))?;
    let dispatch = app.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
        window_id,
        trigger: StudioRuntimeTrigger::ApplyFeedMolarFlowBatch(checked),
    })?;
    match super::super::failure_recovery::runtime_dispatch_from_gui(&dispatch) {
        Some(radishflow_studio::StudioRuntimeDispatch::FeedMolarFlowBatchApplied(outcome)) => {
            Ok(outcome.clone())
        }
        _ => Err(RfError::invalid_input("missing batch application outcome")),
    }
}

#[cfg(test)]
mod tests;
