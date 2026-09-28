use super::*;
use rf_ui::variable_browser::{ObjectId, VariableField, VariableId, VariableSection};

/// Translate only the existing Inspector key boundary; widgets receive typed identities.
pub(super) fn project_numeric_fields(
    controller: &StudioAppHostController,
    fields: &mut [StudioGuiInspectorTargetFieldSnapshot],
) {
    for field in fields {
        let parts = if let Some((stream, kind)) =
            rf_ui::stream_inspector_draft_key_parts(&field.key)
        {
            let kind = match kind {
                rf_ui::StreamInspectorDraftField::TemperatureK => VariableField::Temperature,
                rf_ui::StreamInspectorDraftField::PressurePa => VariableField::Pressure,
                rf_ui::StreamInspectorDraftField::TotalMolarFlowMolS => VariableField::MolarFlow,
                _ => continue,
            };
            (ObjectId::Stream(stream), kind)
        } else if let Some((unit, kind)) = rf_ui::unit_inspector_draft_key_parts(&field.key) {
            let kind = match kind {
                rf_ui::UnitInspectorDraftField::OutletTemperatureK => {
                    VariableField::OutletTemperature
                }
                rf_ui::UnitInspectorDraftField::OutletPressurePa => VariableField::OutletPressure,
                _ => continue,
            };
            (ObjectId::Unit(unit), kind)
        } else {
            continue;
        };
        let id = VariableId {
            document: controller.document().metadata.document_id.clone(),
            object: parts.0,
            section: VariableSection::Inputs,
            field: parts.1,
        };
        match controller.numeric_field_presentation(&id) {
            Ok(presentation) => {
                // Preserve legacy command projections for other consumers; native rendering uses
                // exclusively the typed presentation and generation-checked numeric commands.
                field.numeric = Some(Ok(presentation));
            }
            Err(error) => {
                field.numeric = Some(Err(error.to_string()));
                field.commit_command_id = None;
            }
        }
    }
}
