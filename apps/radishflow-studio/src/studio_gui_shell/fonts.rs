use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;

const CJK_FONT_NAME: &str = "radishflow-system-cjk";

pub(super) fn configure_studio_fonts(ctx: &egui::Context) {
    let Some(font_bytes) = load_system_cjk_font_bytes() else {
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        CJK_FONT_NAME.to_string(),
        Arc::new(egui::FontData::from_owned(font_bytes)),
    );

    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push(CJK_FONT_NAME.to_string());
    }

    ctx.set_fonts(fonts);
}

fn load_system_cjk_font_bytes() -> Option<Vec<u8>> {
    system_cjk_font_candidates()
        .into_iter()
        .find_map(|path| std::fs::read(path).ok())
}

fn system_cjk_font_candidates() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let fonts_dir = std::env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:\\Windows"))
            .join("Fonts");

        [
            "msyh.ttc",
            "msyhl.ttc",
            "simhei.ttf",
            "simsun.ttc",
            "NotoSansCJK-Regular.ttc",
            "NotoSansCJKsc-Regular.otf",
        ]
        .into_iter()
        .map(|file_name| fonts_dir.join(file_name))
        .collect()
    }

    #[cfg(not(windows))]
    {
        [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJKsc-Regular.otf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/System/Library/Fonts/PingFang.ttc",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_cjk_font_candidates_include_platform_fonts() {
        let candidates = system_cjk_font_candidates();

        assert!(!candidates.is_empty());
        #[cfg(windows)]
        assert!(
            candidates
                .iter()
                .any(|path| path.file_name().and_then(|name| name.to_str()) == Some("msyh.ttc"))
        );
    }
}
