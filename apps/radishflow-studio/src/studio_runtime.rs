use rf_types::RfResult;

use crate::{
    EntitlementPreflightOutcome, EntitlementSessionHostRuntime, EntitlementSessionHostTimerEffect,
};

pub type StudioRuntimeConfig = crate::bootstrap::StudioBootstrapConfig;
pub type StudioRuntimeTrigger = crate::bootstrap::StudioBootstrapTrigger;
pub type StudioRuntimeDispatch = crate::bootstrap::StudioBootstrapDispatch;
pub type StudioRuntimeReport = crate::bootstrap::StudioBootstrapReport;
pub type StudioRuntimeEntitlementPreflight = crate::bootstrap::StudioBootstrapEntitlementPreflight;
pub type StudioRuntimeEntitlementSeed = crate::bootstrap::StudioBootstrapEntitlementSeed;
pub type StudioRuntimeEntitlementSessionEvent =
    crate::bootstrap::StudioBootstrapEntitlementSessionEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRuntimeEffect {
    EntitlementTimer(EntitlementSessionHostTimerEffect),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRuntimeOutput {
    pub trigger: StudioRuntimeTrigger,
    pub report: StudioRuntimeReport,
    pub effects: Vec<StudioRuntimeEffect>,
}

impl StudioRuntimeOutput {
    fn from_report(trigger: &StudioRuntimeTrigger, report: StudioRuntimeReport) -> Self {
        let mut effects = Vec::new();
        if let Some(timer_effect) = report.entitlement_host.timer_effect.clone() {
            effects.push(StudioRuntimeEffect::EntitlementTimer(timer_effect));
        }

        Self {
            trigger: trigger.clone(),
            report,
            effects,
        }
    }

    pub fn entitlement_timer_effect(&self) -> Option<&EntitlementSessionHostTimerEffect> {
        self.effects.iter().find_map(|effect| match effect {
            StudioRuntimeEffect::EntitlementTimer(effect) => Some(effect),
        })
    }
}

pub struct StudioRuntime {
    session: crate::bootstrap::BootstrapSession,
}

impl StudioRuntime {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            session: crate::bootstrap::BootstrapSession::new(config)?,
        })
    }

    pub fn dispatch_trigger(
        &mut self,
        trigger: &StudioRuntimeTrigger,
    ) -> RfResult<StudioRuntimeReport> {
        self.dispatch_trigger_output(trigger)
            .map(|output| output.report)
    }

    pub fn dispatch_trigger_output(
        &mut self,
        trigger: &StudioRuntimeTrigger,
    ) -> RfResult<StudioRuntimeOutput> {
        self.session
            .run_trigger(trigger)
            .map(|report| StudioRuntimeOutput::from_report(trigger, report))
    }

    pub fn entitlement_preflight(&self) -> Option<&EntitlementPreflightOutcome> {
        self.session.entitlement_preflight()
    }

    pub fn host_runtime(&self) -> &EntitlementSessionHostRuntime {
        self.session.host_runtime()
    }

    pub fn app_state(&self) -> &rf_ui::AppState {
        self.session.app_state()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        EntitlementPreflightAction, EntitlementSessionEventOutcome,
        EntitlementSessionHostTimerEffect, StudioRuntime, StudioRuntimeConfig,
        StudioRuntimeDispatch, StudioRuntimeEffect, StudioRuntimeEntitlementPreflight,
        StudioRuntimeEntitlementSeed, StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger,
    };

    #[test]
    fn studio_runtime_replays_entitlement_host_event_sequence_with_shared_state() {
        let config = StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        };
        let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");

        let timer_elapsed = runtime
            .dispatch_trigger(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected timer elapsed event");
        let network_restored = runtime
            .dispatch_trigger(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::NetworkRestored,
            ))
            .expect("expected network restored event");
        let window_foregrounded = runtime
            .dispatch_trigger(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::WindowForegrounded,
            ))
            .expect("expected window foregrounded event");

        match &session_event(&timer_elapsed).outcome {
            EntitlementSessionEventOutcome::Tick(tick) => {
                let preflight = tick
                    .preflight
                    .as_ref()
                    .expect("expected refresh preflight on timer elapsed");
                assert_eq!(
                    preflight.decision.action,
                    EntitlementPreflightAction::RefreshOfflineLease
                );
            }
            other => panic!("expected tick outcome, got {other:?}"),
        }
        assert!(matches!(
            timer_elapsed.entitlement_host.timer_effect,
            Some(EntitlementSessionHostTimerEffect::RearmTimer { .. })
        ));

        match &session_event(&network_restored).outcome {
            EntitlementSessionEventOutcome::Tick(tick) => {
                assert!(
                    tick.preflight.is_none(),
                    "expected no preflight after refresh"
                );
            }
            other => panic!("expected tick outcome, got {other:?}"),
        }
        assert!(matches!(
            network_restored.entitlement_host.timer_effect,
            Some(EntitlementSessionHostTimerEffect::KeepTimer { .. })
        ));

        match &session_event(&window_foregrounded).outcome {
            EntitlementSessionEventOutcome::Tick(tick) => {
                assert!(
                    tick.preflight.is_none(),
                    "expected no preflight after refresh"
                );
            }
            other => panic!("expected tick outcome, got {other:?}"),
        }
        assert!(matches!(
            window_foregrounded.entitlement_host.timer_effect,
            Some(EntitlementSessionHostTimerEffect::KeepTimer { .. })
        ));
        assert_eq!(
            network_restored.entitlement_host.snapshot.state.next_timer,
            window_foregrounded
                .entitlement_host
                .snapshot
                .state
                .next_timer
        );
        assert_eq!(
            runtime.host_runtime().current_timer(),
            window_foregrounded
                .entitlement_host
                .snapshot
                .state
                .next_timer
                .as_ref()
        );
    }

    #[test]
    fn studio_runtime_output_surfaces_top_level_entitlement_timer_effects() {
        let config = StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        };
        let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");

        let timer_elapsed = runtime
            .dispatch_trigger_output(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected timer elapsed output");
        let network_restored = runtime
            .dispatch_trigger_output(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::NetworkRestored,
            ))
            .expect("expected network restored output");

        assert_eq!(
            timer_elapsed.trigger,
            StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed
            )
        );
        assert!(matches!(
            timer_elapsed.entitlement_timer_effect(),
            Some(EntitlementSessionHostTimerEffect::RearmTimer { .. })
        ));
        assert_eq!(
            timer_elapsed.effects,
            vec![StudioRuntimeEffect::EntitlementTimer(
                timer_elapsed
                    .report
                    .entitlement_host
                    .timer_effect
                    .clone()
                    .expect("expected timer effect in report"),
            )]
        );

        assert!(matches!(
            network_restored.entitlement_timer_effect(),
            Some(EntitlementSessionHostTimerEffect::KeepTimer { .. })
        ));
    }

    fn session_event(
        report: &crate::StudioRuntimeReport,
    ) -> &crate::EntitlementSessionEventDriverOutcome {
        match &report.dispatch {
            StudioRuntimeDispatch::EntitlementSessionEvent(outcome) => outcome,
            StudioRuntimeDispatch::AppCommand(_) => {
                panic!("expected entitlement session event dispatch")
            }
        }
    }
}
