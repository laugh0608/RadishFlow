//! C1 adoption uses the current candidate and the domain's new confirmation key.
use super::{actions::*, equipment, protocol::*, references::Bindings};
use rf_ui::{
    AppState,
    engineering::{DimensionAdjustment, GeometryAdjustments, editor::*},
};

pub fn prepare(
    app: &AppState,
    refs: &Bindings,
    source: &IdentityRef,
    revision: &RevisionRef,
    target: &AdoptionTarget,
    adjustments: &Adjustments,
) -> Result<EquipmentAction, Failure> {
    let source: rf_types::EquipmentTaskId = refs.resolve(source, Output::Task)?.into();
    equipment::guard(
        app,
        &EquipmentEditorTarget::Task(source.clone()),
        revision,
        refs,
    )?;
    let target = match target {
        AdoptionTarget::New {
            tag,
            name,
            copy_case,
        } => GeometryAdoptionDestination::New {
            tag: tag.clone(),
            name: name.clone(),
            copy_case: *copy_case,
        },
        AdoptionTarget::Existing {
            task,
            expected_revision,
        } => {
            let id: rf_types::EquipmentTaskId = refs.resolve(task, Output::Task)?.into();
            equipment::guard(
                app,
                &EquipmentEditorTarget::RatingGeometry(id.clone()),
                expected_revision,
                refs,
            )?;
            GeometryAdoptionDestination::Existing(id)
        }
    };
    let dimension = |v: &Adjustment| DimensionAdjustment {
        value_m: v.value_m,
        reason: v.reason.clone(),
    };
    Ok(EquipmentAction::PreviewAdoption {
        source,
        target,
        adjustments: GeometryAdjustments {
            diameter: adjustments.diameter.as_ref().map(dimension),
            working_liquid_height: adjustments.working_liquid_height.as_ref().map(dimension),
        },
    })
}
