use rf_types::RfResult;
use std::collections::{BTreeMap, BTreeSet};

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

pub type StudioRuntimeHostEffectId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRuntimeHostFollowUp {
    DispatchTrigger(StudioRuntimeTrigger),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRuntimeHostEffect {
    pub id: StudioRuntimeHostEffectId,
    pub effect: StudioRuntimeEffect,
    pub follow_up: Option<StudioRuntimeHostFollowUp>,
}

impl StudioRuntimeHostEffect {
    pub fn entitlement_timer_effect(&self) -> Option<&EntitlementSessionHostTimerEffect> {
        match &self.effect {
            StudioRuntimeEffect::EntitlementTimer(effect) => Some(effect),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRuntimeHostAckStatus {
    Applied,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRuntimeHostAckResult {
    pub effect_id: StudioRuntimeHostEffectId,
    pub status: StudioRuntimeHostAckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRuntimeOutput {
    pub trigger: StudioRuntimeTrigger,
    pub report: StudioRuntimeReport,
    pub host_effects: Vec<StudioRuntimeHostEffect>,
}

impl StudioRuntimeOutput {
    fn from_report(
        trigger: &StudioRuntimeTrigger,
        report: StudioRuntimeReport,
        host_effects: Vec<StudioRuntimeHostEffect>,
    ) -> Self {
        Self {
            trigger: trigger.clone(),
            report,
            host_effects,
        }
    }

    pub fn entitlement_timer_effect(&self) -> Option<&StudioRuntimeHostEffect> {
        self.host_effects
            .iter()
            .find(|effect| effect.entitlement_timer_effect().is_some())
    }
}

pub struct StudioRuntime {
    session: crate::bootstrap::BootstrapSession,
    next_host_effect_id: StudioRuntimeHostEffectId,
    pending_host_effects: BTreeMap<StudioRuntimeHostEffectId, StudioRuntimeHostEffect>,
    stale_host_effect_ids: BTreeSet<StudioRuntimeHostEffectId>,
    last_applied_host_effect_id: Option<StudioRuntimeHostEffectId>,
}

impl StudioRuntime {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            session: crate::bootstrap::BootstrapSession::new(config)?,
            next_host_effect_id: 1,
            pending_host_effects: BTreeMap::new(),
            stale_host_effect_ids: BTreeSet::new(),
            last_applied_host_effect_id: None,
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
        self.session.run_trigger(trigger).map(|report| {
            let host_effects = self.build_host_effects(&report);
            self.register_host_effects(&host_effects);
            StudioRuntimeOutput::from_report(trigger, report, host_effects)
        })
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

    pub fn acknowledge_host_effect(
        &mut self,
        effect_id: StudioRuntimeHostEffectId,
    ) -> StudioRuntimeHostAckResult {
        if self.pending_host_effects.remove(&effect_id).is_some() {
            self.last_applied_host_effect_id = Some(effect_id);
            return StudioRuntimeHostAckResult {
                effect_id,
                status: StudioRuntimeHostAckStatus::Applied,
            };
        }

        if self.stale_host_effect_ids.remove(&effect_id) {
            return StudioRuntimeHostAckResult {
                effect_id,
                status: StudioRuntimeHostAckStatus::Stale,
            };
        }

        StudioRuntimeHostAckResult {
            effect_id,
            status: StudioRuntimeHostAckStatus::Unknown,
        }
    }

    pub fn pending_host_effects(&self) -> Vec<StudioRuntimeHostEffect> {
        self.pending_host_effects.values().cloned().collect()
    }

    pub fn last_applied_host_effect_id(&self) -> Option<StudioRuntimeHostEffectId> {
        self.last_applied_host_effect_id
    }

    fn build_host_effects(&mut self, report: &StudioRuntimeReport) -> Vec<StudioRuntimeHostEffect> {
        let mut host_effects = Vec::new();

        if let Some(timer_effect) = report.entitlement_host.timer_effect.clone() {
            host_effects.push(StudioRuntimeHostEffect {
                id: self.allocate_host_effect_id(),
                effect: StudioRuntimeEffect::EntitlementTimer(timer_effect),
                follow_up: Some(StudioRuntimeHostFollowUp::DispatchTrigger(
                    StudioRuntimeTrigger::EntitlementSessionEvent(
                        StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                    ),
                )),
            });
        }

        host_effects
    }

    fn register_host_effects(&mut self, host_effects: &[StudioRuntimeHostEffect]) {
        for effect in host_effects {
            if matches!(effect.effect, StudioRuntimeEffect::EntitlementTimer(_)) {
                let pending_ids: Vec<_> = self.pending_host_effects.keys().copied().collect();
                for pending_id in pending_ids {
                    self.pending_host_effects.remove(&pending_id);
                    self.stale_host_effect_ids.insert(pending_id);
                }
            }
            self.pending_host_effects.insert(effect.id, effect.clone());
        }
    }

    fn allocate_host_effect_id(&mut self) -> StudioRuntimeHostEffectId {
        let id = self.next_host_effect_id;
        self.next_host_effect_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        EntitlementPreflightAction, EntitlementSessionEventOutcome,
        EntitlementSessionHostTimerEffect, StudioRuntime, StudioRuntimeConfig,
        StudioRuntimeDispatch, StudioRuntimeEffect, StudioRuntimeEntitlementPreflight,
        StudioRuntimeEntitlementSeed, StudioRuntimeEntitlementSessionEvent,
        StudioRuntimeHostAckStatus, StudioRuntimeHostFollowUp, StudioRuntimeTrigger,
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
            timer_elapsed
                .entitlement_timer_effect()
                .and_then(|effect| effect.entitlement_timer_effect()),
            Some(EntitlementSessionHostTimerEffect::RearmTimer { .. })
        ));
        assert_eq!(timer_elapsed.host_effects.len(), 1);
        let timer_effect = &timer_elapsed.host_effects[0];
        assert_eq!(timer_effect.id, 1);
        assert_eq!(
            timer_effect.follow_up,
            Some(StudioRuntimeHostFollowUp::DispatchTrigger(
                StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed
                )
            ))
        );
        assert_eq!(
            timer_effect.effect,
            StudioRuntimeEffect::EntitlementTimer(
                timer_elapsed
                    .report
                    .entitlement_host
                    .timer_effect
                    .clone()
                    .expect("expected timer effect in report"),
            )
        );

        assert!(matches!(
            network_restored
                .entitlement_timer_effect()
                .and_then(|effect| effect.entitlement_timer_effect()),
            Some(EntitlementSessionHostTimerEffect::KeepTimer { .. })
        ));
    }

    #[test]
    fn studio_runtime_acknowledges_pending_host_effects_and_tracks_last_applied_id() {
        let config = StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        };
        let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");

        let output = runtime
            .dispatch_trigger_output(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected timer elapsed output");
        let effect_id = output.host_effects[0].id;

        let ack = runtime.acknowledge_host_effect(effect_id);
        assert_eq!(ack.status, StudioRuntimeHostAckStatus::Applied);
        assert_eq!(runtime.last_applied_host_effect_id(), Some(effect_id));
        assert!(runtime.pending_host_effects().is_empty());
    }

    #[test]
    fn studio_runtime_marks_replaced_host_effects_as_stale() {
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
        let first_effect_id = timer_elapsed.host_effects[0].id;

        let network_restored = runtime
            .dispatch_trigger_output(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::NetworkRestored,
            ))
            .expect("expected network restored output");
        let second_effect_id = network_restored.host_effects[0].id;

        assert_ne!(first_effect_id, second_effect_id);
        assert_eq!(
            runtime.acknowledge_host_effect(first_effect_id).status,
            StudioRuntimeHostAckStatus::Stale
        );
        assert_eq!(
            runtime.acknowledge_host_effect(second_effect_id).status,
            StudioRuntimeHostAckStatus::Applied
        );
        assert_eq!(
            runtime.acknowledge_host_effect(9_999).status,
            StudioRuntimeHostAckStatus::Unknown
        );
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
