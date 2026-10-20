//! The common Souders–Brown relation. Caller-specific failures retain the exact
//! arithmetic step; callers separately establish source and applicability validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VelocityStep {
    DensityRatio,
    SoudersBrownVelocity,
    AllowedVelocity,
}
pub(crate) struct GasVelocities {
    pub souders_brown_m_s: f64,
    pub allowed_m_s: f64,
}

pub(crate) fn gas_velocities(
    gas_density: f64,
    liquid_density: f64,
    effective_k: f64,
    utilization: f64,
) -> Result<GasVelocities, VelocityStep> {
    fn positive(value: f64, step: VelocityStep) -> Result<f64, VelocityStep> {
        if value.is_finite() && value > 0.0 {
            Ok(value)
        } else {
            Err(step)
        }
    }
    let ratio = positive(
        (liquid_density - gas_density) / gas_density,
        VelocityStep::DensityRatio,
    )?;
    let souders_brown_m_s = positive(
        effective_k * ratio.sqrt(),
        VelocityStep::SoudersBrownVelocity,
    )?;
    let allowed_m_s = positive(
        utilization * souders_brown_m_s,
        VelocityStep::AllowedVelocity,
    )?;
    Ok(GasVelocities {
        souders_brown_m_s,
        allowed_m_s,
    })
}
