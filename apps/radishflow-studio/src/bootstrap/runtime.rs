use std::{path::Path, time::SystemTime};

use crate::{
    EntitlementSessionEvent, EntitlementSessionEventDriverOutcome, EntitlementSessionHostDispatch,
    EntitlementSessionHostRuntime, EntitlementSessionHostTrigger, EntitlementSessionLifecycleEvent,
    EntitlementSessionPanelDriverOutcome, EntitlementSessionPolicy, EntitlementSessionRuntime,
    EntitlementSessionState, RunPanelDriverOutcome, StudioAppAuthCacheContext,
    StudioAppCommandOutcome, StudioAppMutableAuthCacheContext, WorkspaceControlActionOutcome,
    apply_run_panel_recovery_action, commit_inspector_draft, commit_inspector_drafts,
    discard_inspector_draft, discard_inspector_drafts, dispatch_document_history,
    dispatch_document_lifecycle, dispatch_entitlement_session_event_with_control_plane,
    dispatch_run_panel_intent_with_auth_cache, dispatch_run_panel_primary_action_with_auth_cache,
    dispatch_run_panel_widget_action_with_auth_cache, focus_inspector_target,
    normalize_inspector_composition, snapshot_entitlement_session_driver_state,
    snapshot_entitlement_session_schedule, snapshot_run_panel_driver_state, update_inspector_draft,
};
use rf_store::{StoredAuthCacheIndex, read_project_file};
use rf_types::{RfError, RfResult};
use rf_ui::AppState;

use super::seed::{
    BOOTSTRAP_MVP_PROPERTY_PACKAGE_ID, BootstrapControlPlaneClient, app_state_from_project_file,
    initialize_blank_project_thermo_basis, normalized_system_time_now,
    seed_bootstrap_runtime_state, seed_sample_auth_cache,
};
use super::temp_cache::TemporaryCacheRoot;
use super::{
    BootstrapSession, StudioBootstrapConfig, StudioBootstrapDispatch,
    StudioBootstrapEntitlementPreflight, StudioBootstrapEntitlementSessionEvent,
    StudioBootstrapReport, StudioBootstrapTrigger,
};

struct BootstrapSessionResources<'a> {
    facade: &'a crate::StudioAppFacade,
    app_state: &'a mut AppState,
    cache_root: &'a Path,
    auth_cache_index: &'a mut StoredAuthCacheIndex,
    control_plane_client: &'a BootstrapControlPlaneClient,
    policy: &'a crate::EntitlementSessionPolicy,
    session_state: &'a mut crate::EntitlementSessionState,
    host_runtime: &'a mut crate::EntitlementSessionHostRuntime,
}

fn dispatch_bootstrap_entitlement_session_tick(
    mode: &StudioBootstrapEntitlementPreflight,
    session: &mut BootstrapSessionResources<'_>,
) -> RfResult<EntitlementSessionEventDriverOutcome> {
    if matches!(mode, StudioBootstrapEntitlementPreflight::Skip) {
        let now = normalized_system_time_now()?;
        let outcome = EntitlementSessionEventDriverOutcome {
            event: EntitlementSessionEvent::SessionStarted,
            outcome: crate::EntitlementSessionEventOutcome::Tick(Box::new(
                crate::EntitlementSessionTickOutcome {
                    preflight: None,
                    schedule: snapshot_entitlement_session_schedule(
                        session.app_state,
                        now,
                        session.policy,
                        session.session_state,
                    ),
                },
            )),
            state: snapshot_entitlement_session_driver_state(
                session.app_state,
                now,
                session.policy,
                session.session_state,
            ),
        };
        session.host_runtime.snapshot(
            session.app_state,
            now,
            session.policy,
            session.session_state,
        );
        return Ok(outcome);
    }

    let mut context =
        StudioAppMutableAuthCacheContext::new(session.cache_root, session.auth_cache_index);
    let now = normalized_system_time_now()?;
    let mut runtime = EntitlementSessionRuntime {
        facade: session.facade,
        app_state: session.app_state,
        context: &mut context,
        control_plane_client: session.control_plane_client,
        access_token: "bootstrap-access-token",
        now,
        policy: session.policy,
        session_state: session.session_state,
    };
    let outcome = dispatch_entitlement_session_event_with_control_plane(
        EntitlementSessionEvent::SessionStarted,
        &mut runtime,
    )?;
    session.host_runtime.snapshot(
        session.app_state,
        now,
        session.policy,
        session.session_state,
    );
    Ok(outcome)
}

fn dispatch_bootstrap_trigger(
    trigger: &StudioBootstrapTrigger,
    session: &mut BootstrapSessionResources<'_>,
) -> RfResult<StudioBootstrapDispatch> {
    match trigger {
        StudioBootstrapTrigger::AppCommand(command) => {
            let context =
                StudioAppAuthCacheContext::new(session.cache_root, &*session.auth_cache_index);
            Ok(StudioBootstrapDispatch::AppCommand(
                session
                    .facade
                    .execute_with_auth_cache(session.app_state, &context, command)?,
            ))
        }
        StudioBootstrapTrigger::Intent(intent) => {
            let context =
                StudioAppAuthCacheContext::new(session.cache_root, &*session.auth_cache_index);
            command_outcome_from_workspace_control(dispatch_run_panel_intent_with_auth_cache(
                session.facade,
                session.app_state,
                &context,
                intent,
            )?)
        }
        StudioBootstrapTrigger::WidgetPrimaryAction => {
            let context =
                StudioAppAuthCacheContext::new(session.cache_root, &*session.auth_cache_index);
            match dispatch_run_panel_primary_action_with_auth_cache(
                session.facade,
                session.app_state,
                &context,
            )? {
                RunPanelDriverOutcome {
                    dispatch: crate::RunPanelWidgetDispatchOutcome::Executed(outcome),
                    ..
                } => command_outcome_from_workspace_control(*outcome),
                RunPanelDriverOutcome {
                    dispatch:
                        crate::RunPanelWidgetDispatchOutcome::IgnoredDisabled { action_id, detail },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap primary widget action `{:?}` is currently disabled: {}",
                    action_id, detail
                ))),
                RunPanelDriverOutcome {
                    dispatch: crate::RunPanelWidgetDispatchOutcome::IgnoredMissing { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap primary widget action `{:?}` is missing from current widget model",
                    action_id
                ))),
            }
        }
        StudioBootstrapTrigger::WidgetAction(action_id) => {
            let context =
                StudioAppAuthCacheContext::new(session.cache_root, &*session.auth_cache_index);
            match dispatch_run_panel_widget_action_with_auth_cache(
                session.facade,
                session.app_state,
                &context,
                *action_id,
            )? {
                RunPanelDriverOutcome {
                    dispatch: crate::RunPanelWidgetDispatchOutcome::Executed(outcome),
                    ..
                } => command_outcome_from_workspace_control(*outcome),
                RunPanelDriverOutcome {
                    dispatch:
                        crate::RunPanelWidgetDispatchOutcome::IgnoredDisabled { action_id, detail },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap widget action `{:?}` is currently disabled: {}",
                    action_id, detail
                ))),
                RunPanelDriverOutcome {
                    dispatch: crate::RunPanelWidgetDispatchOutcome::IgnoredMissing { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap widget action `{:?}` is missing from current widget model",
                    action_id
                ))),
            }
        }
        StudioBootstrapTrigger::WidgetRecoveryAction => apply_run_panel_recovery_action(
            session.app_state,
        )
        .map(StudioBootstrapDispatch::RunPanelRecovery)
        .ok_or_else(|| {
            RfError::invalid_input(
                "bootstrap run panel recovery action is unavailable in current widget model",
            )
        }),
        StudioBootstrapTrigger::DocumentLifecycle(command) => {
            let outcome = dispatch_document_lifecycle(session.app_state, command.clone())?;
            Ok(StudioBootstrapDispatch::DocumentLifecycle(outcome))
        }
        StudioBootstrapTrigger::InspectorTarget(target) => {
            let outcome = focus_inspector_target(session.app_state, target.clone());
            if outcome.applied_target.is_none() {
                return Err(RfError::invalid_input(format!(
                    "bootstrap inspector target `{target:?}` is not available in current workspace"
                )));
            }
            Ok(StudioBootstrapDispatch::InspectorTarget(outcome))
        }
        StudioBootstrapTrigger::InspectorDraftUpdate(command) => {
            let outcome = update_inspector_draft(session.app_state, command.clone())?;
            Ok(StudioBootstrapDispatch::InspectorDraftUpdate(outcome))
        }
        StudioBootstrapTrigger::InspectorDraftCommit(command) => {
            let outcome = commit_inspector_draft(session.app_state, command.clone())?;
            Ok(StudioBootstrapDispatch::InspectorDraftCommit(outcome))
        }
        StudioBootstrapTrigger::InspectorDraftDiscard(command) => {
            let outcome = discard_inspector_draft(session.app_state, command.clone())?;
            Ok(StudioBootstrapDispatch::InspectorDraftDiscard(outcome))
        }
        StudioBootstrapTrigger::InspectorDraftBatchCommit(command) => {
            let outcome = commit_inspector_drafts(session.app_state, command.clone())?;
            Ok(StudioBootstrapDispatch::InspectorDraftBatchCommit(outcome))
        }
        StudioBootstrapTrigger::InspectorDraftBatchDiscard(command) => {
            let outcome = discard_inspector_drafts(session.app_state, command.clone())?;
            Ok(StudioBootstrapDispatch::InspectorDraftBatchDiscard(outcome))
        }
        StudioBootstrapTrigger::InspectorCompositionNormalize(command) => {
            let outcome = normalize_inspector_composition(session.app_state, command.clone())?;
            Ok(StudioBootstrapDispatch::InspectorCompositionNormalize(
                outcome,
            ))
        }
        StudioBootstrapTrigger::DocumentHistory(command) => {
            let outcome = dispatch_document_history(session.app_state, *command)?;
            Ok(StudioBootstrapDispatch::DocumentHistory(outcome))
        }
        StudioBootstrapTrigger::EntitlementWidgetPrimaryAction => {
            dispatch_bootstrap_entitlement_host_trigger(
                session,
                EntitlementSessionHostTrigger::PanelPrimaryAction,
            )
        }
        StudioBootstrapTrigger::EntitlementWidgetAction(action_id) => {
            dispatch_bootstrap_entitlement_host_trigger(
                session,
                EntitlementSessionHostTrigger::PanelAction(*action_id),
            )
        }
        StudioBootstrapTrigger::EntitlementSessionEvent(event) => {
            let trigger = match event {
                StudioBootstrapEntitlementSessionEvent::LoginCompleted => {
                    EntitlementSessionHostTrigger::LifecycleEvent(
                        EntitlementSessionLifecycleEvent::LoginCompleted,
                    )
                }
                StudioBootstrapEntitlementSessionEvent::TimerElapsed => {
                    EntitlementSessionHostTrigger::LifecycleEvent(
                        EntitlementSessionLifecycleEvent::TimerElapsed,
                    )
                }
                StudioBootstrapEntitlementSessionEvent::NetworkRestored => {
                    EntitlementSessionHostTrigger::LifecycleEvent(
                        EntitlementSessionLifecycleEvent::NetworkRestored,
                    )
                }
                StudioBootstrapEntitlementSessionEvent::WindowForegrounded => {
                    EntitlementSessionHostTrigger::LifecycleEvent(
                        EntitlementSessionLifecycleEvent::WindowForegrounded,
                    )
                }
            };
            dispatch_bootstrap_entitlement_host_trigger(session, trigger)
        }
    }
}

fn dispatch_bootstrap_entitlement_host_trigger(
    session: &mut BootstrapSessionResources<'_>,
    trigger: EntitlementSessionHostTrigger,
) -> RfResult<StudioBootstrapDispatch> {
    let mut context =
        StudioAppMutableAuthCacheContext::new(session.cache_root, session.auth_cache_index);
    let mut runtime = EntitlementSessionRuntime {
        facade: session.facade,
        app_state: session.app_state,
        context: &mut context,
        control_plane_client: session.control_plane_client,
        access_token: "bootstrap-access-token",
        now: normalized_system_time_now()?,
        policy: session.policy,
        session_state: session.session_state,
    };
    let outcome = session
        .host_runtime
        .dispatch_trigger_with_control_plane(trigger, &mut runtime)?;
    match outcome.dispatch {
        EntitlementSessionHostDispatch::Event(outcome) => {
            Ok(StudioBootstrapDispatch::EntitlementSessionEvent(outcome))
        }
        EntitlementSessionHostDispatch::Panel(EntitlementSessionPanelDriverOutcome {
            dispatch: crate::EntitlementPanelWidgetDispatchOutcome::Executed(outcome),
            ..
        }) => Ok(StudioBootstrapDispatch::AppCommand(outcome)),
        EntitlementSessionHostDispatch::Panel(EntitlementSessionPanelDriverOutcome {
            dispatch:
                crate::EntitlementPanelWidgetDispatchOutcome::IgnoredDisabled { action_id, detail },
            ..
        }) => Err(RfError::invalid_input(format!(
            "bootstrap entitlement action `{:?}` is currently disabled: {}",
            action_id, detail
        ))),
        EntitlementSessionHostDispatch::Panel(EntitlementSessionPanelDriverOutcome {
            dispatch: crate::EntitlementPanelWidgetDispatchOutcome::IgnoredMissing { action_id },
            ..
        }) => Err(RfError::invalid_input(format!(
            "bootstrap entitlement action `{:?}` is missing from current widget model",
            action_id
        ))),
    }
}

fn command_outcome_from_workspace_control(
    outcome: WorkspaceControlActionOutcome,
) -> RfResult<StudioBootstrapDispatch> {
    Ok(StudioBootstrapDispatch::AppCommand(
        StudioAppCommandOutcome {
            boundary: outcome.boundary,
            dispatch: outcome.dispatch,
        },
    ))
}

impl BootstrapSession {
    pub(crate) fn new(config: &StudioBootstrapConfig) -> RfResult<Self> {
        let project_file = read_project_file(&config.project_path)?;
        let mut app_state = app_state_from_project_file(&project_file, &config.project_path);
        initialize_blank_project_thermo_basis(&mut app_state, normalized_system_time_now()?)?;
        let cache_root = TemporaryCacheRoot::new("studio-bootstrap")?;
        let seeded_auth_cache = seed_sample_auth_cache(
            cache_root.path(),
            &app_state.workspace.document.flowsheet,
            BOOTSTRAP_MVP_PROPERTY_PACKAGE_ID,
            config.entitlement_seed,
        )?;
        seed_bootstrap_runtime_state(&mut app_state, &seeded_auth_cache);
        let control_plane_client = BootstrapControlPlaneClient::from_seed(&seeded_auth_cache);
        let mut session = Self {
            app_state,
            cache_root,
            auth_cache_index: seeded_auth_cache.auth_cache_index,
            control_plane_client,
            facade: crate::StudioAppFacade::new(),
            session_policy: EntitlementSessionPolicy::default(),
            entitlement_session_state: EntitlementSessionState::default(),
            host_runtime: EntitlementSessionHostRuntime::default(),
            entitlement_preflight: None,
        };
        session.run_initial_preflight(&config.entitlement_preflight)?;
        Ok(session)
    }

    fn run_initial_preflight(
        &mut self,
        mode: &StudioBootstrapEntitlementPreflight,
    ) -> RfResult<()> {
        let entitlement_session_tick = {
            let mut session_resources = self.resources();
            dispatch_bootstrap_entitlement_session_tick(mode, &mut session_resources)?
        };
        self.entitlement_preflight = match entitlement_session_tick.outcome {
            crate::EntitlementSessionEventOutcome::Tick(tick) => tick.preflight,
            crate::EntitlementSessionEventOutcome::RecordedCommand { .. } => None,
        };
        Ok(())
    }

    pub(crate) fn run_trigger(
        &mut self,
        trigger: &StudioBootstrapTrigger,
    ) -> RfResult<StudioBootstrapReport> {
        let dispatch = {
            let mut session_resources = self.resources();
            dispatch_bootstrap_trigger(trigger, &mut session_resources)?
        };
        self.build_report(dispatch)
    }

    fn build_report(
        &mut self,
        dispatch: StudioBootstrapDispatch,
    ) -> RfResult<StudioBootstrapReport> {
        let schedule_now = normalized_system_time_now()?;
        let driver_state = snapshot_run_panel_driver_state(&self.app_state);
        let entitlement_host = match self.host_runtime.last_output() {
            Some(output) => output,
            None => self.host_runtime.snapshot(
                &self.app_state,
                schedule_now,
                &self.session_policy,
                &self.entitlement_session_state,
            ),
        };

        Ok(StudioBootstrapReport {
            entitlement_preflight: self.entitlement_preflight.clone(),
            entitlement_host,
            dispatch,
            control_state: driver_state.control_state,
            run_panel: driver_state.widget,
            log_entries: self.app_state.log_feed.entries.iter().cloned().collect(),
        })
    }

    fn resources(&mut self) -> BootstrapSessionResources<'_> {
        BootstrapSessionResources {
            facade: &self.facade,
            app_state: &mut self.app_state,
            cache_root: self.cache_root.path(),
            auth_cache_index: &mut self.auth_cache_index,
            control_plane_client: &self.control_plane_client,
            policy: &self.session_policy,
            session_state: &mut self.entitlement_session_state,
            host_runtime: &mut self.host_runtime,
        }
    }

    pub(crate) fn entitlement_preflight(&self) -> Option<&crate::EntitlementPreflightOutcome> {
        self.entitlement_preflight.as_ref()
    }

    pub(crate) fn host_runtime(&self) -> &EntitlementSessionHostRuntime {
        &self.host_runtime
    }

    pub(crate) fn app_state(&self) -> &AppState {
        &self.app_state
    }

    pub(crate) fn refresh_local_canvas_suggestions(&mut self) {
        let mut suggestions: Vec<_> = self
            .app_state
            .workspace
            .canvas_interaction
            .suggestions
            .iter()
            .filter(|suggestion| suggestion.source != rf_ui::SuggestionSource::LocalRules)
            .cloned()
            .collect();
        suggestions
            .extend(crate::studio_local_rules::generate_local_canvas_suggestions(&self.app_state));
        self.app_state.replace_canvas_suggestions(suggestions);
    }

    pub(crate) fn replace_canvas_suggestions(&mut self, suggestions: Vec<rf_ui::CanvasSuggestion>) {
        self.app_state.replace_canvas_suggestions(suggestions);
    }

    pub(crate) fn begin_canvas_place_unit(
        &mut self,
        unit_kind: impl Into<String>,
    ) -> rf_ui::CanvasEditIntent {
        self.app_state.begin_canvas_place_unit(unit_kind)
    }

    pub(crate) fn cancel_canvas_pending_edit(&mut self) -> Option<rf_ui::CanvasEditIntent> {
        self.app_state.cancel_canvas_pending_edit()
    }

    pub(crate) fn commit_canvas_pending_edit_at(
        &mut self,
        position: rf_ui::CanvasPoint,
    ) -> RfResult<Option<rf_ui::CanvasEditCommitResult>> {
        let result = self
            .app_state
            .commit_canvas_pending_edit_at(position, SystemTime::now())?;
        if result.is_some() {
            self.refresh_local_canvas_suggestions();
        }
        Ok(result)
    }

    pub(crate) fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        let accepted = self.app_state.accept_focused_canvas_suggestion_by_tab()?;
        if accepted.is_none() {
            return Ok(None);
        }

        self.refresh_local_canvas_suggestions();
        self.dispatch_automatic_run_after_canvas_write_if_needed()?;

        Ok(accepted)
    }

    pub(crate) fn accept_canvas_suggestion(
        &mut self,
        suggestion_id: &rf_ui::CanvasSuggestionId,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        let accepted = self.app_state.accept_canvas_suggestion(suggestion_id)?;
        if accepted.is_none() {
            return Ok(None);
        }

        self.refresh_local_canvas_suggestions();
        self.dispatch_automatic_run_after_canvas_write_if_needed()?;

        Ok(accepted)
    }

    pub(crate) fn reject_focused_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.app_state.reject_focused_canvas_suggestion()
    }

    pub(crate) fn focus_next_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.app_state.focus_next_canvas_suggestion()
    }

    pub(crate) fn focus_previous_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.app_state.focus_previous_canvas_suggestion()
    }

    fn dispatch_automatic_run_after_canvas_write_if_needed(&mut self) -> RfResult<()> {
        let run_panel = &self.app_state.workspace.run_panel;
        if !matches!(run_panel.simulation_mode, rf_ui::SimulationMode::Active)
            || run_panel.pending_reason != Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
        {
            return Ok(());
        }

        let _ = self.run_trigger(&StudioBootstrapTrigger::AppCommand(
            crate::StudioAppCommand::run_workspace(
                crate::WorkspaceRunCommand::automatic_preferred(),
            ),
        ))?;

        Ok(())
    }
}
