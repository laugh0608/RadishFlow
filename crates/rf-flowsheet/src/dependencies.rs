//! Material connections retain stream/port identity; unit dependencies deduplicate
//! parallel material edges. Execution strategies do not belong in this view.
use crate::{MaterialConnection, validate_connections};
use rf_model::Flowsheet;
use rf_types::{RfResult, UnitId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDependencies {
    pub connections: Vec<MaterialConnection>,
    pub successors: BTreeMap<UnitId, BTreeSet<UnitId>>,
}

pub fn material_dependencies(flowsheet: &Flowsheet) -> RfResult<UnitDependencies> {
    let connections = validate_connections(flowsheet)?;
    let mut successors: BTreeMap<_, BTreeSet<_>> = flowsheet
        .units
        .keys()
        .cloned()
        .map(|id| (id, BTreeSet::new()))
        .collect();
    for connection in &connections {
        if let Some(sink) = &connection.sink {
            successors
                .get_mut(&connection.source.unit_id)
                .expect("validated source")
                .insert(sink.unit_id.clone());
        }
    }
    Ok(UnitDependencies {
        connections,
        successors,
    })
}
