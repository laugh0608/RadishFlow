use rf_types::RfResult;

use crate::{
    EntitlementPreflightOutcome, EntitlementSessionHostRuntime, EntitlementSessionHostTimerEffect,
    StudioBootstrapConfig, StudioBootstrapReport, StudioBootstrapTrigger,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRuntimeEffect {
    EntitlementTimer(EntitlementSessionHostTimerEffect),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRuntimeOutput {
    pub trigger: StudioBootstrapTrigger,
    pub report: StudioBootstrapReport,
    pub effects: Vec<StudioRuntimeEffect>,
}

impl StudioRuntimeOutput {
    fn from_report(trigger: &StudioBootstrapTrigger, report: StudioBootstrapReport) -> Self {
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
    pub fn new(config: &StudioBootstrapConfig) -> RfResult<Self> {
        Ok(Self {
            session: crate::bootstrap::BootstrapSession::new(config)?,
        })
    }

    pub fn dispatch_trigger(
        &mut self,
        trigger: &StudioBootstrapTrigger,
    ) -> RfResult<StudioBootstrapReport> {
        self.dispatch_trigger_output(trigger)
            .map(|output| output.report)
    }

    pub fn dispatch_trigger_output(
        &mut self,
        trigger: &StudioBootstrapTrigger,
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
    use crate::bootstrap::{StudioBootstrapEntitlementPreflight, StudioBootstrapEntitlementSeed};
    use crate::{
        EntitlementPreflightAction, EntitlementSessionEventOutcome,
        EntitlementSessionHostTimerEffect, StudioBootstrapConfig, StudioBootstrapDispatch,
        StudioBootstrapEntitlementSessionEvent, StudioBootstrapTrigger, StudioRuntime,
        StudioRuntimeEffect,
    };

    #[test]
    fn studio_runtime_replays_entitlement_host_event_sequence_with_shared_state() {
        let config = StudioBootstrapConfig {
            entitlement_preflight: StudioBootstrapEntitlementPreflight::Skip,
            entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
            ..StudioBootstrapConfig::default()
        };
        let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");

        let timer_elapsed = runtime
            .dispatch_trigger(&StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected timer elapsed event");
        let network_restored = runtime
            .dispatch_trigger(&StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::NetworkRestored,
            ))
            .expect("expected network restored event");
        let window_foregrounded = runtime
            .dispatch_trigger(&StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::WindowForegrounded,
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
        let config = StudioBootstrapConfig {
            entitlement_preflight: StudioBootstrapEntitlementPreflight::Skip,
            entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
            ..StudioBootstrapConfig::default()
        };
        let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");

        let timer_elapsed = runtime
            .dispatch_trigger_output(&StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected timer elapsed output");
        let network_restored = runtime
            .dispatch_trigger_output(&StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::NetworkRestored,
            ))
            .expect("expected network restored output");

        assert_eq!(
            timer_elapsed.trigger,
            StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::TimerElapsed
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
        report: &crate::StudioBootstrapReport,
    ) -> &crate::EntitlementSessionEventDriverOutcome {
        match &report.dispatch {
            StudioBootstrapDispatch::EntitlementSessionEvent(outcome) => outcome,
            StudioBootstrapDispatch::AppCommand(_) => {
                panic!("expected entitlement session event dispatch")
            }
        }
    }
}
