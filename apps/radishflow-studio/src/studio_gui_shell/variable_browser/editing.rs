//! A second view of the workspace's U2 session, never a browser-owned draft.
use super::*;
use rf_ui::variable_browser::{VariableField, VariableSection};
use rf_ui::{NumericEditCommand, NumericFieldPresentation, WorkspaceState};

/// VB-D04 deliberately exposes fewer fields than the general numeric API.
/// Check live connections, not names, cached rows, or stable-reference availability.
fn field_key(workspace: &WorkspaceState, selection: &BrowserSelection) -> Option<String> {
    let variable = references::selected_variable(selection)?;
    if variable.document != workspace.document.metadata.document_id
        || variable.section != VariableSection::Inputs
    {
        return None;
    }
    match (&variable.object, &variable.field) {
        (ObjectId::Unit(id), field) => {
            let unit = workspace.document.flowsheet.units.get(id)?;
            if unit.kind != "heater" {
                return None;
            }
            let field = match field {
                VariableField::OutletTemperature => {
                    rf_ui::UnitInspectorDraftField::OutletTemperatureK
                }
                VariableField::OutletPressure => rf_ui::UnitInspectorDraftField::OutletPressurePa,
                _ => return None,
            };
            Some(rf_ui::unit_inspector_draft_key(id, &field))
        }
        (ObjectId::Stream(id), field) => {
            let mut feed_outlet = false;
            for unit in workspace.document.flowsheet.units.values() {
                if unit.ports.iter().any(|port| {
                    port.direction == rf_types::PortDirection::Outlet
                        && port.kind == rf_types::PortKind::Material
                        && port.stream_id.as_ref() == Some(id)
                }) {
                    if unit.kind != "feed" {
                        return None;
                    }
                    feed_outlet = true;
                }
            }
            if !feed_outlet {
                return None;
            }
            let field = match field {
                VariableField::Temperature => rf_ui::StreamInspectorDraftField::TemperatureK,
                VariableField::Pressure => rf_ui::StreamInspectorDraftField::PressurePa,
                VariableField::MolarFlow => rf_ui::StreamInspectorDraftField::TotalMolarFlowMolS,
                _ => return None,
            };
            Some(rf_ui::stream_inspector_draft_key(id, &field))
        }
    }
}

fn presentation(
    app: &ReadyAppState,
    state: &VariableBrowserState,
    selection: &BrowserSelection,
) -> Option<(String, NumericFieldPresentation)> {
    let workspace = &app.platform_host.modeling_inputs().workspace;
    if app.read_only_project.is_some() || !state.belongs_to(workspace) {
        return None;
    }
    let key = field_key(workspace, selection)?;
    let variable = references::selected_variable(selection)?;
    let field = app
        .platform_host
        .numeric_field_presentation(&variable, None)
        .ok()?;
    Some((key, field))
}

pub(super) fn render(
    ui: &mut egui::Ui,
    state: &VariableBrowserState,
    app: &mut ReadyAppState,
    selection: &BrowserSelection,
) {
    let Some((key, mut field)) = presentation(app, state, selection) else {
        return;
    };
    let locale = app.locale;
    ui.separator();
    let focus = field.generation.is_none()
        && ui.button(tr(locale, "编辑", "Edit")).clicked()
        && app.dispatch_numeric_command(
            &mut field,
            NumericEditCommand::Begin(
                references::selected_variable(selection).expect("numeric field"),
            ),
        );
    if field.generation.is_none() {
        return;
    }
    ui.label(format!(
        "{}: {} {}",
        tr(locale, "已提交值", "Committed value"),
        if field.committed_text.is_empty() {
            "—"
        } else {
            &field.committed_text
        },
        field.display_unit.definition().symbol,
    ));
    let response = ui
        .push_id("browser-numeric-editor", |ui| {
            app.render_numeric_editor(
                ui,
                &key,
                tr(locale, "本次输入", "Input value"),
                &field,
                super::super::numeric_input::NumericEditSurface::Browser { focus },
            );
        })
        .response;
    // The shared editor consumes Escape for its input and unit menu. The browser
    // also cancels when an action button owns focus, before navigation handles it.
    let action_focused = ui
        .memory(|memory| memory.focused())
        .and_then(|id| ui.ctx().read_response(id))
        .is_some_and(|focused| response.rect.contains(focused.rect.center()));
    let composing = field.composing
        || ui.input(|input| {
            input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Ime(_)))
        });
    if action_focused && composing {
        // Do not let the browser interpret an IME escape as cancel or Back.
        ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    }
    if action_focused
        && !composing
        && !ui.memory(|memory| memory.any_popup_open())
        && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        && let Some((_, mut current)) = presentation(app, state, selection)
        && let Some(generation) = current.generation
    {
        let command = NumericEditCommand::Cancel {
            variable: current.variable.clone(),
            generation,
        };
        app.dispatch_numeric_command(&mut current, command);
    }
    ui.small(tr(
        locale,
        "Tab、返回列表与切换视图保留草稿；Enter / 应用显式提交。",
        "Tab, Back and view changes keep drafts; Enter / Apply commits explicitly.",
    ));
}

#[cfg(test)]
mod tests;
