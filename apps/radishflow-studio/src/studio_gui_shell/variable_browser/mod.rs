//! Session-local browsing and navigation; values always come from the active workspace.
use super::locale::tr;
use super::*;
use rf_ui::project_query::{ProjectField, ProjectObjectRef};
use rf_ui::variable_browser::{ObjectId, StableVariableReference, VariableId};

mod navigation;
mod projection;
mod references;
mod render;

use projection::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum BrowserMode {
    #[default]
    Expanded,
    Side,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum DirectoryView {
    #[default]
    Project,
    Variables,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum SectionFilter {
    #[default]
    All,
    Inputs,
    Results,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BrowserSelection {
    object: ProjectObjectRef,
    field: Option<ProjectField>,
}

#[derive(Debug, Clone)]
enum BrowserNavigation {
    Selection(BrowserSelection),
    StableReference(StableVariableReference),
    EquipmentResult(ProjectObjectRef),
    FlowResults(ProjectObjectRef),
    Property,
}

#[derive(Default)]
pub(super) struct VariableBrowserState {
    pub open: bool,
    pub mode: BrowserMode,
    directory: DirectoryView,
    query: String,
    reference_input: Option<String>,
    section: SectionFilter,
    scope: Option<ProjectObjectRef>,
    selected: Option<BrowserSelection>,
    detail_page: bool,
    session: Option<rf_ui::FlowRunQualification>,
    cache: BrowserIndex,
    row_layout: render::RowLayout,
    notice: Option<String>,
    pending_navigation: Option<BrowserNavigation>,
    pending_focus: Option<VariableId>,
    return_available: bool,
    area_rect: Option<egui::Rect>,
}

enum BrowserIntent {
    Navigate(BrowserNavigation),
    Copy(BrowserSelection),
    CopyReference(BrowserSelection),
    OpenReference(String),
    InitializeReferences,
    Close,
    Flowsheet,
    Dock,
    Expand,
}

impl VariableBrowserState {
    pub(super) fn set_notice(&mut self, notice: String) {
        self.notice = Some(notice);
    }
}

impl ReadyAppState {
    pub(super) fn reconcile_browser_session(&mut self) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if self.read_only_project.is_some()
            || (self.variable_browser.session.is_some()
                && !self.variable_browser.belongs_to(workspace))
        {
            self.variable_browser = VariableBrowserState::default();
            if self.right_sidebar_tab == StudioShellRightSidebarTab::Variables {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            }
        }
    }
    pub(super) fn open_variable_browser(&mut self) {
        self.variable_browser.open = true;
        self.variable_browser.mode = BrowserMode::Expanded;
        self.command_palette.close();
    }

    pub(super) fn browser_expanded(&self) -> bool {
        self.variable_browser.open && self.variable_browser.mode == BrowserMode::Expanded
    }

    pub(super) fn render_variable_browser(&mut self, ctx: &egui::Context) {
        if !self.browser_expanded() {
            return;
        }
        egui::CentralPanel::default().show(ctx, |ui| self.render_variable_browser_content(ui));
    }

    pub(super) fn render_variable_browser_content(&mut self, ui: &mut egui::Ui) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let mut state = std::mem::take(&mut self.variable_browser);
        state.synchronize(workspace, self.locale);
        state.area_rect = Some(ui.max_rect());
        let intent = render::browser(ui, &mut state, workspace, self.locale);
        self.variable_browser = state;
        match intent {
            Some(BrowserIntent::Navigate(target)) => self.request_browser_navigation(target),
            Some(BrowserIntent::Copy(target)) => {
                // Re-read at activation, never copy a cached/rendered result value.
                match self.browser_copy_value(&target) {
                    Ok(value) => ui.ctx().copy_text(value),
                    Err(reason) => self.variable_browser.notice = Some(reason),
                }
            }
            Some(BrowserIntent::CopyReference(target)) => match self.copy_stable_reference(&target)
            {
                Ok(reference) => ui.ctx().copy_text(reference),
                Err(reason) => self.variable_browser.notice = Some(reason),
            },
            Some(BrowserIntent::OpenReference(text)) => self.open_stable_reference(&text),
            Some(BrowserIntent::InitializeReferences) => {
                self.request_object_identity_initialization()
            }
            Some(BrowserIntent::Close) => self.close_variable_browser(),
            Some(BrowserIntent::Flowsheet) => {
                self.close_variable_browser();
                self.screen = StudioShellScreen::Workbench;
            }
            Some(BrowserIntent::Dock) => {
                self.variable_browser.mode = BrowserMode::Side;
                self.screen = StudioShellScreen::Workbench;
                self.right_sidebar_tab = StudioShellRightSidebarTab::Variables;
            }
            Some(BrowserIntent::Expand) => self.open_variable_browser(),
            None => {}
        }
    }

    pub(super) fn close_variable_browser(&mut self) {
        self.variable_browser.open = false;
        if self.right_sidebar_tab == StudioShellRightSidebarTab::Variables {
            self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
        }
    }

    pub(super) fn render_browser_return(&mut self, ui: &mut egui::Ui) {
        if self.variable_browser.return_available
            && ui
                .button(tr(
                    self.locale,
                    "← 返回变量浏览器",
                    "← Return to variable browser",
                ))
                .clicked()
        {
            self.open_variable_browser();
        }
    }

    pub(super) fn browser_copy_value(&self, target: &BrowserSelection) -> Result<String, String> {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if !self.variable_browser.belongs_to(workspace) {
            return Err(tr(
                self.locale,
                "工程会话已变化，请重新选择。",
                "Project session changed; select again.",
            )
            .into());
        }
        let read = read_selection(workspace, target, self.locale)?;
        read.copy.ok_or_else(|| {
            tr(
                self.locale,
                "当前数值不可复制；请检查资格或单位。",
                "The current value cannot be copied; check its status and unit.",
            )
            .into()
        })
    }
}

impl VariableBrowserState {
    pub(super) fn owns_shortcuts(&self, ctx: &egui::Context) -> bool {
        if !self.open {
            return false;
        }
        if self.mode == BrowserMode::Expanded {
            return true;
        }
        let Some(rect) = self.area_rect else {
            return false;
        };
        ctx.memory(|m| m.focused())
            .and_then(|id| ctx.read_response(id))
            .is_some_and(|response| rect.contains(response.rect.center()))
            || ctx
                .pointer_latest_pos()
                .is_some_and(|position| rect.contains(position))
    }

    fn belongs_to(&self, workspace: &rf_ui::WorkspaceState) -> bool {
        self.session
            .as_ref()
            .is_some_and(|s| s.same_session(&workspace.flow_run_qualification()))
    }

    fn synchronize(&mut self, workspace: &rf_ui::WorkspaceState, locale: StudioShellLocale) {
        if self.session.is_some() && !self.belongs_to(workspace) {
            let (open, mode) = (self.open, self.mode);
            *self = Self {
                open,
                mode,
                ..Default::default()
            };
            self.notice = Some(
                tr(
                    locale,
                    "工程已切换；旧选择与搜索已清理。",
                    "Project changed; the previous selection and search were cleared.",
                )
                .into(),
            );
        }
        self.session = Some(workspace.flow_run_qualification());
        self.cache.refresh(workspace, locale);
        self.cache
            .filter(&self.query, self.section, self.scope.as_ref());
    }
}

#[cfg(test)]
mod tests;
