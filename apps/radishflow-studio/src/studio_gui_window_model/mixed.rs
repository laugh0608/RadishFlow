#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowMixedModel {
    pub evidence_lines: Vec<String>,
    pub result: std::sync::Arc<rf_solver::mixed::MixedResult>,
    pub units: Vec<super::StudioGuiWindowUnitExecutionResultModel>,
}
