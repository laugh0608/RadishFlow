use super::{ObjectId, VariableField, VariableId, VariableSection};
use crate::{
    WorkspaceState,
    project_query::{ProjectField, ProjectObjectId, ProjectObjectRef, ProjectQuery},
};
use rf_types::ObjectCreationId;
use std::{fmt, str::FromStr};

const FIELDS: &[(&str, VariableField)] = &[
    ("name", VariableField::Name),
    ("outlet_temperature", VariableField::OutletTemperature),
    ("outlet_pressure", VariableField::OutletPressure),
    (
        "outlet_a_split_fraction",
        VariableField::OutletASplitFraction,
    ),
    ("temperature", VariableField::Temperature),
    ("pressure", VariableField::Pressure),
    ("molar_flow", VariableField::MolarFlow),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableVariableReference {
    variable: VariableId,
    creation: ObjectCreationId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableReferenceError {
    Malformed,
    UnsupportedVersion,
    UnsupportedField,
    DifferentDocument,
    IdentitiesUnavailable,
    ObjectMissing,
    ObjectNotSaved,
    ObjectRecreated,
    FieldMissing,
}

impl StableVariableReference {
    pub fn for_saved_variable(
        workspace: &WorkspaceState,
        variable: VariableId,
    ) -> Result<Self, VariableReferenceError> {
        check_target(workspace, &variable)?;
        let creation = workspace
            .saved_object_creation_id(&variable.object)
            .ok_or(VariableReferenceError::ObjectNotSaved)?
            .clone();
        check_field(workspace, &variable)?;
        Ok(Self { variable, creation })
    }

    pub fn resolve(
        &self,
        workspace: &WorkspaceState,
    ) -> Result<VariableId, VariableReferenceError> {
        check_target(workspace, &self.variable)?;
        if workspace.object_creation_id(&self.variable.object) != Some(&self.creation) {
            return Err(VariableReferenceError::ObjectRecreated);
        }
        check_field(workspace, &self.variable)?;
        Ok(self.variable.clone())
    }

    pub fn supports(variable: &VariableId) -> bool {
        use VariableField::*;
        use VariableSection::*;
        matches!(
            (&variable.object, variable.section, &variable.field),
            (
                ObjectId::Unit(_),
                Inputs,
                Name | OutletTemperature | OutletPressure | OutletASplitFraction
            ) | (ObjectId::Stream(_), Inputs, Name)
                | (ObjectId::Stream(_), _, Temperature | Pressure | MolarFlow)
        )
    }
}

fn object_ref(variable: &VariableId) -> ProjectObjectRef {
    ProjectObjectRef {
        document: variable.document.clone(),
        object: ProjectObjectId::Flow(variable.object.clone()),
    }
}

fn check_target(
    workspace: &WorkspaceState,
    variable: &VariableId,
) -> Result<(), VariableReferenceError> {
    if !StableVariableReference::supports(variable) {
        return Err(VariableReferenceError::UnsupportedField);
    }
    if variable.document != workspace.document.metadata.document_id {
        return Err(VariableReferenceError::DifferentDocument);
    }
    if !workspace.object_identities_enabled() {
        return Err(VariableReferenceError::IdentitiesUnavailable);
    }
    ProjectQuery::new(workspace)
        .record(&object_ref(variable))
        .map_err(|_| VariableReferenceError::ObjectMissing)?;
    Ok(())
}

fn check_field(
    workspace: &WorkspaceState,
    variable: &VariableId,
) -> Result<(), VariableReferenceError> {
    ProjectQuery::new(workspace)
        .read_field(
            &object_ref(variable),
            &ProjectField::Flow {
                section: variable.section,
                field: variable.field.clone(),
            },
        )
        .map_err(|_| VariableReferenceError::FieldMissing)?;
    Ok(())
}

impl fmt::Display for StableVariableReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, object) = match &self.variable.object {
            ObjectId::Unit(id) => ("unit", id.as_str()),
            ObjectId::Stream(id) => ("stream", id.as_str()),
        };
        let section = match self.variable.section {
            VariableSection::Inputs => "inputs",
            VariableSection::Results => "results",
        };
        let field = FIELDS
            .iter()
            .find(|(_, field)| field == &self.variable.field)
            .expect("a stable reference contains a supported field")
            .0;
        write!(
            f,
            "rfvar:1/{}/{kind}/{}/{}/{section}/{field}",
            encode_segment(self.variable.document.as_str()),
            encode_segment(object),
            self.creation
        )
    }
}

impl FromStr for StableVariableReference {
    type Err = VariableReferenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        use VariableReferenceError::{Malformed, UnsupportedField, UnsupportedVersion};
        let mut parts = value.split('/');
        let version = parts
            .next()
            .and_then(|prefix| prefix.strip_prefix("rfvar:"))
            .ok_or(Malformed)?;
        if version.is_empty() || !version.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(Malformed);
        }
        if version != "1" {
            return Err(UnsupportedVersion);
        }
        let document = decode_segment(parts.next().ok_or(Malformed)?)?;
        let kind = parts.next().ok_or(Malformed)?;
        let object = decode_segment(parts.next().ok_or(Malformed)?)?;
        let object = match kind {
            "unit" => ObjectId::Unit(object.into()),
            "stream" => ObjectId::Stream(object.into()),
            _ => return Err(Malformed),
        };
        let creation = parts
            .next()
            .ok_or(Malformed)?
            .parse()
            .map_err(|_| Malformed)?;
        let section = match parts.next() {
            Some("inputs") => VariableSection::Inputs,
            Some("results") => VariableSection::Results,
            _ => return Err(Malformed),
        };
        let token = parts
            .next()
            .filter(|part| !part.is_empty())
            .ok_or(Malformed)?;
        if parts.next().is_some()
            || !token
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(Malformed);
        }
        let field = FIELDS
            .iter()
            .find(|(name, _)| *name == token)
            .ok_or(UnsupportedField)?
            .1
            .clone();
        let variable = VariableId {
            document: document.into(),
            object,
            section,
            field,
        };
        if !Self::supports(&variable) {
            return Err(UnsupportedField);
        }
        Ok(Self { variable, creation })
    }
}

fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

fn encode_segment(value: &str) -> String {
    use fmt::Write as _;
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if unreserved(byte) {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").expect("writing to a string is infallible");
        }
    }
    encoded
}

fn decode_segment(encoded: &str) -> Result<String, VariableReferenceError> {
    use VariableReferenceError::Malformed;
    fn hex(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }
    if encoded.is_empty() {
        return Err(Malformed);
    }
    let mut bytes = encoded.bytes();
    let mut decoded = Vec::with_capacity(encoded.len());
    while let Some(byte) = bytes.next() {
        if unreserved(byte) {
            decoded.push(byte);
        } else if byte == b'%' {
            let high = bytes.next().and_then(hex).ok_or(Malformed)?;
            let low = bytes.next().and_then(hex).ok_or(Malformed)?;
            let value = (high << 4) | low;
            if unreserved(value) {
                return Err(Malformed);
            }
            decoded.push(value);
        } else {
            return Err(Malformed);
        }
    }
    String::from_utf8(decoded).map_err(|_| Malformed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppState, DocumentMetadata, FlowsheetDocument};
    use rf_model::{Flowsheet, MaterialStreamState};
    use std::time::UNIX_EPOCH;

    fn initialized(document: &str, stream: &str) -> AppState {
        let mut flowsheet = Flowsheet::new("reference test");
        flowsheet
            .insert_stream(MaterialStreamState::new(stream, "same label"))
            .unwrap();
        let mut app = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new(document, "References", UNIX_EPOCH),
        ));
        let prepared = app.workspace.prepare_object_identities().unwrap();
        app.workspace.install_saved_object_identities(prepared);
        app
    }

    #[test]
    fn stable_reference_roundtrips_unicode_and_rejects_noncanonical_paths() {
        let document = "工程/ %É";
        let stream = "E\u{301}/% 流股";
        let app = initialized(document, stream);
        let variable = VariableId {
            document: document.into(),
            object: ObjectId::Stream(stream.into()),
            section: VariableSection::Results,
            field: VariableField::Temperature,
        };
        let reference =
            StableVariableReference::for_saved_variable(&app.workspace, variable.clone()).unwrap();
        let text = reference.to_string();
        assert!(text.contains("%2F%20%25"));
        assert!(text.contains("E%CC%81%2F%25%20"));
        assert_eq!(text.parse::<StableVariableReference>().unwrap(), reference);
        assert_eq!(reference.resolve(&app.workspace).unwrap(), variable);
        for invalid in [
            text.replace("%2F", "%2f"),
            text.replace("%2F", "%FF"),
            text.replace("/E%CC", "/%45%CC"),
            format!("{text}/"),
        ] {
            assert_eq!(
                invalid.parse::<StableVariableReference>(),
                Err(VariableReferenceError::Malformed)
            );
        }
        assert_eq!(
            text.replace("rfvar:1", "rfvar:2")
                .parse::<StableVariableReference>(),
            Err(VariableReferenceError::UnsupportedVersion)
        );
        assert_eq!(
            text.replace("/temperature", "/name")
                .parse::<StableVariableReference>(),
            Err(VariableReferenceError::UnsupportedField)
        );

        let independent = initialized(document, stream);
        assert_eq!(
            reference.resolve(&independent.workspace),
            Err(VariableReferenceError::ObjectRecreated)
        );
        let other = initialized("other", stream);
        assert_eq!(
            reference.resolve(&other.workspace),
            Err(VariableReferenceError::DifferentDocument)
        );
    }
}
