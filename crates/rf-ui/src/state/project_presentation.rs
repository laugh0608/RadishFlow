use rf_types::units::DisplayUnitSet;

/// Loaded project presentation, kept separate from engineering inputs and solve revision.
/// I1 exposes read-only choices; settings, saved baselines and presentation history follow in I2.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectPresentationState {
    display_units: DisplayUnitSet,
    source_file_version: Option<u32>,
}

impl ProjectPresentationState {
    /// Called by the application bridge after the storage layer validates the file.
    pub fn from_loaded(display_units: DisplayUnitSet, source_file_version: u32) -> Self {
        Self {
            display_units,
            source_file_version: Some(source_file_version),
        }
    }

    pub fn display_units(&self) -> &DisplayUnitSet {
        &self.display_units
    }

    pub fn source_file_version(&self) -> Option<u32> {
        self.source_file_version
    }
}
