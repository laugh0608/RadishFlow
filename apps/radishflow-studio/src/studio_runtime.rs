use rf_types::RfResult;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    EntitlementPreflightOutcome, EntitlementSessionHostRuntime, EntitlementSessionHostTimerEffect,
    EntitlementSessionTimerArm,
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
pub enum StudioRuntimeTimerHostCommand {
    KeepTimer {
        effect_id: StudioRuntimeHostEffectId,
        timer: EntitlementSessionTimerArm,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
    },
    ArmTimer {
        effect_id: StudioRuntimeHostEffectId,
        timer: EntitlementSessionTimerArm,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
    },
    RearmTimer {
        effect_id: StudioRuntimeHostEffectId,
        previous: EntitlementSessionTimerArm,
        next: EntitlementSessionTimerArm,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
    },
    ClearTimer {
        effect_id: StudioRuntimeHostEffectId,
        previous: EntitlementSessionTimerArm,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
    },
}

impl StudioRuntimeTimerHostCommand {
    pub fn effect_id(&self) -> StudioRuntimeHostEffectId {
        match self {
            Self::KeepTimer { effect_id, .. }
            | Self::ArmTimer { effect_id, .. }
            | Self::RearmTimer { effect_id, .. }
            | Self::ClearTimer { effect_id, .. } => *effect_id,
        }
    }

    pub fn follow_up_trigger(&self) -> Option<&StudioRuntimeTrigger> {
        match self {
            Self::KeepTimer {
                follow_up_trigger, ..
            }
            | Self::ArmTimer {
                follow_up_trigger, ..
            }
            | Self::RearmTimer {
                follow_up_trigger, ..
            }
            | Self::ClearTimer {
                follow_up_trigger, ..
            } => follow_up_trigger.as_ref(),
        }
    }

    fn from_host_effect(effect: &StudioRuntimeHostEffect) -> Option<Self> {
        let follow_up_trigger = effect
            .follow_up
            .as_ref()
            .map(|StudioRuntimeHostFollowUp::DispatchTrigger(trigger)| trigger.clone());

        match &effect.effect {
            StudioRuntimeEffect::EntitlementTimer(timer_effect) => match timer_effect {
                EntitlementSessionHostTimerEffect::KeepTimer { timer } => Some(Self::KeepTimer {
                    effect_id: effect.id,
                    timer: timer.clone(),
                    follow_up_trigger,
                }),
                EntitlementSessionHostTimerEffect::ArmTimer { timer } => Some(Self::ArmTimer {
                    effect_id: effect.id,
                    timer: timer.clone(),
                    follow_up_trigger,
                }),
                EntitlementSessionHostTimerEffect::RearmTimer { previous, next } => {
                    Some(Self::RearmTimer {
                        effect_id: effect.id,
                        previous: previous.clone(),
                        next: next.clone(),
                        follow_up_trigger,
                    })
                }
                EntitlementSessionHostTimerEffect::ClearTimer { previous } => {
                    Some(Self::ClearTimer {
                        effect_id: effect.id,
                        previous: previous.clone(),
                        follow_up_trigger,
                    })
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRuntimeTimerHandleSlot {
    pub effect_id: StudioRuntimeHostEffectId,
    pub timer: EntitlementSessionTimerArm,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioRuntimeTimerHostState {
    entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRuntimeTimerHostTransition {
    KeepTimer {
        slot: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
    },
    ArmTimer {
        slot: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
    },
    RearmTimer {
        previous: Option<StudioRuntimeTimerHandleSlot>,
        next: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
    },
    ClearTimer {
        previous: Option<StudioRuntimeTimerHandleSlot>,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
    },
    IgnoreStale {
        current: Option<StudioRuntimeTimerHandleSlot>,
        stale_effect_id: StudioRuntimeHostEffectId,
    },
}

impl StudioRuntimeTimerHostState {
    pub fn entitlement_timer(&self) -> Option<&StudioRuntimeTimerHandleSlot> {
        self.entitlement_timer.as_ref()
    }

    pub fn restore(&mut self, slot: StudioRuntimeTimerHandleSlot) {
        self.entitlement_timer = Some(slot);
    }

    pub fn clear(&mut self) -> Option<StudioRuntimeTimerHandleSlot> {
        self.entitlement_timer.take()
    }

    pub fn apply_command(
        &mut self,
        command: &StudioRuntimeTimerHostCommand,
    ) -> StudioRuntimeTimerHostTransition {
        if self.is_stale(command.effect_id()) {
            return StudioRuntimeTimerHostTransition::IgnoreStale {
                current: self.entitlement_timer.clone(),
                stale_effect_id: command.effect_id(),
            };
        }

        match command {
            StudioRuntimeTimerHostCommand::KeepTimer {
                effect_id,
                timer,
                follow_up_trigger,
            } => {
                let slot = StudioRuntimeTimerHandleSlot {
                    effect_id: *effect_id,
                    timer: timer.clone(),
                };
                self.entitlement_timer = Some(slot.clone());
                StudioRuntimeTimerHostTransition::KeepTimer {
                    slot,
                    follow_up_trigger: follow_up_trigger.clone(),
                }
            }
            StudioRuntimeTimerHostCommand::ArmTimer {
                effect_id,
                timer,
                follow_up_trigger,
            } => {
                let slot = StudioRuntimeTimerHandleSlot {
                    effect_id: *effect_id,
                    timer: timer.clone(),
                };
                self.entitlement_timer = Some(slot.clone());
                StudioRuntimeTimerHostTransition::ArmTimer {
                    slot,
                    follow_up_trigger: follow_up_trigger.clone(),
                }
            }
            StudioRuntimeTimerHostCommand::RearmTimer {
                effect_id,
                next,
                follow_up_trigger,
                ..
            } => {
                let previous = self.entitlement_timer.clone();
                let slot = StudioRuntimeTimerHandleSlot {
                    effect_id: *effect_id,
                    timer: next.clone(),
                };
                self.entitlement_timer = Some(slot.clone());
                StudioRuntimeTimerHostTransition::RearmTimer {
                    previous,
                    next: slot,
                    follow_up_trigger: follow_up_trigger.clone(),
                }
            }
            StudioRuntimeTimerHostCommand::ClearTimer {
                effect_id: _,
                follow_up_trigger,
                ..
            } => {
                let previous = self.entitlement_timer.take();
                StudioRuntimeTimerHostTransition::ClearTimer {
                    previous,
                    follow_up_trigger: follow_up_trigger.clone(),
                }
            }
        }
    }

    fn is_stale(&self, effect_id: StudioRuntimeHostEffectId) -> bool {
        self.entitlement_timer
            .as_ref()
            .map(|slot| slot.effect_id > effect_id)
            .unwrap_or(false)
    }
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

    pub fn entitlement_timer_host_command(&self) -> Option<StudioRuntimeTimerHostCommand> {
        self.entitlement_timer_effect()
            .and_then(StudioRuntimeTimerHostCommand::from_host_effect)
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

    pub fn refresh_local_canvas_suggestions(&mut self) {
        self.session.refresh_local_canvas_suggestions();
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<rf_ui::CanvasSuggestion>) {
        self.session.replace_canvas_suggestions(suggestions);
    }

    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        self.session.accept_focused_canvas_suggestion_by_tab()
    }

    pub fn reject_focused_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.session.reject_focused_canvas_suggestion()
    }

    pub fn focus_next_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.session.focus_next_canvas_suggestion()
    }

    pub fn focus_previous_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.session.focus_previous_canvas_suggestion()
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

    pub fn pending_entitlement_timer_host_command(&self) -> Option<StudioRuntimeTimerHostCommand> {
        self.pending_host_effects()
            .into_iter()
            .find_map(|effect| StudioRuntimeTimerHostCommand::from_host_effect(&effect))
    }

    pub fn acknowledge_entitlement_timer_host_command(
        &mut self,
        command: &StudioRuntimeTimerHostCommand,
    ) -> StudioRuntimeHostAckResult {
        self.acknowledge_host_effect(command.effect_id())
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
        StudioRuntimeHostAckStatus, StudioRuntimeHostFollowUp, StudioRuntimeTimerHostCommand,
        StudioRuntimeTimerHostState, StudioRuntimeTimerHostTransition, StudioRuntimeTrigger,
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
        assert!(matches!(
            network_restored.entitlement_timer_host_command(),
            Some(StudioRuntimeTimerHostCommand::KeepTimer { .. })
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
        assert!(runtime.pending_entitlement_timer_host_command().is_none());
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

    #[test]
    fn studio_runtime_exposes_pending_entitlement_timer_host_command() {
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
        let output_command = output
            .entitlement_timer_host_command()
            .expect("expected timer host command from output");
        let pending_command = runtime
            .pending_entitlement_timer_host_command()
            .expect("expected pending timer host command");

        assert_eq!(pending_command, output_command);
        assert!(matches!(
            pending_command,
            StudioRuntimeTimerHostCommand::RearmTimer { .. }
        ));
    }

    #[test]
    fn timer_host_state_applies_pending_command_and_tracks_current_slot() {
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
        let command = output
            .entitlement_timer_host_command()
            .expect("expected timer host command");
        let mut host_state = StudioRuntimeTimerHostState::default();

        let transition = host_state.apply_command(&command);
        assert!(matches!(
            transition,
            StudioRuntimeTimerHostTransition::RearmTimer { .. }
        ));
        let slot = host_state
            .entitlement_timer()
            .expect("expected current timer slot after apply");
        assert_eq!(slot.effect_id, command.effect_id());
        assert_eq!(
            runtime
                .acknowledge_entitlement_timer_host_command(&command)
                .status,
            StudioRuntimeHostAckStatus::Applied
        );
    }

    #[test]
    fn timer_host_state_ignores_stale_command_after_newer_timer_is_applied() {
        let config = StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        };
        let mut runtime = StudioRuntime::new(&config).expect("expected studio runtime");
        let first_output = runtime
            .dispatch_trigger_output(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected first output");
        let first_command = first_output
            .entitlement_timer_host_command()
            .expect("expected first timer command");
        let second_output = runtime
            .dispatch_trigger_output(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::NetworkRestored,
            ))
            .expect("expected second output");
        let second_command = second_output
            .entitlement_timer_host_command()
            .expect("expected second timer command");
        let mut host_state = StudioRuntimeTimerHostState::default();

        let _ = host_state.apply_command(&second_command);
        let stale_transition = host_state.apply_command(&first_command);

        assert!(matches!(
            stale_transition,
            StudioRuntimeTimerHostTransition::IgnoreStale { .. }
        ));
        assert_eq!(
            host_state.entitlement_timer().map(|slot| slot.effect_id),
            Some(second_command.effect_id())
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
            StudioRuntimeDispatch::RunPanelRecovery(_) => {
                panic!("expected entitlement session event dispatch")
            }
            StudioRuntimeDispatch::InspectorTarget(_) => {
                panic!("expected entitlement session event dispatch")
            }
            StudioRuntimeDispatch::InspectorDraftUpdate(_) => {
                panic!("expected entitlement session event dispatch")
            }
        }
    }
}
