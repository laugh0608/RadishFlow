use rf_types::RfResult;

use crate::{
    EntitlementPreflightOutcome, EntitlementSessionHostRuntime, StudioBootstrapConfig,
    StudioBootstrapReport, StudioBootstrapTrigger,
};

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
        self.session.run_trigger(trigger)
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
