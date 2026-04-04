use std::collections::{BTreeMap, BTreeSet};

use rf_types::{RfError, RfResult};

use crate::{
    StudioAppWindowHostClose, StudioAppWindowHostCommand, StudioAppWindowHostCommandOutcome,
    StudioAppWindowHostDispatch, StudioAppWindowHostGlobalEvent, StudioAppWindowHostManager,
    StudioRuntimeConfig, StudioRuntimeHostAckResult, StudioRuntimeReport,
    StudioRuntimeTimerHandleSlot, StudioRuntimeTrigger, StudioWindowHostEvent, StudioWindowHostId,
    StudioWindowHostRegistration, StudioWindowHostRetirement, StudioWindowHostRole,
    StudioWindowSessionDispatch, StudioWindowTimerDriverAckResult,
    StudioWindowTimerDriverTransition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostCommand {
    OpenWindow,
    DispatchWindowTrigger {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    FocusWindow {
        window_id: StudioWindowHostId,
    },
    DispatchGlobalEvent {
        event: StudioAppWindowHostGlobalEvent,
    },
    CloseWindow {
        window_id: StudioWindowHostId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostCommandOutcome {
    WindowOpened(StudioWindowHostRegistration),
    WindowDispatched(StudioAppWindowHostDispatch),
    WindowClosed(StudioAppWindowHostClose),
    IgnoredGlobalEvent {
        event: StudioAppWindowHostGlobalEvent,
    },
    IgnoredClose {
        window_id: StudioWindowHostId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostWindowSnapshot {
    pub window_id: StudioWindowHostId,
    pub role: StudioWindowHostRole,
    pub is_foreground: bool,
    pub entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

pub type StudioAppHostWindowState = StudioAppHostWindowSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostWindowChange {
    Added {
        current: StudioAppHostWindowSnapshot,
    },
    Removed {
        previous: StudioAppHostWindowSnapshot,
    },
    Updated {
        previous: StudioAppHostWindowSnapshot,
        current: StudioAppHostWindowSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostWindowSelectionChange {
    pub previous: Option<StudioWindowHostId>,
    pub current: Option<StudioWindowHostId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostTimerSlotChange {
    pub previous: Option<StudioRuntimeTimerHandleSlot>,
    pub current: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostSnapshot {
    pub registered_windows: Vec<StudioWindowHostId>,
    pub windows: Vec<StudioAppHostWindowSnapshot>,
    pub foreground_window_id: Option<StudioWindowHostId>,
    pub entitlement_timer_owner_window_id: Option<StudioWindowHostId>,
    pub parked_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum StudioAppHostEntitlementTimerState {
    #[default]
    Idle,
    Owned {
        owner_window_id: StudioWindowHostId,
        slot: Option<StudioRuntimeTimerHandleSlot>,
    },
    Parked {
        slot: StudioRuntimeTimerHandleSlot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostState {
    pub registered_windows: Vec<StudioWindowHostId>,
    pub windows: Vec<StudioAppHostWindowState>,
    pub foreground_window_id: Option<StudioWindowHostId>,
    pub entitlement_timer: StudioAppHostEntitlementTimerState,
}

impl StudioAppHostState {
    pub fn from_snapshot(snapshot: &StudioAppHostSnapshot) -> Self {
        Self {
            registered_windows: snapshot.registered_windows.clone(),
            windows: snapshot.windows.clone(),
            foreground_window_id: snapshot.foreground_window_id,
            entitlement_timer: entitlement_timer_state_from_snapshot(snapshot),
        }
    }

    pub fn window(&self, window_id: StudioWindowHostId) -> Option<&StudioAppHostWindowState> {
        self.windows
            .iter()
            .find(|window| window.window_id == window_id)
    }

    pub fn entitlement_timer_owner_window_id(&self) -> Option<StudioWindowHostId> {
        match self.entitlement_timer {
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id, ..
            } => Some(owner_window_id),
            StudioAppHostEntitlementTimerState::Idle
            | StudioAppHostEntitlementTimerState::Parked { .. } => None,
        }
    }

    pub fn parked_entitlement_timer(&self) -> Option<&StudioRuntimeTimerHandleSlot> {
        match &self.entitlement_timer {
            StudioAppHostEntitlementTimerState::Parked { slot } => Some(slot),
            StudioAppHostEntitlementTimerState::Idle
            | StudioAppHostEntitlementTimerState::Owned { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostEntitlementTimerStateChange {
    pub previous: StudioAppHostEntitlementTimerState,
    pub current: StudioAppHostEntitlementTimerState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostProjection {
    pub state: StudioAppHostState,
    pub added_windows: Vec<StudioAppHostWindowState>,
    pub removed_window_ids: Vec<StudioWindowHostId>,
    pub updated_windows: Vec<StudioAppHostWindowState>,
    pub foreground_window_change: Option<StudioAppHostWindowSelectionChange>,
    pub entitlement_timer_change: Option<StudioAppHostEntitlementTimerStateChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostChangeSet {
    pub window_changes: Vec<StudioAppHostWindowChange>,
    pub foreground_window_change: Option<StudioAppHostWindowSelectionChange>,
    pub entitlement_timer_owner_change: Option<StudioAppHostWindowSelectionChange>,
    pub parked_entitlement_timer_change: Option<StudioAppHostTimerSlotChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostOutput {
    pub outcome: StudioAppHostCommandOutcome,
    pub snapshot: StudioAppHostSnapshot,
    pub changes: StudioAppHostChangeSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostStore {
    state: StudioAppHostState,
}

impl StudioAppHostStore {
    pub fn new(initial_state: StudioAppHostState) -> Self {
        Self {
            state: initial_state,
        }
    }

    pub fn from_snapshot(snapshot: &StudioAppHostSnapshot) -> Self {
        Self::new(StudioAppHostState::from_snapshot(snapshot))
    }

    pub fn state(&self) -> &StudioAppHostState {
        &self.state
    }

    pub fn project_output(&self, output: &StudioAppHostOutput) -> StudioAppHostProjection {
        let next_state = StudioAppHostState::from_snapshot(&output.snapshot);
        let mut added_windows = Vec::new();
        let mut removed_window_ids = Vec::new();
        let mut updated_windows = Vec::new();

        for change in &output.changes.window_changes {
            match change {
                StudioAppHostWindowChange::Added { current } => {
                    added_windows.push(current.clone());
                }
                StudioAppHostWindowChange::Removed { previous } => {
                    removed_window_ids.push(previous.window_id);
                }
                StudioAppHostWindowChange::Updated { current, .. } => {
                    updated_windows.push(current.clone());
                }
            }
        }

        let entitlement_timer_change = diff_entitlement_timer_state(
            &self.state.entitlement_timer,
            &next_state.entitlement_timer,
        );

        StudioAppHostProjection {
            state: next_state,
            added_windows,
            removed_window_ids,
            updated_windows,
            foreground_window_change: output.changes.foreground_window_change.clone(),
            entitlement_timer_change,
        }
    }

    pub fn apply_output(&mut self, output: &StudioAppHostOutput) -> StudioAppHostProjection {
        let projection = self.project_output(output);
        self.state = projection.state.clone();
        projection
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostOpenWindowResult {
    pub projection: StudioAppHostProjection,
    pub registration: StudioWindowHostRegistration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostWindowDispatchResult {
    pub projection: StudioAppHostProjection,
    pub target_window_id: StudioWindowHostId,
    pub effects: StudioAppHostDispatchEffects,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostGlobalEventResult {
    pub projection: StudioAppHostProjection,
    pub dispatch: Option<StudioAppHostWindowDispatchResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostCloseWindowResult {
    pub projection: StudioAppHostProjection,
    pub close: Option<StudioAppHostCloseEffects>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostEntitlementTimerEffect {
    Keep {
        owner_window_id: StudioWindowHostId,
        effect_id: u64,
        slot: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
        ack: StudioRuntimeHostAckResult,
    },
    Arm {
        owner_window_id: StudioWindowHostId,
        effect_id: u64,
        slot: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
        ack: StudioRuntimeHostAckResult,
    },
    Rearm {
        owner_window_id: StudioWindowHostId,
        effect_id: u64,
        previous_slot: Option<StudioRuntimeTimerHandleSlot>,
        next_slot: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
        ack: StudioRuntimeHostAckResult,
    },
    Clear {
        owner_window_id: StudioWindowHostId,
        effect_id: u64,
        previous_slot: Option<StudioRuntimeTimerHandleSlot>,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
        ack: StudioRuntimeHostAckResult,
    },
    IgnoreStale {
        owner_window_id: StudioWindowHostId,
        stale_effect_id: u64,
        current_slot: Option<StudioRuntimeTimerHandleSlot>,
        ack: StudioRuntimeHostAckResult,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostDispatchEffects {
    pub runtime_report: StudioRuntimeReport,
    pub entitlement_timer_effect: Option<StudioAppHostEntitlementTimerEffect>,
    pub native_timer_transitions: Vec<StudioWindowTimerDriverTransition>,
    pub native_timer_acks: Vec<StudioWindowTimerDriverAckResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostCloseEffects {
    pub window_id: StudioWindowHostId,
    pub cleared_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
    pub retirement: StudioWindowHostRetirement,
    pub next_foreground_window_id: Option<StudioWindowHostId>,
    pub native_timer_transitions: Vec<StudioWindowTimerDriverTransition>,
    pub native_timer_acks: Vec<StudioWindowTimerDriverAckResult>,
}

pub struct StudioAppHost {
    window_host_manager: StudioAppWindowHostManager,
}

impl StudioAppHost {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            window_host_manager: StudioAppWindowHostManager::new(config)?,
        })
    }

    pub fn window_host_manager(&self) -> &StudioAppWindowHostManager {
        &self.window_host_manager
    }

    pub fn snapshot(&self) -> StudioAppHostSnapshot {
        let registered_windows = self.window_host_manager.registered_windows();
        let foreground_window_id = self.window_host_manager.foreground_window_id();
        let entitlement_timer_owner_window_id = self
            .window_host_manager
            .session()
            .host_port()
            .entitlement_timer_owner();
        let windows = registered_windows
            .iter()
            .copied()
            .map(|window_id| {
                let role = if entitlement_timer_owner_window_id == Some(window_id) {
                    StudioWindowHostRole::EntitlementTimerOwner
                } else {
                    StudioWindowHostRole::Observer
                };
                let entitlement_timer = self
                    .window_host_manager
                    .session()
                    .host_port()
                    .window_state(window_id)
                    .and_then(|state| state.entitlement_timer().cloned());

                StudioAppHostWindowSnapshot {
                    window_id,
                    role,
                    is_foreground: foreground_window_id == Some(window_id),
                    entitlement_timer,
                }
            })
            .collect();

        StudioAppHostSnapshot {
            registered_windows,
            windows,
            foreground_window_id,
            entitlement_timer_owner_window_id,
            parked_entitlement_timer: self
                .window_host_manager
                .session()
                .host_port()
                .parked_entitlement_timer()
                .cloned(),
        }
    }

    pub fn execute_command(
        &mut self,
        command: StudioAppHostCommand,
    ) -> RfResult<StudioAppHostOutput> {
        let previous_snapshot = self.snapshot();
        let outcome = self
            .window_host_manager
            .execute_command(map_command(command))
            .map(map_outcome)?;
        let snapshot = self.snapshot();

        Ok(StudioAppHostOutput {
            outcome,
            changes: diff_app_host_snapshots(&previous_snapshot, &snapshot),
            snapshot,
        })
    }
}

pub struct StudioAppHostController {
    app_host: StudioAppHost,
    store: StudioAppHostStore,
}

impl StudioAppHostController {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        let app_host = StudioAppHost::new(config)?;
        let store = StudioAppHostStore::from_snapshot(&app_host.snapshot());

        Ok(Self { app_host, store })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.store.state()
    }

    pub fn open_window(&mut self) -> RfResult<StudioAppHostOpenWindowResult> {
        let (outcome, projection) = self.execute_command(StudioAppHostCommand::OpenWindow)?;
        let StudioAppHostCommandOutcome::WindowOpened(registration) = outcome else {
            return Err(RfError::invalid_input(
                "app host controller expected window open outcome",
            ));
        };

        Ok(StudioAppHostOpenWindowResult {
            projection,
            registration,
        })
    }

    pub fn dispatch_window_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    ) -> RfResult<StudioAppHostWindowDispatchResult> {
        let (outcome, projection) = self
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger { window_id, trigger })?;
        let StudioAppHostCommandOutcome::WindowDispatched(dispatch) = outcome else {
            return Err(RfError::invalid_input(
                "app host controller expected window dispatch outcome",
            ));
        };

        Ok(StudioAppHostWindowDispatchResult {
            projection,
            target_window_id: dispatch.target_window_id,
            effects: dispatch_effects_from_session(dispatch.dispatch),
        })
    }

    pub fn focus_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppHostWindowDispatchResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::FocusWindow { window_id })?;
        let StudioAppHostCommandOutcome::WindowDispatched(dispatch) = outcome else {
            return Err(RfError::invalid_input(
                "app host controller expected focus dispatch outcome",
            ));
        };

        Ok(StudioAppHostWindowDispatchResult {
            projection,
            target_window_id: dispatch.target_window_id,
            effects: dispatch_effects_from_session(dispatch.dispatch),
        })
    }

    pub fn dispatch_global_event(
        &mut self,
        event: StudioAppWindowHostGlobalEvent,
    ) -> RfResult<StudioAppHostGlobalEventResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchGlobalEvent { event })?;
        let dispatch = match outcome {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch) => {
                Some(StudioAppHostWindowDispatchResult {
                    projection: projection.clone(),
                    target_window_id: dispatch.target_window_id,
                    effects: dispatch_effects_from_session(dispatch.dispatch),
                })
            }
            StudioAppHostCommandOutcome::IgnoredGlobalEvent { .. } => None,
            other => {
                return Err(RfError::invalid_input(format!(
                    "app host controller expected global event outcome, got {other:?}"
                )));
            }
        };

        Ok(StudioAppHostGlobalEventResult {
            projection,
            dispatch,
        })
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppHostCloseWindowResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::CloseWindow { window_id })?;
        let close = match outcome {
            StudioAppHostCommandOutcome::WindowClosed(close) => {
                Some(close_effects_from_shutdown(close))
            }
            StudioAppHostCommandOutcome::IgnoredClose { .. } => None,
            other => {
                return Err(RfError::invalid_input(format!(
                    "app host controller expected close outcome, got {other:?}"
                )));
            }
        };

        Ok(StudioAppHostCloseWindowResult { projection, close })
    }

    fn execute_command(
        &mut self,
        command: StudioAppHostCommand,
    ) -> RfResult<(StudioAppHostCommandOutcome, StudioAppHostProjection)> {
        self.app_host.execute_command(command).map(|output| {
            let projection = self.store.apply_output(&output);
            (output.outcome, projection)
        })
    }
}

fn dispatch_effects_from_session(
    dispatch: StudioWindowSessionDispatch,
) -> StudioAppHostDispatchEffects {
    StudioAppHostDispatchEffects {
        runtime_report: dispatch.host_output.runtime_output.report,
        entitlement_timer_effect: dispatch
            .host_output
            .window_event
            .map(entitlement_timer_effect_from_window_event),
        native_timer_transitions: dispatch.timer_driver_transitions,
        native_timer_acks: dispatch.timer_driver_acks,
    }
}

fn close_effects_from_shutdown(close: StudioAppWindowHostClose) -> StudioAppHostCloseEffects {
    StudioAppHostCloseEffects {
        window_id: close.window_id,
        cleared_entitlement_timer: close.shutdown.host_shutdown.cleared_entitlement_timer,
        retirement: close.shutdown.host_shutdown.retirement,
        next_foreground_window_id: close.next_foreground_window_id,
        native_timer_transitions: close.shutdown.timer_driver_transitions,
        native_timer_acks: close.shutdown.timer_driver_acks,
    }
}

fn entitlement_timer_effect_from_window_event(
    event: StudioWindowHostEvent,
) -> StudioAppHostEntitlementTimerEffect {
    match event {
        StudioWindowHostEvent::EntitlementTimerApplied {
            window_id,
            command,
            transition,
            ack,
        } => match transition {
            crate::StudioRuntimeTimerHostTransition::KeepTimer {
                slot,
                follow_up_trigger,
            } => StudioAppHostEntitlementTimerEffect::Keep {
                owner_window_id: window_id,
                effect_id: command.effect_id(),
                slot,
                follow_up_trigger,
                ack,
            },
            crate::StudioRuntimeTimerHostTransition::ArmTimer {
                slot,
                follow_up_trigger,
            } => StudioAppHostEntitlementTimerEffect::Arm {
                owner_window_id: window_id,
                effect_id: command.effect_id(),
                slot,
                follow_up_trigger,
                ack,
            },
            crate::StudioRuntimeTimerHostTransition::RearmTimer {
                previous,
                next,
                follow_up_trigger,
            } => StudioAppHostEntitlementTimerEffect::Rearm {
                owner_window_id: window_id,
                effect_id: command.effect_id(),
                previous_slot: previous,
                next_slot: next,
                follow_up_trigger,
                ack,
            },
            crate::StudioRuntimeTimerHostTransition::ClearTimer {
                previous,
                follow_up_trigger,
            } => StudioAppHostEntitlementTimerEffect::Clear {
                owner_window_id: window_id,
                effect_id: command.effect_id(),
                previous_slot: previous,
                follow_up_trigger,
                ack,
            },
            crate::StudioRuntimeTimerHostTransition::IgnoreStale {
                current,
                stale_effect_id,
            } => StudioAppHostEntitlementTimerEffect::IgnoreStale {
                owner_window_id: window_id,
                stale_effect_id,
                current_slot: current,
                ack,
            },
        },
    }
}

fn diff_app_host_snapshots(
    previous: &StudioAppHostSnapshot,
    current: &StudioAppHostSnapshot,
) -> StudioAppHostChangeSet {
    let previous_windows = snapshot_windows_by_id(&previous.windows);
    let current_windows = snapshot_windows_by_id(&current.windows);
    let mut window_ids = BTreeSet::new();
    window_ids.extend(previous_windows.keys().copied());
    window_ids.extend(current_windows.keys().copied());

    let mut window_changes = Vec::new();
    for window_id in window_ids {
        match (
            previous_windows.get(&window_id),
            current_windows.get(&window_id),
        ) {
            (None, Some(current)) => window_changes.push(StudioAppHostWindowChange::Added {
                current: current.clone(),
            }),
            (Some(previous), None) => window_changes.push(StudioAppHostWindowChange::Removed {
                previous: previous.clone(),
            }),
            (Some(previous), Some(current)) if previous != current => {
                window_changes.push(StudioAppHostWindowChange::Updated {
                    previous: previous.clone(),
                    current: current.clone(),
                })
            }
            (Some(_), Some(_)) | (None, None) => {}
        }
    }

    StudioAppHostChangeSet {
        window_changes,
        foreground_window_change: diff_window_selection(
            previous.foreground_window_id,
            current.foreground_window_id,
        ),
        entitlement_timer_owner_change: diff_window_selection(
            previous.entitlement_timer_owner_window_id,
            current.entitlement_timer_owner_window_id,
        ),
        parked_entitlement_timer_change: diff_timer_slot(
            previous.parked_entitlement_timer.as_ref(),
            current.parked_entitlement_timer.as_ref(),
        ),
    }
}

fn snapshot_windows_by_id(
    windows: &[StudioAppHostWindowSnapshot],
) -> BTreeMap<StudioWindowHostId, StudioAppHostWindowSnapshot> {
    windows
        .iter()
        .cloned()
        .map(|window| (window.window_id, window))
        .collect()
}

fn diff_window_selection(
    previous: Option<StudioWindowHostId>,
    current: Option<StudioWindowHostId>,
) -> Option<StudioAppHostWindowSelectionChange> {
    if previous == current {
        return None;
    }

    Some(StudioAppHostWindowSelectionChange { previous, current })
}

fn diff_timer_slot(
    previous: Option<&StudioRuntimeTimerHandleSlot>,
    current: Option<&StudioRuntimeTimerHandleSlot>,
) -> Option<StudioAppHostTimerSlotChange> {
    if previous == current {
        return None;
    }

    Some(StudioAppHostTimerSlotChange {
        previous: previous.cloned(),
        current: current.cloned(),
    })
}

fn entitlement_timer_state_from_snapshot(
    snapshot: &StudioAppHostSnapshot,
) -> StudioAppHostEntitlementTimerState {
    if let Some(owner_window_id) = snapshot.entitlement_timer_owner_window_id {
        let slot = snapshot
            .windows
            .iter()
            .find(|window| window.window_id == owner_window_id)
            .and_then(|window| window.entitlement_timer.clone());
        return StudioAppHostEntitlementTimerState::Owned {
            owner_window_id,
            slot,
        };
    }

    if let Some(slot) = snapshot.parked_entitlement_timer.clone() {
        return StudioAppHostEntitlementTimerState::Parked { slot };
    }

    StudioAppHostEntitlementTimerState::Idle
}

fn diff_entitlement_timer_state(
    previous: &StudioAppHostEntitlementTimerState,
    current: &StudioAppHostEntitlementTimerState,
) -> Option<StudioAppHostEntitlementTimerStateChange> {
    if previous == current {
        return None;
    }

    Some(StudioAppHostEntitlementTimerStateChange {
        previous: previous.clone(),
        current: current.clone(),
    })
}

fn map_command(command: StudioAppHostCommand) -> StudioAppWindowHostCommand {
    match command {
        StudioAppHostCommand::OpenWindow => StudioAppWindowHostCommand::OpenWindow,
        StudioAppHostCommand::DispatchWindowTrigger { window_id, trigger } => {
            StudioAppWindowHostCommand::DispatchTrigger { window_id, trigger }
        }
        StudioAppHostCommand::FocusWindow { window_id } => {
            StudioAppWindowHostCommand::FocusWindow { window_id }
        }
        StudioAppHostCommand::DispatchGlobalEvent { event } => {
            StudioAppWindowHostCommand::DispatchGlobalEvent { event }
        }
        StudioAppHostCommand::CloseWindow { window_id } => {
            StudioAppWindowHostCommand::CloseWindow { window_id }
        }
    }
}

fn map_outcome(outcome: StudioAppWindowHostCommandOutcome) -> StudioAppHostCommandOutcome {
    match outcome {
        StudioAppWindowHostCommandOutcome::WindowOpened(registration) => {
            StudioAppHostCommandOutcome::WindowOpened(registration)
        }
        StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch)
        }
        StudioAppWindowHostCommandOutcome::WindowClosed(close) => {
            StudioAppHostCommandOutcome::WindowClosed(close)
        }
        StudioAppWindowHostCommandOutcome::IgnoredGlobalEvent { event } => {
            StudioAppHostCommandOutcome::IgnoredGlobalEvent { event }
        }
        StudioAppWindowHostCommandOutcome::IgnoredClose { window_id } => {
            StudioAppHostCommandOutcome::IgnoredClose { window_id }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioAppHost, StudioAppHostCommand, StudioAppHostCommandOutcome, StudioAppHostController,
        StudioAppHostEntitlementTimerEffect, StudioAppHostEntitlementTimerState,
        StudioAppHostStore, StudioAppHostTimerSlotChange, StudioAppHostWindowChange,
        StudioAppHostWindowSelectionChange, StudioAppWindowHostGlobalEvent,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger, StudioWindowHostRetirement,
        StudioWindowHostRole,
    };

    fn lease_expiring_config() -> crate::StudioRuntimeConfig {
        crate::StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..crate::StudioRuntimeConfig::default()
        }
    }

    #[test]
    fn app_host_returns_snapshot_with_window_open_and_focus_updates() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");

        let first = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open");
        let first_window = match &first.outcome {
            StudioAppHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        assert_eq!(
            first_window.role,
            StudioWindowHostRole::EntitlementTimerOwner
        );
        assert_eq!(
            first.snapshot.registered_windows,
            vec![first_window.window_id]
        );
        assert_eq!(
            first.snapshot.foreground_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(
            first.snapshot.entitlement_timer_owner_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(
            first.snapshot.windows,
            vec![crate::StudioAppHostWindowSnapshot {
                window_id: first_window.window_id,
                role: StudioWindowHostRole::EntitlementTimerOwner,
                is_foreground: true,
                entitlement_timer: None,
            }]
        );
        assert_eq!(
            first.changes.window_changes,
            vec![StudioAppHostWindowChange::Added {
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: first_window.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    is_foreground: true,
                    entitlement_timer: None,
                },
            }]
        );
        assert_eq!(
            first.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: None,
                current: Some(first_window.window_id),
            })
        );
        assert_eq!(
            first.changes.entitlement_timer_owner_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: None,
                current: Some(first_window.window_id),
            })
        );
        assert_eq!(first.changes.parked_entitlement_timer_change, None);

        let second = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected second window open");
        let second_window = match &second.outcome {
            StudioAppHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        assert_eq!(second_window.role, StudioWindowHostRole::Observer);
        assert_eq!(
            second.snapshot.registered_windows,
            vec![first_window.window_id, second_window.window_id]
        );
        assert_eq!(
            second.snapshot.foreground_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(second.snapshot.windows.len(), 2);
        assert_eq!(
            second.snapshot.windows[0].role,
            StudioWindowHostRole::EntitlementTimerOwner
        );
        assert!(second.snapshot.windows[0].is_foreground);
        assert_eq!(
            second.snapshot.windows[1].role,
            StudioWindowHostRole::Observer
        );
        assert!(!second.snapshot.windows[1].is_foreground);
        assert_eq!(
            second.changes.window_changes,
            vec![StudioAppHostWindowChange::Added {
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: second_window.window_id,
                    role: StudioWindowHostRole::Observer,
                    is_foreground: false,
                    entitlement_timer: None,
                },
            }]
        );
        assert_eq!(second.changes.foreground_window_change, None);
        assert_eq!(second.changes.entitlement_timer_owner_change, None);

        let focused = app_host
            .execute_command(StudioAppHostCommand::FocusWindow {
                window_id: second_window.window_id,
            })
            .expect("expected focus command");
        assert_eq!(
            focused.snapshot.foreground_window_id,
            Some(second_window.window_id)
        );
        assert_eq!(
            focused.snapshot.entitlement_timer_owner_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(
            focused.snapshot.windows[1],
            crate::StudioAppHostWindowSnapshot {
                window_id: second_window.window_id,
                role: StudioWindowHostRole::Observer,
                is_foreground: true,
                entitlement_timer: None,
            }
        );
        let focused_owner_timer = focused.snapshot.windows[0].entitlement_timer.clone();
        assert_eq!(
            focused.changes.window_changes,
            vec![
                StudioAppHostWindowChange::Updated {
                    previous: crate::StudioAppHostWindowSnapshot {
                        window_id: first_window.window_id,
                        role: StudioWindowHostRole::EntitlementTimerOwner,
                        is_foreground: true,
                        entitlement_timer: None,
                    },
                    current: crate::StudioAppHostWindowSnapshot {
                        window_id: first_window.window_id,
                        role: StudioWindowHostRole::EntitlementTimerOwner,
                        is_foreground: false,
                        entitlement_timer: focused_owner_timer,
                    },
                },
                StudioAppHostWindowChange::Updated {
                    previous: crate::StudioAppHostWindowSnapshot {
                        window_id: second_window.window_id,
                        role: StudioWindowHostRole::Observer,
                        is_foreground: false,
                        entitlement_timer: None,
                    },
                    current: crate::StudioAppHostWindowSnapshot {
                        window_id: second_window.window_id,
                        role: StudioWindowHostRole::Observer,
                        is_foreground: true,
                        entitlement_timer: None,
                    },
                },
            ]
        );
        assert_eq!(
            focused.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first_window.window_id),
                current: Some(second_window.window_id),
            })
        );
        assert_eq!(focused.changes.entitlement_timer_owner_change, None);
    }

    #[test]
    fn app_host_snapshot_tracks_parked_timer_across_last_close_and_reopen() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let first = match app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open")
            .outcome
        {
            StudioAppHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let triggered = app_host
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
                window_id: first.window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger");
        assert_eq!(
            triggered.snapshot.entitlement_timer_owner_window_id,
            Some(first.window_id)
        );
        assert_eq!(triggered.snapshot.windows.len(), 1);
        assert_eq!(
            triggered.snapshot.windows[0]
                .entitlement_timer
                .as_ref()
                .map(|slot| slot.effect_id),
            Some(1)
        );
        let triggered_timer = triggered.snapshot.windows[0]
            .entitlement_timer
            .clone()
            .expect("expected timer slot");
        assert_eq!(
            triggered.changes.window_changes,
            vec![StudioAppHostWindowChange::Updated {
                previous: crate::StudioAppHostWindowSnapshot {
                    window_id: first.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    is_foreground: true,
                    entitlement_timer: None,
                },
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: first.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    is_foreground: true,
                    entitlement_timer: Some(triggered_timer.clone()),
                },
            }]
        );
        assert_eq!(triggered.changes.foreground_window_change, None);
        assert_eq!(triggered.changes.entitlement_timer_owner_change, None);

        let closed = app_host
            .execute_command(StudioAppHostCommand::CloseWindow {
                window_id: first.window_id,
            })
            .expect("expected close command");
        match &closed.outcome {
            StudioAppHostCommandOutcome::WindowClosed(close) => {
                assert_eq!(
                    close.shutdown.host_shutdown.retirement,
                    StudioWindowHostRetirement::Parked {
                        parked_entitlement_timer: close
                            .shutdown
                            .host_shutdown
                            .cleared_entitlement_timer
                            .clone(),
                    }
                );
            }
            other => panic!("expected window closed outcome, got {other:?}"),
        }
        assert!(closed.snapshot.registered_windows.is_empty());
        assert!(closed.snapshot.windows.is_empty());
        assert!(closed.snapshot.foreground_window_id.is_none());
        assert!(closed.snapshot.entitlement_timer_owner_window_id.is_none());
        assert_eq!(
            closed
                .snapshot
                .parked_entitlement_timer
                .as_ref()
                .map(|slot| slot.effect_id),
            Some(1)
        );
        let parked_timer = closed
            .snapshot
            .parked_entitlement_timer
            .clone()
            .expect("expected parked timer");
        assert_eq!(
            closed.changes.window_changes,
            vec![StudioAppHostWindowChange::Removed {
                previous: crate::StudioAppHostWindowSnapshot {
                    window_id: first.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    is_foreground: true,
                    entitlement_timer: Some(triggered_timer.clone()),
                },
            }]
        );
        assert_eq!(
            closed.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first.window_id),
                current: None,
            })
        );
        assert_eq!(
            closed.changes.entitlement_timer_owner_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first.window_id),
                current: None,
            })
        );
        assert_eq!(
            closed.changes.parked_entitlement_timer_change,
            Some(StudioAppHostTimerSlotChange {
                previous: None,
                current: Some(parked_timer.clone()),
            })
        );

        let reopened = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected reopen");
        assert_eq!(reopened.snapshot.parked_entitlement_timer, None);
        assert_eq!(reopened.snapshot.registered_windows.len(), 1);
        assert_eq!(
            reopened.snapshot.windows[0]
                .entitlement_timer
                .as_ref()
                .map(|slot| slot.effect_id),
            Some(1)
        );
        let restored_timer = reopened.snapshot.windows[0]
            .entitlement_timer
            .clone()
            .expect("expected restored timer");
        assert!(matches!(
            reopened.outcome,
            StudioAppHostCommandOutcome::WindowOpened(_)
        ));
        assert_eq!(
            reopened.changes.window_changes,
            vec![StudioAppHostWindowChange::Added {
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: reopened.snapshot.windows[0].window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    is_foreground: true,
                    entitlement_timer: Some(restored_timer.clone()),
                },
            }]
        );
        assert_eq!(
            reopened.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: None,
                current: Some(reopened.snapshot.windows[0].window_id),
            })
        );
        assert_eq!(
            reopened.changes.entitlement_timer_owner_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: None,
                current: Some(reopened.snapshot.windows[0].window_id),
            })
        );
        assert_eq!(
            reopened.changes.parked_entitlement_timer_change,
            Some(StudioAppHostTimerSlotChange {
                previous: Some(parked_timer),
                current: None,
            })
        );
    }

    #[test]
    fn app_host_change_set_captures_owner_transfer_on_close() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let first = match app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open")
            .outcome
        {
            StudioAppHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let second = match app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected second window open")
            .outcome
        {
            StudioAppHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let _ = app_host
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
                window_id: first.window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger");

        let closed = app_host
            .execute_command(StudioAppHostCommand::CloseWindow {
                window_id: first.window_id,
            })
            .expect("expected owner close");
        let transferred_timer = closed.snapshot.windows[0]
            .entitlement_timer
            .clone()
            .expect("expected transferred timer");

        assert_eq!(
            closed.changes.window_changes,
            vec![
                StudioAppHostWindowChange::Removed {
                    previous: crate::StudioAppHostWindowSnapshot {
                        window_id: first.window_id,
                        role: StudioWindowHostRole::EntitlementTimerOwner,
                        is_foreground: true,
                        entitlement_timer: Some(transferred_timer.clone()),
                    },
                },
                StudioAppHostWindowChange::Updated {
                    previous: crate::StudioAppHostWindowSnapshot {
                        window_id: second.window_id,
                        role: StudioWindowHostRole::Observer,
                        is_foreground: false,
                        entitlement_timer: None,
                    },
                    current: crate::StudioAppHostWindowSnapshot {
                        window_id: second.window_id,
                        role: StudioWindowHostRole::EntitlementTimerOwner,
                        is_foreground: true,
                        entitlement_timer: Some(transferred_timer),
                    },
                },
            ]
        );
        assert_eq!(
            closed.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first.window_id),
                current: Some(second.window_id),
            })
        );
        assert_eq!(
            closed.changes.entitlement_timer_owner_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first.window_id),
                current: Some(second.window_id),
            })
        );
        assert_eq!(closed.changes.parked_entitlement_timer_change, None);
    }

    #[test]
    fn app_host_surfaces_ignored_global_events_with_stable_snapshot() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");

        let output = app_host
            .execute_command(StudioAppHostCommand::DispatchGlobalEvent {
                event: StudioAppWindowHostGlobalEvent::NetworkRestored,
            })
            .expect("expected ignored global event");

        assert_eq!(
            output.outcome,
            StudioAppHostCommandOutcome::IgnoredGlobalEvent {
                event: StudioAppWindowHostGlobalEvent::NetworkRestored,
            }
        );
        assert!(output.snapshot.registered_windows.is_empty());
        assert!(output.snapshot.windows.is_empty());
        assert!(output.snapshot.foreground_window_id.is_none());
        assert!(output.changes.window_changes.is_empty());
        assert_eq!(output.changes.foreground_window_change, None);
        assert_eq!(output.changes.entitlement_timer_owner_change, None);
        assert_eq!(output.changes.parked_entitlement_timer_change, None);
    }

    #[test]
    fn app_host_store_projects_output_into_single_state_boundary() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let mut store = StudioAppHostStore::from_snapshot(&app_host.snapshot());

        let first_open = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open");
        let first_projection = store.apply_output(&first_open);
        let first_window = first_projection
            .added_windows
            .first()
            .expect("expected first added window");

        assert_eq!(
            first_projection.state.registered_windows,
            vec![first_window.window_id]
        );
        assert_eq!(
            first_projection.state.foreground_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(
            first_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: first_window.window_id,
                slot: None,
            }
        );
        assert_eq!(first_projection.removed_window_ids, Vec::<u64>::new());
        assert!(first_projection.updated_windows.is_empty());

        let second_open = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected second window open");
        let second_projection = store.apply_output(&second_open);
        let second_window = second_projection
            .added_windows
            .first()
            .expect("expected second added window");

        assert_eq!(
            second_projection.state.registered_windows,
            vec![first_window.window_id, second_window.window_id]
        );
        assert_eq!(
            second_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: first_window.window_id,
                slot: None,
            }
        );

        let focused = app_host
            .execute_command(StudioAppHostCommand::FocusWindow {
                window_id: second_window.window_id,
            })
            .expect("expected focus command");
        let focused_projection = store.apply_output(&focused);

        assert_eq!(
            focused_projection.state.foreground_window_id,
            Some(second_window.window_id)
        );
        assert_eq!(focused_projection.added_windows, Vec::new());
        assert_eq!(focused_projection.removed_window_ids, Vec::<u64>::new());
        assert_eq!(focused_projection.updated_windows.len(), 2);
        assert_eq!(
            focused_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: first_window.window_id,
                slot: focused_projection
                    .state
                    .window(first_window.window_id)
                    .and_then(|window| window.entitlement_timer.clone()),
            }
        );
        assert!(focused_projection.entitlement_timer_change.is_some());
    }

    #[test]
    fn app_host_store_collapses_owner_and_parked_timer_semantics() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let mut store = StudioAppHostStore::from_snapshot(&app_host.snapshot());

        let opened = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected window open");
        let opened_projection = store.apply_output(&opened);
        let window_id = opened_projection
            .added_windows
            .first()
            .expect("expected opened window")
            .window_id;

        let triggered = app_host
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
                window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger");
        let triggered_projection = store.apply_output(&triggered);
        let owned_slot = triggered_projection
            .state
            .window(window_id)
            .and_then(|window| window.entitlement_timer.clone())
            .expect("expected owned timer slot");

        assert_eq!(
            triggered_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: window_id,
                slot: Some(owned_slot.clone()),
            }
        );

        let closed = app_host
            .execute_command(StudioAppHostCommand::CloseWindow { window_id })
            .expect("expected close command");
        let closed_projection = store.apply_output(&closed);

        assert!(closed_projection.state.windows.is_empty());
        assert_eq!(
            closed_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Parked {
                slot: owned_slot.clone(),
            }
        );
        assert_eq!(closed_projection.removed_window_ids, vec![window_id]);

        let reopened = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected reopen");
        let reopened_projection = store.apply_output(&reopened);
        let reopened_window_id = reopened_projection
            .added_windows
            .first()
            .expect("expected reopened window")
            .window_id;

        assert_eq!(
            reopened_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: reopened_window_id,
                slot: reopened_projection
                    .state
                    .window(reopened_window_id)
                    .and_then(|window| window.entitlement_timer.clone()),
            }
        );
        assert!(
            reopened_projection
                .state
                .parked_entitlement_timer()
                .is_none()
        );
    }

    #[test]
    fn app_host_controller_advances_state_through_typed_command_methods() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let opened = controller.open_window().expect("expected window open");
        assert_eq!(
            opened.projection.state.registered_windows,
            vec![opened.registration.window_id]
        );
        assert_eq!(
            controller.state().foreground_window_id,
            Some(opened.registration.window_id)
        );

        let dispatched = controller
            .dispatch_window_trigger(
                opened.registration.window_id,
                StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected trigger dispatch");
        let owner_slot = dispatched
            .projection
            .state
            .window(opened.registration.window_id)
            .and_then(|window| window.entitlement_timer.clone())
            .expect("expected timer slot");

        assert_eq!(dispatched.target_window_id, opened.registration.window_id);
        assert!(matches!(
            dispatched.effects.entitlement_timer_effect,
            Some(StudioAppHostEntitlementTimerEffect::Rearm {
                owner_window_id,
                effect_id: 1,
                ..
            }) if owner_window_id == opened.registration.window_id
        ));
        assert!(matches!(
            dispatched.effects.native_timer_transitions.as_slice(),
            [crate::StudioWindowTimerDriverTransition::RearmNativeTimer { window_id, .. }]
            if *window_id == opened.registration.window_id
        ));
        assert_eq!(dispatched.effects.native_timer_acks.len(), 1);
        assert_eq!(
            controller.state().entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: opened.registration.window_id,
                slot: Some(owner_slot),
            }
        );
    }

    #[test]
    fn app_host_controller_returns_optional_results_for_ignored_cases() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let ignored_global = controller
            .dispatch_global_event(StudioAppWindowHostGlobalEvent::NetworkRestored)
            .expect("expected ignored global event");
        assert!(ignored_global.dispatch.is_none());
        assert!(ignored_global.projection.state.windows.is_empty());

        let opened = controller.open_window().expect("expected window open");
        let closed = controller
            .close_window(opened.registration.window_id)
            .expect("expected close");
        assert!(closed.close.is_some());

        let ignored_close = controller
            .close_window(opened.registration.window_id)
            .expect("expected ignored close");
        assert!(ignored_close.close.is_none());
        assert!(ignored_close.projection.state.windows.is_empty());
    }

    #[test]
    fn app_host_controller_maps_close_side_effects_into_gui_facing_summary() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");
        let _ = controller
            .dispatch_window_trigger(
                opened.registration.window_id,
                StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer trigger");

        let closed = controller
            .close_window(opened.registration.window_id)
            .expect("expected close");
        let close = closed.close.expect("expected close effects");

        assert_eq!(close.window_id, opened.registration.window_id);
        assert!(matches!(
            close.retirement,
            StudioWindowHostRetirement::Parked {
                parked_entitlement_timer: Some(_)
            }
        ));
        assert!(matches!(
            close.native_timer_transitions.as_slice(),
            [crate::StudioWindowTimerDriverTransition::ParkNativeTimer { from_window_id, .. }]
            if *from_window_id == opened.registration.window_id
        ));
        assert!(close.native_timer_acks.is_empty());
        assert!(matches!(
            closed.projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Parked { .. }
        ));
    }
}
