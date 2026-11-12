use rf_model::Flowsheet;
use rf_types::{FlowObjectIdentities, ObjectCreationId, RfError, RfResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ObjectIdentitiesWire {
    units: Vec<IdentityWire>,
    streams: Vec<IdentityWire>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IdentityWire {
    object_id: String,
    creation_id: String,
}

impl ObjectIdentitiesWire {
    pub(super) fn decode(self, flowsheet: &Flowsheet) -> RfResult<FlowObjectIdentities> {
        let identities = FlowObjectIdentities {
            units: decode_entries(self.units)?,
            streams: decode_entries(self.streams)?,
        };
        validate(&identities, flowsheet)?;
        Ok(identities)
    }

    pub(super) fn encode(
        identities: &FlowObjectIdentities,
        flowsheet: &Flowsheet,
    ) -> RfResult<Self> {
        validate(identities, flowsheet)?;
        Ok(Self {
            units: encode_entries(&identities.units),
            streams: encode_entries(&identities.streams),
        })
    }
}

fn decode_entries<K: From<String> + Ord>(
    entries: Vec<IdentityWire>,
) -> RfResult<BTreeMap<K, ObjectCreationId>> {
    let mut result = BTreeMap::new();
    for entry in entries {
        let creation = entry.creation_id.parse().map_err(|error: RfError| {
            RfError::invalid_input(format!("objectIdentities: {}: {error}", entry.object_id))
        })?;
        if result.insert(entry.object_id.into(), creation).is_some() {
            return Err(RfError::invalid_input(
                "objectIdentities: duplicate object ID",
            ));
        }
    }
    Ok(result)
}

fn encode_entries<K: AsRef<str>>(entries: &BTreeMap<K, ObjectCreationId>) -> Vec<IdentityWire> {
    entries
        .iter()
        .map(|(object, creation)| IdentityWire {
            object_id: object.as_ref().to_owned(),
            creation_id: creation.to_string(),
        })
        .collect()
}

fn validate(identities: &FlowObjectIdentities, flowsheet: &Flowsheet) -> RfResult<()> {
    if !identities.units.keys().eq(flowsheet.units.keys())
        || !identities.streams.keys().eq(flowsheet.streams.keys())
    {
        return Err(RfError::invalid_input(
            "objectIdentities: records must match the stored units and streams",
        ));
    }
    let mut seen = BTreeSet::new();
    if identities
        .units
        .values()
        .chain(identities.streams.values())
        .any(|id| !seen.insert(id))
    {
        return Err(RfError::invalid_input(
            "objectIdentities: duplicate creation ID",
        ));
    }
    Ok(())
}
