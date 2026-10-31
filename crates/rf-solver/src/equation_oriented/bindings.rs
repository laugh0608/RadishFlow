use super::*;
use std::collections::BTreeSet;
impl EquationOptions {
    pub fn referenced_unit_ids(&self) -> BTreeSet<UnitId> {
        let mut ids = BTreeSet::new();
        if let Some(r) = &self.released {
            ids.insert(r.binding.unit.clone());
        }
        if let Some(t) = &self.target {
            ids.insert(t.source.unit.clone());
        }
        if let Some(scales) = &self.scales {
            for v in scales.variables.keys() {
                match v {
                    VariableBinding::PortComponent { port, .. } => {
                        ids.insert(port.unit.clone());
                    }
                    VariableBinding::Parameter(p) => {
                        ids.insert(p.unit.clone());
                    }
                }
            }
            for e in scales.equations.keys() {
                match e {
                    EquationBinding::Unit { unit, .. } => {
                        ids.insert(unit.clone());
                    }
                    EquationBinding::Connection { source, sink, .. } => {
                        ids.insert(source.unit.clone());
                        ids.insert(sink.unit.clone());
                    }
                    EquationBinding::ProductTotal { source } => {
                        ids.insert(source.unit.clone());
                    }
                }
            }
        }
        ids
    }
    pub fn referenced_stream_ids(&self) -> BTreeSet<StreamId> {
        let mut ids = BTreeSet::new();
        if let Some(t) = &self.target {
            ids.insert(t.source.stream.clone());
        }
        if let Some(scales) = &self.scales {
            for v in scales.variables.keys() {
                if let VariableBinding::PortComponent { port, .. } = v {
                    ids.insert(port.stream.clone());
                }
            }
            for e in scales.equations.keys() {
                match e {
                    EquationBinding::Connection { source, sink, .. } => {
                        ids.insert(source.stream.clone());
                        ids.insert(sink.stream.clone());
                    }
                    EquationBinding::ProductTotal { source } => {
                        ids.insert(source.stream.clone());
                    }
                    EquationBinding::Unit { .. } => {}
                }
            }
        }
        ids
    }
    pub(super) fn validate_identities(&self) -> EoResult<()> {
        let blank = self
            .referenced_unit_ids()
            .iter()
            .any(|id| id.as_str().trim().is_empty())
            || self
                .referenced_stream_ids()
                .iter()
                .any(|id| id.as_str().trim().is_empty());
        let mut invalid_component = false;
        if let Some(scales) = &self.scales {
            invalid_component=scales.variables.keys().any(|v|matches!(v,VariableBinding::PortComponent {component,..} if component.as_str().trim().is_empty())) || scales.equations.keys().any(|e|matches!(e,EquationBinding::Unit {component,..}|EquationBinding::Connection {component,..} if component.as_str().trim().is_empty()));
            if scales.equations.keys().any(|e|matches!(e,EquationBinding::Connection {source,sink,..} if source.stream!=sink.stream)) {
                return Err(error(EquationFailureKind::InvalidBinding,"connection_identity","connection endpoints must refer to the same stream"));
            }
        }
        if blank || invalid_component {
            return Err(error(
                EquationFailureKind::InvalidBinding,
                "blank_identity",
                "EO identities must be nonblank",
            ));
        }
        Ok(())
    }
}
