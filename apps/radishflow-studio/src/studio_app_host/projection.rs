use super::*;

pub(super) fn dispatch_effects_from_session(
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

pub(super) fn close_effects_from_shutdown(
    close: StudioAppWindowHostClose,
) -> StudioAppHostCloseEffects {
    StudioAppHostCloseEffects {
        window_id: close.window_id,
        cleared_entitlement_timer: close.shutdown.host_shutdown.cleared_entitlement_timer,
        retirement: close.shutdown.host_shutdown.retirement,
        next_foreground_window_id: close.next_foreground_window_id,
        native_timer_transitions: close.shutdown.timer_driver_transitions,
        native_timer_acks: close.shutdown.timer_driver_acks,
    }
}

pub(super) fn registration_from_opened_window(
    opened: impl Borrow<StudioAppWindowHostOpenWindow>,
) -> StudioWindowHostRegistration {
    let opened = opened.borrow();
    StudioWindowHostRegistration {
        window_id: opened.window_id,
        role: opened.role,
        layout_slot: opened.layout_slot,
        restored_entitlement_timer: opened.restored_entitlement_timer.clone(),
        timer_driver_commands: opened.timer_driver_commands.clone(),
    }
}

pub(super) fn entitlement_timer_effect_from_window_event(
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

pub(super) fn diff_app_host_snapshots(
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

pub(super) fn snapshot_windows_by_id(
    windows: &[StudioAppHostWindowSnapshot],
) -> BTreeMap<StudioWindowHostId, StudioAppHostWindowSnapshot> {
    windows
        .iter()
        .cloned()
        .map(|window| (window.window_id, window))
        .collect()
}

pub(super) fn diff_window_selection(
    previous: Option<StudioWindowHostId>,
    current: Option<StudioWindowHostId>,
) -> Option<StudioAppHostWindowSelectionChange> {
    if previous == current {
        return None;
    }

    Some(StudioAppHostWindowSelectionChange { previous, current })
}

pub(super) fn diff_timer_slot(
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

pub(super) fn entitlement_timer_state_from_snapshot(
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

pub(super) fn diff_entitlement_timer_state(
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

pub(super) fn map_command(command: StudioAppHostCommand) -> StudioAppWindowHostCommand {
    match command {
        StudioAppHostCommand::OpenWindow => StudioAppWindowHostCommand::OpenWindow,
        StudioAppHostCommand::DispatchWindowTrigger { window_id, trigger } => {
            StudioAppWindowHostCommand::DispatchTrigger { window_id, trigger }
        }
        StudioAppHostCommand::DispatchCanvasInteraction { action } => {
            StudioAppWindowHostCommand::DispatchCanvasInteraction { action }
        }
        StudioAppHostCommand::DispatchUiAction { action } => {
            StudioAppWindowHostCommand::DispatchUiAction {
                action: action.into(),
            }
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

impl From<StudioAppHostUiAction> for StudioAppWindowHostUiAction {
    fn from(value: StudioAppHostUiAction) -> Self {
        match value {
            StudioAppHostUiAction::SaveDocument => Self::SaveDocument,
            StudioAppHostUiAction::UndoDocumentCommand => Self::UndoDocumentCommand,
            StudioAppHostUiAction::RedoDocumentCommand => Self::RedoDocumentCommand,
            StudioAppHostUiAction::RunManualWorkspace => Self::RunManualWorkspace,
            StudioAppHostUiAction::ResumeWorkspace => Self::ResumeWorkspace,
            StudioAppHostUiAction::HoldWorkspace => Self::HoldWorkspace,
            StudioAppHostUiAction::ActivateWorkspace => Self::ActivateWorkspace,
            StudioAppHostUiAction::RecoverRunPanelFailure => Self::RecoverRunPanelFailure,
            StudioAppHostUiAction::SyncEntitlement => Self::SyncEntitlement,
            StudioAppHostUiAction::RefreshOfflineLease => Self::RefreshOfflineLease,
        }
    }
}

impl From<StudioAppWindowHostUiAction> for StudioAppHostUiAction {
    fn from(value: StudioAppWindowHostUiAction) -> Self {
        match value {
            StudioAppWindowHostUiAction::SaveDocument => Self::SaveDocument,
            StudioAppWindowHostUiAction::UndoDocumentCommand => Self::UndoDocumentCommand,
            StudioAppWindowHostUiAction::RedoDocumentCommand => Self::RedoDocumentCommand,
            StudioAppWindowHostUiAction::RunManualWorkspace => Self::RunManualWorkspace,
            StudioAppWindowHostUiAction::ResumeWorkspace => Self::ResumeWorkspace,
            StudioAppWindowHostUiAction::HoldWorkspace => Self::HoldWorkspace,
            StudioAppWindowHostUiAction::ActivateWorkspace => Self::ActivateWorkspace,
            StudioAppWindowHostUiAction::RecoverRunPanelFailure => Self::RecoverRunPanelFailure,
            StudioAppWindowHostUiAction::SyncEntitlement => Self::SyncEntitlement,
            StudioAppWindowHostUiAction::RefreshOfflineLease => Self::RefreshOfflineLease,
        }
    }
}

impl From<StudioAppWindowHostUiActionDisabledReason> for StudioAppHostUiActionDisabledReason {
    fn from(value: StudioAppWindowHostUiActionDisabledReason) -> Self {
        match value {
            StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow => {
                Self::NoRegisteredWindow
            }
            StudioAppWindowHostUiActionDisabledReason::SaveUnavailable => Self::SaveUnavailable,
            StudioAppWindowHostUiActionDisabledReason::UndoUnavailable => Self::UndoUnavailable,
            StudioAppWindowHostUiActionDisabledReason::RedoUnavailable => Self::RedoUnavailable,
            StudioAppWindowHostUiActionDisabledReason::RunManualUnavailable => {
                Self::RunManualUnavailable
            }
            StudioAppWindowHostUiActionDisabledReason::ResumeUnavailable => Self::ResumeUnavailable,
            StudioAppWindowHostUiActionDisabledReason::HoldUnavailable => Self::HoldUnavailable,
            StudioAppWindowHostUiActionDisabledReason::ActivateUnavailable => {
                Self::ActivateUnavailable
            }
            StudioAppWindowHostUiActionDisabledReason::NoRunPanelRecovery => {
                Self::NoRunPanelRecovery
            }
            StudioAppWindowHostUiActionDisabledReason::SyncEntitlementUnavailable => {
                Self::SyncEntitlementUnavailable
            }
            StudioAppWindowHostUiActionDisabledReason::RefreshOfflineLeaseUnavailable => {
                Self::RefreshOfflineLeaseUnavailable
            }
        }
    }
}

pub(super) fn ui_action_state_from_window_host(
    state: StudioAppWindowHostUiActionState,
) -> StudioAppHostUiActionState {
    StudioAppHostUiActionState {
        action: state.action.into(),
        availability: match state.availability {
            StudioAppWindowHostUiActionAvailability::Enabled { target_window_id } => {
                StudioAppHostUiActionAvailability::Enabled { target_window_id }
            }
            StudioAppWindowHostUiActionAvailability::Disabled {
                reason,
                target_window_id,
            } => StudioAppHostUiActionAvailability::Disabled {
                reason: reason.into(),
                target_window_id,
            },
        },
    }
}

pub(super) fn ui_command_model_from_states(
    states: &[StudioAppHostUiActionState],
) -> StudioAppHostUiCommandModel {
    let mut actions: Vec<_> = states
        .iter()
        .cloned()
        .map(ui_action_model_from_state)
        .collect();
    actions.extend(placeholder_ui_command_models());
    actions.sort_by_key(|action| (ui_command_group_sort_key(action.group), action.sort_order));

    StudioAppHostUiCommandModel { actions }
}

pub(super) fn ui_action_model_from_state(
    state: StudioAppHostUiActionState,
) -> StudioAppHostUiActionModel {
    let (command_id, group, sort_order, label) = match state.action {
        StudioAppHostUiAction::SaveDocument => (
            crate::FILE_SAVE_COMMAND_ID,
            StudioAppHostUiCommandGroup::File,
            10,
            "Save",
        ),
        StudioAppHostUiAction::UndoDocumentCommand => (
            crate::EDIT_UNDO_COMMAND_ID,
            StudioAppHostUiCommandGroup::Edit,
            10,
            "Undo",
        ),
        StudioAppHostUiAction::RedoDocumentCommand => (
            crate::EDIT_REDO_COMMAND_ID,
            StudioAppHostUiCommandGroup::Edit,
            20,
            "Redo",
        ),
        StudioAppHostUiAction::RunManualWorkspace => (
            "run_panel.run_manual",
            StudioAppHostUiCommandGroup::RunPanel,
            100,
            "Run workspace",
        ),
        StudioAppHostUiAction::ResumeWorkspace => (
            "run_panel.resume_workspace",
            StudioAppHostUiCommandGroup::RunPanel,
            110,
            "Resume workspace",
        ),
        StudioAppHostUiAction::HoldWorkspace => (
            "run_panel.set_hold",
            StudioAppHostUiCommandGroup::RunPanel,
            120,
            "Hold workspace",
        ),
        StudioAppHostUiAction::ActivateWorkspace => (
            "run_panel.set_active",
            StudioAppHostUiCommandGroup::RunPanel,
            130,
            "Activate workspace",
        ),
        StudioAppHostUiAction::RecoverRunPanelFailure => (
            "run_panel.recover_failure",
            StudioAppHostUiCommandGroup::Recovery,
            200,
            "Recover run panel failure",
        ),
        StudioAppHostUiAction::SyncEntitlement => (
            "entitlement.sync",
            StudioAppHostUiCommandGroup::Entitlement,
            300,
            "Sync entitlement",
        ),
        StudioAppHostUiAction::RefreshOfflineLease => (
            "entitlement.refresh_offline_lease",
            StudioAppHostUiCommandGroup::Entitlement,
            310,
            "Refresh offline lease",
        ),
    };
    let (enabled, detail, target_window_id) = match state.availability {
        StudioAppHostUiActionAvailability::Enabled { target_window_id } => {
            let detail = match state.action {
                StudioAppHostUiAction::SaveDocument => {
                    "Save the current document to its project path"
                }
                StudioAppHostUiAction::UndoDocumentCommand => {
                    "Undo the latest document command in the target window"
                }
                StudioAppHostUiAction::RedoDocumentCommand => {
                    "Redo the next document command in the target window"
                }
                StudioAppHostUiAction::RunManualWorkspace => {
                    "Dispatch the current manual run action in the target window"
                }
                StudioAppHostUiAction::ResumeWorkspace => {
                    "Dispatch the current resume action in the target window"
                }
                StudioAppHostUiAction::HoldWorkspace => {
                    "Dispatch the current hold action in the target window"
                }
                StudioAppHostUiAction::ActivateWorkspace => {
                    "Dispatch the current activate action in the target window"
                }
                StudioAppHostUiAction::RecoverRunPanelFailure => {
                    "Apply the current run panel recovery action in the target window"
                }
                StudioAppHostUiAction::SyncEntitlement => {
                    "Dispatch the current entitlement sync action in the target window"
                }
                StudioAppHostUiAction::RefreshOfflineLease => {
                    "Dispatch the current offline lease refresh action in the target window"
                }
            };

            (true, detail, Some(target_window_id))
        }
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
            target_window_id,
        } => {
            let detail = match state.action {
                StudioAppHostUiAction::SaveDocument => {
                    "Open a studio window before saving the document"
                }
                StudioAppHostUiAction::UndoDocumentCommand => {
                    "Open a studio window before undoing document commands"
                }
                StudioAppHostUiAction::RedoDocumentCommand => {
                    "Open a studio window before redoing document commands"
                }
                StudioAppHostUiAction::RunManualWorkspace => {
                    "Open a studio window before running the workspace"
                }
                StudioAppHostUiAction::ResumeWorkspace => {
                    "Open a studio window before resuming the workspace"
                }
                StudioAppHostUiAction::HoldWorkspace => {
                    "Open a studio window before holding the workspace"
                }
                StudioAppHostUiAction::ActivateWorkspace => {
                    "Open a studio window before activating the workspace"
                }
                StudioAppHostUiAction::RecoverRunPanelFailure => {
                    "Open a studio window before requesting run panel recovery"
                }
                StudioAppHostUiAction::SyncEntitlement => {
                    "Open a studio window before syncing entitlement"
                }
                StudioAppHostUiAction::RefreshOfflineLease => {
                    "Open a studio window before refreshing the offline lease"
                }
            };

            (false, detail, target_window_id)
        }
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::SaveUnavailable,
            target_window_id,
        } => (
            false,
            "The current document has no project path; use Save As from the workspace panel",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::UndoUnavailable,
            target_window_id,
        } => (
            false,
            "There is no document command to undo in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::RedoUnavailable,
            target_window_id,
        } => (
            false,
            "There is no document command to redo in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::RunManualUnavailable,
            target_window_id,
        } => (
            false,
            "Manual run is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::ResumeUnavailable,
            target_window_id,
        } => (
            false,
            "Resume is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::HoldUnavailable,
            target_window_id,
        } => (
            false,
            "Hold is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::ActivateUnavailable,
            target_window_id,
        } => (
            false,
            "Activate is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::NoRunPanelRecovery,
            target_window_id,
        } => (
            false,
            "No run panel recovery action is currently available in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::SyncEntitlementUnavailable,
            target_window_id,
        } => (
            false,
            "Sync entitlement is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::RefreshOfflineLeaseUnavailable,
            target_window_id,
        } => (
            false,
            "Offline lease refresh is currently unavailable in the target window",
            target_window_id,
        ),
    };

    StudioAppHostUiActionModel {
        action: Some(state.action),
        command_id,
        group,
        sort_order,
        label,
        enabled,
        detail,
        target_window_id,
    }
}

pub(super) fn placeholder_ui_command_models() -> Vec<StudioAppHostUiActionModel> {
    Vec::new()
}

pub(super) fn ui_command_group_sort_key(group: StudioAppHostUiCommandGroup) -> u16 {
    match group {
        StudioAppHostUiCommandGroup::File => 10,
        StudioAppHostUiCommandGroup::Edit => 20,
        StudioAppHostUiCommandGroup::RunPanel => 100,
        StudioAppHostUiCommandGroup::Recovery => 200,
        StudioAppHostUiCommandGroup::Entitlement => 300,
    }
}

pub(super) fn map_outcome(
    outcome: StudioAppWindowHostCommandOutcome,
) -> StudioAppHostCommandOutcome {
    match outcome {
        StudioAppWindowHostCommandOutcome::WindowOpened(opened) => {
            StudioAppHostCommandOutcome::WindowOpened(opened)
        }
        StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch)
        }
        StudioAppWindowHostCommandOutcome::CanvasInteracted(result) => {
            StudioAppHostCommandOutcome::CanvasInteracted(result)
        }
        StudioAppWindowHostCommandOutcome::WindowClosed(close) => {
            StudioAppHostCommandOutcome::WindowClosed(close)
        }
        StudioAppWindowHostCommandOutcome::IgnoredUiAction => {
            StudioAppHostCommandOutcome::IgnoredUiAction
        }
        StudioAppWindowHostCommandOutcome::IgnoredGlobalEvent { event } => {
            StudioAppHostCommandOutcome::IgnoredGlobalEvent { event }
        }
        StudioAppWindowHostCommandOutcome::IgnoredClose { window_id } => {
            StudioAppHostCommandOutcome::IgnoredClose { window_id }
        }
    }
}
