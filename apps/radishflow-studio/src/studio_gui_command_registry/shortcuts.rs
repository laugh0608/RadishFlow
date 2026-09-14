#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StudioGuiShortcutModifier {
    /// Platform command modifier: Command on macOS, Control elsewhere.
    Primary,
    /// Physical Control, including on macOS.
    Ctrl,
    Shift,
    Alt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StudioGuiShortcutKey {
    S,
    Z,
    Y,
    F5,
    F6,
    F8,
    Tab,
    Escape,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StudioGuiShortcut {
    pub modifiers: Vec<StudioGuiShortcutModifier>,
    pub key: StudioGuiShortcutKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiShortcutPlatform {
    MacOs,
    Windows,
    Linux,
}

impl StudioGuiShortcutPlatform {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    pub fn redo_shortcut(self) -> StudioGuiShortcut {
        if self == Self::MacOs {
            StudioGuiShortcut {
                modifiers: vec![
                    StudioGuiShortcutModifier::Primary,
                    StudioGuiShortcutModifier::Shift,
                ],
                key: StudioGuiShortcutKey::Z,
            }
        } else {
            StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Primary],
                key: StudioGuiShortcutKey::Y,
            }
        }
    }
}

impl StudioGuiShortcut {
    pub fn format(&self, platform: StudioGuiShortcutPlatform) -> String {
        let mac_command = platform == StudioGuiShortcutPlatform::MacOs
            && self.modifiers.contains(&StudioGuiShortcutModifier::Primary);
        let mut parts = Vec::new();
        // A fixed order keeps labels independent of modifier vector order.
        for modifier in [
            StudioGuiShortcutModifier::Ctrl,
            StudioGuiShortcutModifier::Alt,
            StudioGuiShortcutModifier::Shift,
            StudioGuiShortcutModifier::Primary,
        ] {
            if !self.modifiers.contains(&modifier) {
                continue;
            }
            parts.push(match modifier {
                StudioGuiShortcutModifier::Primary if mac_command => "⌘",
                StudioGuiShortcutModifier::Primary | StudioGuiShortcutModifier::Ctrl => "Ctrl",
                StudioGuiShortcutModifier::Shift if mac_command => "⇧",
                StudioGuiShortcutModifier::Alt if mac_command => "⌥",
                StudioGuiShortcutModifier::Shift => "Shift",
                StudioGuiShortcutModifier::Alt => "Alt",
            });
        }
        if !mac_command && self.modifiers.contains(&StudioGuiShortcutModifier::Primary) {
            // Control precedes Shift/Alt in Windows and Linux labels.
            parts.rotate_right(1);
        }
        parts.push(match self.key {
            StudioGuiShortcutKey::S => "S",
            StudioGuiShortcutKey::Z => "Z",
            StudioGuiShortcutKey::Y => "Y",
            StudioGuiShortcutKey::F5 => "F5",
            StudioGuiShortcutKey::F6 => "F6",
            StudioGuiShortcutKey::F8 => "F8",
            StudioGuiShortcutKey::Tab => "Tab",
            StudioGuiShortcutKey::Escape => "Escape",
        });
        parts.join(if mac_command { "" } else { "+" })
    }
}
