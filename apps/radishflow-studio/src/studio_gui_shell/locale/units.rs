use super::StudioShellLocale;
use rf_types::units::QuantityKind;

#[derive(Clone, Copy)]
pub(in super::super) enum UnitSettingsText {
    Open,
    Title,
    Scope,
    FullSi,
    Engineering,
    Unapplied,
    Apply,
    HistoryScope,
    Undo,
    Redo,
    RestoreSaved,
    PersonalScope,
    SaveDefaults,
    RecoverDefaults,
    DefaultUnavailable,
    DefaultUnavailableHelp,
    UseSiOnce,
    RecoveryTitle,
    RecoveryHelp,
    BackupAndReplace,
    DefaultsSaved,
    DefaultsSavedHelp,
    DefaultsSaveFailed,
    DefaultsSaveFailedHelp,
    ApplyFailed,
    HistoryFailed,
    DefaultLoadFailed,
    DiagnosticDetails,
}

impl StudioShellLocale {
    pub(in super::super) fn unit_settings_text(self, key: UnitSettingsText) -> &'static str {
        use UnitSettingsText::*;
        let (zh, en) = match key {
            Open => ("项目显示单位…", "Project display units…"),
            Title => ("项目显示单位", "Project display units"),
            Scope => (
                "作用范围：当前工程，随工程文件保存。检查器可单独覆盖；当前输入保持本次输入单位，结果仍以自身标签为准。",
                "Applies to this project and is saved with it. Inspectors can override these units. Active inputs keep their input units; results use the units shown in their labels.",
            ),
            FullSi => ("完整 SI", "Full SI"),
            Engineering => ("工程集", "Engineering units"),
            Unapplied => (
                "当前选择尚未应用；应用后才进入显示设置保存状态。",
                "These choices are not applied yet. Apply them before saving the display settings.",
            ),
            Apply => ("应用显示设置", "Apply display settings"),
            HistoryScope => (
                "撤销 / 重做按项目与检查器共用的呈现历史执行；恢复已保存设置只影响项目。",
                "Undo and redo share the project and Inspector presentation history. Restore saved settings affects only the project.",
            ),
            Undo => ("撤销显示设置", "Undo display settings"),
            Redo => ("重做显示设置", "Redo display settings"),
            RestoreSaved => ("恢复已保存设置", "Restore saved settings"),
            PersonalScope => (
                "个人默认只采用已应用的项目选择；未应用的设置草稿不写入默认。",
                "Personal defaults use the applied project settings. Unapplied choices are not included.",
            ),
            SaveDefaults => ("设为新工程默认", "Use as defaults for new projects"),
            RecoverDefaults => ("恢复个人默认…", "Recover personal defaults…"),
            DefaultUnavailable => ("个人单位默认不可用", "Personal unit defaults unavailable"),
            DefaultUnavailableHelp => (
                "原默认文件保持不变。可明确选择完整 SI 创建此工程，或取消并在显示单位设置中恢复默认。",
                "The existing defaults file is unchanged. Create this project with full SI units, or cancel and recover the defaults in display unit settings.",
            ),
            UseSiOnce => ("本次使用 SI 新建", "Create this project with SI"),
            RecoveryTitle => ("恢复个人单位默认", "Recover personal unit defaults"),
            RecoveryHelp => (
                "将先备份原默认文件，再用当前已应用的项目单位集替换。备份或替换失败时保留可重试状态。",
                "Back up the existing defaults file, then replace it with the applied project units. If either step fails, the recovery can be retried.",
            ),
            BackupAndReplace => ("备份并替换默认", "Back up and replace defaults"),
            DefaultsSaved => ("个人默认已保存", "Personal defaults saved"),
            DefaultsSavedHelp => (
                "新工程默认已保存；现有工程保持原设置。",
                "Defaults saved for new projects. Existing projects keep their settings.",
            ),
            DefaultsSaveFailed => ("个人默认保存失败", "Could not save personal defaults"),
            DefaultsSaveFailedHelp => (
                "内存默认与工程设置保持不变，可重试。",
                "The defaults in memory and project settings are unchanged. You can retry.",
            ),
            ApplyFailed => ("显示设置未应用", "Display settings were not applied"),
            HistoryFailed => ("显示设置未修改", "Display settings were not changed"),
            DefaultLoadFailed => ("个人默认未加载", "Personal defaults were not loaded"),
            DiagnosticDetails => ("诊断详情", "Diagnostic details"),
        };
        match self {
            Self::ZhCn => zh,
            Self::En => en,
        }
    }

    pub(in super::super) fn quantity_name(self, quantity: QuantityKind) -> &'static str {
        let (zh, en) = match quantity {
            QuantityKind::AbsoluteTemperature => ("绝对温度", "Absolute temperature"),
            QuantityKind::TemperatureDifference => ("温差", "Temperature difference"),
            QuantityKind::AbsolutePressure => ("绝对压力", "Absolute pressure"),
            QuantityKind::MolarFlow => ("摩尔流量", "Molar flow"),
            QuantityKind::MoleFraction => ("摩尔分数", "Mole fraction"),
            QuantityKind::MolarPhaseFraction => ("相摩尔分率", "Molar phase fraction"),
            QuantityKind::MolarEnthalpy => ("摩尔焓", "Molar enthalpy"),
        };
        match self {
            Self::ZhCn => zh,
            Self::En => en,
        }
    }

    pub(in super::super) fn project_unit_selector_name(self, quantity: QuantityKind) -> String {
        let quantity = self.quantity_name(quantity);
        match self {
            Self::ZhCn => format!("项目显示单位：{quantity}"),
            Self::En => format!("Project display unit: {quantity}"),
        }
    }

    pub(in super::super) fn unit_defaults_recovered(self, backup: &std::path::Path) -> String {
        match self {
            Self::ZhCn => format!("新工程默认已恢复；原文件备份：{}", backup.display()),
            Self::En => format!(
                "Defaults restored for new projects. Previous file backed up to: {}",
                backup.display()
            ),
        }
    }
}
