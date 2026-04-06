use rf_types::RfResult;

use crate::{
    StudioRuntimeConfig, StudioRuntimeHostPort, StudioRuntimeHostPortOutput, StudioRuntimeTrigger,
    StudioWindowHostId, StudioWindowHostLifecycleEvent, StudioWindowHostRegistration,
    StudioWindowHostShutdown, StudioWindowHostTimerDriverCommand, StudioWindowNativeTimerHandleId,
    StudioWindowTimerDriverAckResult, StudioWindowTimerDriverAckStatus,
    StudioWindowTimerDriverState, StudioWindowTimerDriverTransition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWindowSessionDispatch {
    pub host_output: StudioRuntimeHostPortOutput,
    pub timer_driver_transitions: Vec<StudioWindowTimerDriverTransition>,
    pub timer_driver_acks: Vec<StudioWindowTimerDriverAckResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWindowSessionShutdown {
    pub host_shutdown: StudioWindowHostShutdown,
    pub timer_driver_transitions: Vec<StudioWindowTimerDriverTransition>,
    pub timer_driver_acks: Vec<StudioWindowTimerDriverAckResult>,
}

pub struct StudioWindowSession {
    host_port: StudioRuntimeHostPort,
    timer_driver_state: StudioWindowTimerDriverState,
    next_native_timer_handle_id: StudioWindowNativeTimerHandleId,
}

impl StudioWindowSession {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            host_port: StudioRuntimeHostPort::new(config)?,
            timer_driver_state: StudioWindowTimerDriverState::default(),
            next_native_timer_handle_id: 1,
        })
    }

    pub fn host_port(&self) -> &StudioRuntimeHostPort {
        &self.host_port
    }

    pub fn timer_driver_state(&self) -> &StudioWindowTimerDriverState {
        &self.timer_driver_state
    }

    pub fn refresh_local_canvas_suggestions(&mut self) {
        self.host_port.refresh_local_canvas_suggestions();
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<rf_ui::CanvasSuggestion>) {
        self.host_port.replace_canvas_suggestions(suggestions);
    }

    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        self.host_port.accept_focused_canvas_suggestion_by_tab()
    }

    pub fn reject_focused_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.host_port.reject_focused_canvas_suggestion()
    }

    pub fn focus_next_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.host_port.focus_next_canvas_suggestion()
    }

    pub fn focus_previous_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.host_port.focus_previous_canvas_suggestion()
    }

    pub fn open_window(&mut self) -> StudioWindowHostRegistration {
        let registration = self.host_port.open_window();
        let _ = self.apply_timer_driver_commands(&registration.timer_driver_commands);
        registration
    }

    pub fn dispatch_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: &StudioRuntimeTrigger,
    ) -> RfResult<StudioWindowSessionDispatch> {
        let host_output = self.host_port.dispatch_trigger(window_id, trigger)?;
        let (timer_driver_transitions, timer_driver_acks) =
            self.apply_timer_driver_commands(&host_output.timer_driver_commands);

        Ok(StudioWindowSessionDispatch {
            host_output,
            timer_driver_transitions,
            timer_driver_acks,
        })
    }

    pub fn dispatch_lifecycle_event(
        &mut self,
        window_id: StudioWindowHostId,
        event: StudioWindowHostLifecycleEvent,
    ) -> RfResult<StudioWindowSessionDispatch> {
        let host_output = self.host_port.dispatch_lifecycle_event(window_id, event)?;
        let (timer_driver_transitions, timer_driver_acks) =
            self.apply_timer_driver_commands(&host_output.timer_driver_commands);

        Ok(StudioWindowSessionDispatch {
            host_output,
            timer_driver_transitions,
            timer_driver_acks,
        })
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> Option<StudioWindowSessionShutdown> {
        let host_shutdown = self.host_port.close_window(window_id)?;
        let (timer_driver_transitions, timer_driver_acks) =
            self.apply_timer_driver_commands(&host_shutdown.timer_driver_commands);

        Some(StudioWindowSessionShutdown {
            host_shutdown,
            timer_driver_transitions,
            timer_driver_acks,
        })
    }

    fn apply_timer_driver_commands(
        &mut self,
        commands: &[StudioWindowHostTimerDriverCommand],
    ) -> (
        Vec<StudioWindowTimerDriverTransition>,
        Vec<StudioWindowTimerDriverAckResult>,
    ) {
        let mut transitions = Vec::new();
        let mut acks = Vec::new();

        for command in commands {
            let transition = self.timer_driver_state.apply_command(command);
            if let Some(window_id) = pending_ack_window_id(&transition) {
                let handle_id = self.next_native_timer_handle_id;
                self.next_native_timer_handle_id += 1;
                let ack = self
                    .timer_driver_state
                    .acknowledge_native_timer(window_id, handle_id);
                if ack.status == StudioWindowTimerDriverAckStatus::Applied {
                    acks.push(ack);
                }
            }
            transitions.push(transition);
        }

        (transitions, acks)
    }
}

fn pending_ack_window_id(
    transition: &StudioWindowTimerDriverTransition,
) -> Option<StudioWindowHostId> {
    match transition {
        StudioWindowTimerDriverTransition::ArmNativeTimer { window_id, .. }
        | StudioWindowTimerDriverTransition::RearmNativeTimer { window_id, .. } => Some(*window_id),
        StudioWindowTimerDriverTransition::KeepNativeTimer { .. }
        | StudioWindowTimerDriverTransition::ClearNativeTimer { .. }
        | StudioWindowTimerDriverTransition::IgnoreStale { .. }
        | StudioWindowTimerDriverTransition::TransferNativeTimer { .. }
        | StudioWindowTimerDriverTransition::ParkNativeTimer { .. }
        | StudioWindowTimerDriverTransition::RestoreParkedTimer { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger, StudioWindowHostLifecycleEvent,
        StudioWindowHostRetirement, StudioWindowHostRole, StudioWindowSession,
        StudioWindowTimerDriverAckStatus, StudioWindowTimerDriverTransition,
    };

    fn lease_expiring_config() -> crate::StudioRuntimeConfig {
        crate::StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..crate::StudioRuntimeConfig::default()
        }
    }

    #[test]
    fn window_session_dispatches_timer_effects_and_acknowledges_native_handle() {
        let mut session =
            StudioWindowSession::new(&lease_expiring_config()).expect("expected window session");
        let window = session.open_window();

        let dispatch = session
            .dispatch_trigger(
                window.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer elapsed dispatch");

        assert!(matches!(
            dispatch.timer_driver_transitions.as_slice(),
            [StudioWindowTimerDriverTransition::RearmNativeTimer { window_id, next_slot, .. }]
            if *window_id == window.window_id && next_slot.effect_id == 1
        ));
        assert_eq!(dispatch.timer_driver_acks.len(), 1);
        assert_eq!(
            dispatch.timer_driver_acks[0].status,
            StudioWindowTimerDriverAckStatus::Applied
        );
        assert_eq!(
            session
                .timer_driver_state()
                .window_binding(window.window_id)
                .map(|binding| binding.handle_id),
            Some(1)
        );
    }

    #[test]
    fn window_session_routes_lifecycle_events_through_single_adapter_entry() {
        let mut session =
            StudioWindowSession::new(&lease_expiring_config()).expect("expected window session");
        let window = session.open_window();

        let dispatch = session
            .dispatch_lifecycle_event(
                window.window_id,
                StudioWindowHostLifecycleEvent::TimerElapsed,
            )
            .expect("expected lifecycle dispatch");

        assert_eq!(
            dispatch.host_output.runtime_output.trigger,
            StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed
            )
        );
        assert_eq!(dispatch.timer_driver_acks.len(), 1);
    }

    #[test]
    fn window_session_close_transfers_timer_binding_to_remaining_window() {
        let mut session =
            StudioWindowSession::new(&lease_expiring_config()).expect("expected window session");
        let first = session.open_window();
        let second = session.open_window();
        assert_eq!(first.role, StudioWindowHostRole::EntitlementTimerOwner);
        assert_eq!(second.role, StudioWindowHostRole::Observer);

        let _ = session
            .dispatch_trigger(
                first.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer elapsed dispatch");
        let shutdown = session
            .close_window(first.window_id)
            .expect("expected first window shutdown");

        assert_eq!(
            shutdown.host_shutdown.retirement,
            StudioWindowHostRetirement::Transferred {
                new_owner_window_id: second.window_id,
                restored_entitlement_timer: shutdown
                    .host_shutdown
                    .cleared_entitlement_timer
                    .clone(),
            }
        );
        assert!(matches!(
            shutdown.timer_driver_transitions.as_slice(),
            [StudioWindowTimerDriverTransition::TransferNativeTimer { from_window_id, to_window_id, .. }]
            if *from_window_id == first.window_id && *to_window_id == second.window_id
        ));
        assert!(shutdown.timer_driver_acks.is_empty());
        assert_eq!(
            session
                .timer_driver_state()
                .window_binding(second.window_id)
                .map(|binding| binding.handle_id),
            Some(1)
        );
    }

    #[test]
    fn window_session_close_parks_binding_and_open_restores_without_duplicate_ack() {
        let mut session =
            StudioWindowSession::new(&lease_expiring_config()).expect("expected window session");
        let first = session.open_window();

        let _ = session
            .dispatch_trigger(
                first.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer elapsed dispatch");
        let shutdown = session
            .close_window(first.window_id)
            .expect("expected first window shutdown");
        assert!(matches!(
            shutdown.timer_driver_transitions.as_slice(),
            [StudioWindowTimerDriverTransition::ParkNativeTimer { from_window_id, .. }]
            if *from_window_id == first.window_id
        ));

        let reopened = session.open_window();
        assert_eq!(reopened.role, StudioWindowHostRole::EntitlementTimerOwner);
        assert_eq!(
            session
                .timer_driver_state()
                .window_binding(reopened.window_id)
                .map(|binding| binding.handle_id),
            Some(1)
        );
        assert!(session.timer_driver_state().parked_binding().is_none());
    }
}
