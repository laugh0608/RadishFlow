use super::*;

pub(super) fn dispatch_from_controller(
    dispatch: StudioAppHostWindowDispatchResult,
    canvas: StudioGuiCanvasState,
) -> StudioGuiHostDispatch {
    let native_timers = StudioGuiNativeTimerEffects::from_driver(
        &dispatch.effects.native_timer_transitions,
        &dispatch.effects.native_timer_acks,
    );
    StudioGuiHostDispatch {
        ui_commands: ui_commands_from_projection(&dispatch.projection),
        canvas,
        projection: dispatch.projection,
        target_window_id: dispatch.target_window_id,
        effects: dispatch.effects,
        native_timers,
    }
}

pub(super) fn foreground_entitlement_dispatch_detail(
    target_window_id: Option<StudioWindowHostId>,
) -> String {
    match target_window_id {
        Some(window_id) => {
            format!("Foreground entitlement action was not accepted by window #{window_id}")
        }
        None => "Open a window before dispatching entitlement actions".to_string(),
    }
}

pub(super) fn global_event_from_controller(
    result: StudioAppHostGlobalEventResult,
    canvas: StudioGuiCanvasState,
) -> StudioGuiHostGlobalEventDispatch {
    StudioGuiHostGlobalEventDispatch {
        ui_commands: ui_commands_from_projection(&result.projection),
        canvas: canvas.clone(),
        projection: result.projection.clone(),
        dispatch: result
            .dispatch
            .map(|dispatch| dispatch_from_controller(dispatch, canvas)),
    }
}

pub(super) fn ui_commands_from_projection(
    projection: &StudioAppHostProjection,
) -> StudioAppHostUiCommandModel {
    projection.state.ui_command_model()
}

pub(super) fn global_event_from_lifecycle(
    event: StudioGuiHostLifecycleEvent,
) -> StudioAppWindowHostGlobalEvent {
    match event {
        StudioGuiHostLifecycleEvent::WindowForegrounded { .. } => {
            unreachable!("window foregrounding is routed through focus_window before global mapping")
        }
        StudioGuiHostLifecycleEvent::LoginCompleted => {
            StudioAppWindowHostGlobalEvent::LoginCompleted
        }
        StudioGuiHostLifecycleEvent::NetworkRestored => {
            StudioAppWindowHostGlobalEvent::NetworkRestored
        }
        StudioGuiHostLifecycleEvent::TimerElapsed => StudioAppWindowHostGlobalEvent::TimerElapsed,
        StudioGuiHostLifecycleEvent::RunPanelRecoveryRequested => {
            StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested
        }
    }
}
