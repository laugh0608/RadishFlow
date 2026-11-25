use super::*;
use radishflow_studio::{
    StudioRunActivity, UnitControlReply, UnitRunCommand, UnitRunControl, UnitRunPhase,
};

pub(in crate::studio_gui_shell) const START_PAUSED: &str = "run_panel.pause_at_first_unit";

fn labels(command: UnitRunCommand) -> (&'static str, &'static str, &'static str) {
    match command {
        UnitRunCommand::Pause { .. } => ("pause", "暂停", "Pause"),
        UnitRunCommand::Step { .. } => ("step", "单步（一个单元）", "Step one unit"),
        UnitRunCommand::Continue { .. } => ("continue", "继续本次运行", "Continue this run"),
    }
}

fn commands(target: &StudioRunActivity) -> Vec<UnitRunCommand> {
    if target.phase == WorkspaceRunPhase::CancelRequested {
        return Vec::new();
    }
    match &target.unit_control {
        Some(UnitRunControl {
            phase: UnitRunPhase::Running,
            last_released,
            ..
        }) => vec![UnitRunCommand::Pause {
            last_released: *last_released,
        }],
        Some(UnitRunControl {
            phase: UnitRunPhase::Paused { step, .. },
            ..
        }) => vec![
            UnitRunCommand::Step { step: *step },
            UnitRunCommand::Continue { step: *step },
        ],
        _ => Vec::new(),
    }
}

fn command_id(number: u64, command: UnitRunCommand) -> String {
    let position = match command {
        UnitRunCommand::Pause { last_released } => last_released,
        UnitRunCommand::Step { step } | UnitRunCommand::Continue { step } => step,
    };
    format!(
        "background.unit.{}.{}.{}",
        labels(command).0,
        number,
        position
    )
}

pub(super) fn control_feedback(reply: UnitControlReply, zh: bool) -> Option<&'static str> {
    let (cn, en) = match reply {
        UnitControlReply::Accepted => return None,
        UnitControlReply::AlreadyRequested => {
            ("已请求在下一单元前暂停。", "A pause is already requested.")
        }
        UnitControlReply::Stale => (
            "停点已变化，请重新选择。",
            "The stop changed. Select it again.",
        ),
        UnitControlReply::NotPaused => (
            "本次运行尚未停在单元边界。",
            "This run is not paused at a unit boundary.",
        ),
        UnitControlReply::Cancelling => ("本次运行正在取消。", "This run is being cancelled."),
        UnitControlReply::Finished => ("本次运行已结束。", "This run has finished."),
        UnitControlReply::UnknownRun => ("目标运行已不可用。", "The target run is unavailable."),
        UnitControlReply::Unsupported => (
            "此任务不支持单元暂停。",
            "This task does not support pausing at units.",
        ),
    };
    Some(if zh { cn } else { en })
}

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn start_paused_button(&mut self, ui: &mut egui::Ui) {
        if self.read_only_project.is_some()
            || self.background_runs.is_none()
            || self
                .platform_host
                .modeling_inputs()
                .workspace
                .flow_plan
                .current()
                != &rf_ui::flow_plan::FlowPlanConfiguration::Acyclic
        {
            return;
        }
        let label = if self.locale == StudioShellLocale::ZhCn {
            "运行并在首步前暂停"
        } else {
            "Run and pause before first unit"
        };
        let response = ui.add_enabled(self.can_start_background_flow(), egui::Button::new(label));
        if response.has_focus() {
            response.scroll_to_me(None);
        }
        if response.clicked() {
            self.dispatch_ui_command(START_PAUSED);
        }
    }

    pub(in crate::studio_gui_shell) fn start_flow_paused(&mut self) {
        let expected = radishflow_studio::flow_actions::FlowEditContext::capture(
            &self.platform_host.modeling_inputs().workspace,
        );
        let access = if self.read_only_project.is_some() {
            radishflow_studio::run_actions::StudioRunAccess::ReadOnly
        } else {
            radishflow_studio::run_actions::StudioRunAccess::Allowed
        };
        self.background_action(Action::StartAtFirstUnit { expected, access });
    }

    pub(in crate::studio_gui_shell) fn unit_control_status(
        &self,
        target: &StudioRunActivity,
    ) -> Option<String> {
        if target.phase == WorkspaceRunPhase::CancelRequested {
            return None;
        }
        let zh = self.locale == StudioShellLocale::ZhCn;
        let control = target.unit_control.as_ref()?;
        if control.observation.as_ref().is_some_and(|watch| {
            matches!(
                watch.reading,
                radishflow_studio::outlet_temperature::TemperatureReading::Unavailable { .. }
            )
        }) {
            return Some(
                if zh {
                    "监视值不可用 · 正在终止本次运行"
                } else {
                    "Monitored value unavailable · terminating this run"
                }
                .into(),
            );
        }
        Some(match &control.phase {
            UnitRunPhase::PauseRequested => if zh {
                "正在暂停 · 等待单元边界"
            } else {
                "Pause requested · waiting for a unit boundary"
            }
            .into(),
            UnitRunPhase::Stepping => if zh {
                "单步执行中"
            } else {
                "Stepping one unit"
            }
            .into(),
            UnitRunPhase::Paused { step, total, unit } => {
                let done = step - 1;
                if zh {
                    format!("已暂停 · 已完成 {done}/{total} · 下一单元 {unit} 尚未开始")
                } else {
                    format!("Paused · {done}/{total} completed · next unit {unit} has not started")
                }
            }
            _ => return None,
        })
    }

    pub(super) fn render_unit_control_buttons(
        &mut self,
        ui: &mut egui::Ui,
        target: &StudioRunActivity,
        status: &egui::Response,
    ) {
        for command in commands(target) {
            let (_, cn, en) = labels(command);
            let response = ui.button(if self.locale == StudioShellLocale::ZhCn {
                cn
            } else {
                en
            });
            if response.has_focus() {
                response.scroll_to_me(None);
            }
            if response.clicked() {
                status.request_focus();
                self.background_action(Action::ControlUnits {
                    id: target.id.clone(),
                    command,
                });
            }
        }
    }

    pub(super) fn append_unit_commands(
        &self,
        items: &mut Vec<radishflow_studio::StudioGuiWindowCommandPaletteItemModel>,
        target: &StudioRunActivity,
        query: &str,
    ) {
        let current = self
            .background_runs
            .as_ref()
            .and_then(|owner| owner.borrow().activity());
        let zh = self.locale == StudioShellLocale::ZhCn;
        for command in commands(target) {
            let (_, cn, en) = labels(command);
            let label = format!("{} #{}", if zh { cn } else { en }, target.number);
            let search = format!("{cn} {en} {label}").to_lowercase();
            if !query
                .to_lowercase()
                .split_whitespace()
                .all(|term| search.contains(term))
            {
                continue;
            }
            let enabled = current
                .as_ref()
                .is_some_and(|a| a.id == target.id && commands(a).contains(&command));
            items.insert(
                0,
                radishflow_studio::StudioGuiWindowCommandPaletteItemModel {
                    command_id: command_id(target.number, command),
                    enabled,
                    label,
                    detail: target.target.clone(),
                    hover_text: target.target.clone(),
                    menu_path_text: if zh { "流程运行" } else { "Flowsheet run" }.into(),
                },
            );
        }
    }

    pub(in crate::studio_gui_shell) fn dispatch_unit_command(&mut self, id: &str) -> bool {
        if !id.starts_with("background.unit.") {
            return false;
        }
        if let Some(target) = &self.command_palette.background_target
            && let Some(command) = commands(target)
                .into_iter()
                .find(|c| command_id(target.number, *c) == id)
        {
            self.background_action(Action::ControlUnits {
                id: target.id.clone(),
                command,
            });
        }
        true
    }
}
