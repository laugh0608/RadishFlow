//! Session-local return navigation for read-only run diagnostic actions.
use super::*;
use radishflow_studio::StudioGuiWindowCommandActionModel;
use radishflow_studio::modeling_specifications::SpecificationTarget;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RunDiagnosticSurface {
    Toolbar,
    Drawer,
    Runtime(StudioGuiWindowAreaId),
}

pub(super) struct RunDiagnosticNavigation {
    screen: StudioShellScreen,
    window: Option<StudioWindowHostId>,
    drawer: StudioShellBottomDrawerTab,
    inspector: StudioShellRightSidebarTab,
    runtime_details: bool,
    background_details: bool,
    surface: RunDiagnosticSurface,
    button: egui::Id,
    action: StudioGuiWindowCommandActionModel,
    target: SpecificationTarget,
    phase: ReturnPhase,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReturnPhase {
    AtTarget,
    RestoreButton,
    ShowNotice {
        reason: ReturnNotice,
        focus_pending: bool,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReturnNotice {
    TargetRemoved,
    ResultChanged,
    ActionUnavailable,
}

impl ReadyAppState {
    pub(super) fn render_flow_page_context(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        let title = match self.screen {
            StudioShellScreen::Run => ShellText::FlowsheetRun,
            StudioShellScreen::Results => ShellText::FlowsheetResults,
            _ => return,
        };
        ui.horizontal_wrapped(|ui| {
            ui.strong(self.locale.text(title));
            render_wrapped_label(ui, &window.runtime.workspace_document.flowsheet_name);
            if ui
                .button(text(self.locale, "运行诊断", "Run diagnostics"))
                .clicked()
            {
                self.open_flow_diagnostics();
            }
        });
    }

    fn open_flow_diagnostics(&mut self) {
        self.bottom_drawer_tab = StudioShellBottomDrawerTab::Diagnostics;
        self.background_run_details_open = false;
        self.modeling_problems.set_runtime_details(true);
    }

    pub(super) fn at_run_diagnostic_target(&self) -> bool {
        self.run_diagnostic_navigation
            .as_ref()
            .is_some_and(|navigation| navigation.phase == ReturnPhase::AtTarget)
    }

    pub(super) fn render_run_diagnostic_return(&mut self, ui: &mut egui::Ui) {
        if self.screen == StudioShellScreen::Workbench
            && self
                .run_diagnostic_navigation
                .as_ref()
                .is_some_and(|navigation| {
                    navigation.phase == ReturnPhase::AtTarget
                        && navigation.window == self.current_window_id()
                })
            && ui
                .button(text(
                    self.locale,
                    "← 返回原运行诊断",
                    "← Back to originating run diagnostic",
                ))
                .clicked()
        {
            ui.input_mut(|input| {
                input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                input.consume_key(egui::Modifiers::NONE, egui::Key::Space);
            });
            self.return_to_run_diagnostic();
        }
    }

    fn return_to_run_diagnostic(&mut self) {
        let Some(mut navigation) = self.run_diagnostic_navigation.take() else {
            return;
        };
        self.modeling_problems.clear_destination();
        self.screen = navigation.screen;
        self.bottom_drawer_tab = navigation.drawer;
        self.right_sidebar_tab = navigation.inspector;
        self.background_run_details_open = navigation.background_details;
        self.modeling_problems
            .set_runtime_details(navigation.runtime_details);
        let workspace = &self.platform_host.modeling_inputs().workspace;
        navigation.phase = if !navigation.target.exists(&workspace.document) {
            ReturnPhase::ShowNotice {
                reason: ReturnNotice::TargetRemoved,
                focus_pending: true,
            }
        } else if !navigation.action.result_is_current(workspace) {
            ReturnPhase::ShowNotice {
                reason: ReturnNotice::ResultChanged,
                focus_pending: true,
            }
        } else {
            ReturnPhase::RestoreButton
        };
        self.run_diagnostic_navigation = Some(navigation);
    }

    pub(super) fn activate_run_diagnostic(
        &mut self,
        surface: RunDiagnosticSurface,
        button: egui::Id,
        action: &StudioGuiWindowCommandActionModel,
        port: Option<rf_types::DiagnosticPortTarget>,
    ) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let recovery_command = action.command_id == "run_panel.recover_failure";
        let target = if recovery_command {
            workspace
                .run_panel
                .notice
                .as_ref()
                .and_then(|notice| notice.recovery_action.as_ref())
                .filter(|recovery| recovery.mutation.is_none())
                .and_then(recovery_target)
        } else if let Some(port) = port {
            Some(SpecificationTarget::Port(port))
        } else {
            radishflow_studio::inspector_target_from_command_id(&action.command_id)
                .map(specification_target)
        };
        let Some(target) = target.filter(|target| {
            action.result_guard.is_some()
                && action.result_is_current(workspace)
                && target.exists(&workspace.document)
        }) else {
            self.dispatch_result_action(action);
            return;
        };
        let origin = RunDiagnosticNavigation {
            screen: self.screen,
            window: self.current_window_id(),
            drawer: self.bottom_drawer_tab,
            inspector: self.right_sidebar_tab,
            runtime_details: self.modeling_problems.runtime_details(),
            background_details: self.background_run_details_open,
            surface,
            button,
            action: action.clone(),
            target: target.clone(),
            phase: ReturnPhase::AtTarget,
        };
        self.dispatch_result_action(action);
        self.screen = StudioShellScreen::Workbench;
        self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
        if let Some(view) = origin.window.map(rf_ui::DisplayUnitViewId) {
            self.unit_settings.closed_inspectors.remove(&view);
        }
        if matches!(target, SpecificationTarget::Port(_)) {
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::Diagnostics;
            self.background_run_details_open = false;
            self.modeling_problems.set_runtime_details(false);
        }
        self.modeling_problems.show_destination(target);
        self.run_diagnostic_navigation = Some(origin);
    }

    fn returning_to_run_diagnostic(&self, surface: RunDiagnosticSurface) -> bool {
        self.run_diagnostic_navigation
            .as_ref()
            .is_some_and(|navigation| {
                navigation.phase != ReturnPhase::AtTarget
                    && navigation.surface == surface
                    && navigation.screen == self.screen
                    && navigation.window == self.current_window_id()
                    && (surface != RunDiagnosticSurface::Drawer
                        || (navigation.drawer == self.bottom_drawer_tab
                            && navigation.runtime_details
                                == self.modeling_problems.runtime_details()
                            && navigation.background_details == self.background_run_details_open))
            })
    }

    pub(super) fn restore_run_diagnostic_button(
        &mut self,
        surface: RunDiagnosticSurface,
        action: &StudioGuiWindowCommandActionModel,
        response: &egui::Response,
    ) {
        if self.returning_to_run_diagnostic(surface)
            && self
                .run_diagnostic_navigation
                .as_ref()
                .is_some_and(|navigation| {
                    navigation.phase == ReturnPhase::RestoreButton
                        && navigation.button == response.id
                        && &navigation.action == action
                        && response.enabled()
                })
        {
            response.scroll_to_me(None);
            response.request_focus();
            self.run_diagnostic_navigation = None;
        }
    }

    pub(super) fn render_run_diagnostic_return_notice(
        &mut self,
        ui: &mut egui::Ui,
        surface: RunDiagnosticSurface,
    ) {
        if !self.returning_to_run_diagnostic(surface) {
            return;
        }
        let navigation = self
            .run_diagnostic_navigation
            .as_mut()
            .expect("matching return");
        let (reason, needs_focus) = match navigation.phase {
            ReturnPhase::RestoreButton => (ReturnNotice::ActionUnavailable, true),
            ReturnPhase::ShowNotice {
                reason,
                focus_pending,
            } => (reason, focus_pending),
            ReturnPhase::AtTarget => unreachable!("already in the target"),
        };
        let message = match reason {
            ReturnNotice::TargetRemoved => text(
                self.locale,
                "原诊断目标已移除。",
                "The original diagnostic target was removed.",
            ),
            ReturnNotice::ResultChanged => text(
                self.locale,
                "运行或输入已变化，原诊断已失效。",
                "The run or inputs changed. The original diagnostic is no longer current.",
            ),
            ReturnNotice::ActionUnavailable => text(
                self.locale,
                "原诊断操作在当前视图中不可用。",
                "The original diagnostic action is unavailable in this view.",
            ),
        };
        let response = ui.add(
            egui::Label::new(message)
                .wrap()
                .sense(egui::Sense::focusable_noninteractive()),
        );
        if needs_focus {
            response.scroll_to_me(None);
            response.request_focus();
        }
        navigation.phase = ReturnPhase::ShowNotice {
            reason,
            focus_pending: false,
        };
    }
}

fn recovery_target(action: &rf_ui::RunPanelRecoveryAction) -> Option<SpecificationTarget> {
    match (
        &action.target_unit_id,
        &action.target_stream_id,
        &action.target_port_name,
    ) {
        (Some(unit), _, Some(port)) => Some(SpecificationTarget::Port(
            rf_types::DiagnosticPortTarget::new(unit.clone(), port.clone()),
        )),
        (Some(unit), _, _) => Some(SpecificationTarget::Unit(unit.clone())),
        (_, Some(stream), _) => Some(SpecificationTarget::Stream(stream.clone())),
        _ => None,
    }
}

fn specification_target(target: rf_ui::InspectorTarget) -> SpecificationTarget {
    match target {
        rf_ui::InspectorTarget::Unit(unit) => SpecificationTarget::Unit(unit),
        rf_ui::InspectorTarget::Stream(stream) => SpecificationTarget::Stream(stream),
    }
}

fn text(locale: StudioShellLocale, zh: &'static str, en: &'static str) -> &'static str {
    match locale {
        StudioShellLocale::ZhCn => zh,
        StudioShellLocale::En => en,
    }
}
