use std::sync::Arc;

use eframe::egui;

const UI_LATIN_FONT_NAME: &str = "radishflow-ui-latin";
const UI_CJK_FONT_NAME: &str = "radishflow-ui-cjk";

const UI_LATIN_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/InterVariable.ttf");
const UI_CJK_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/SourceHanSansSC-Regular.otf");

pub(super) fn configure_studio_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        UI_LATIN_FONT_NAME.to_string(),
        Arc::new(egui::FontData::from_owned(UI_LATIN_FONT_BYTES.to_vec())),
    );
    fonts.font_data.insert(
        UI_CJK_FONT_NAME.to_string(),
        Arc::new(egui::FontData::from_owned(UI_CJK_FONT_BYTES.to_vec())),
    );

    let proportional_fonts = fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default();
    proportional_fonts.insert(0, UI_CJK_FONT_NAME.to_string());
    proportional_fonts.insert(0, UI_LATIN_FONT_NAME.to_string());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(UI_CJK_FONT_NAME.to_string());

    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_font_assets_are_available() {
        assert!(UI_LATIN_FONT_BYTES.len() > 512 * 1024);
        assert!(UI_CJK_FONT_BYTES.len() > 8 * 1024 * 1024);
    }
}
