use gilgamesh_egui::egui::{Context, FontFamily, FontId, RawInput};

#[test]
fn default_fonts_cover_labels_and_numeric_controls() {
    let context = Context::default();
    let mut output = context.run_pass(RawInput::default(), |context| {
        context.fonts_mut(|fonts| {
            for family in [FontFamily::Proportional, FontFamily::Monospace] {
                let font = FontId::new(16.0, family);
                assert!(
                    fonts.has_glyphs(
                        &font,
                        "Scene controls Roughness Metallic Light (lux) 0123456789 éàç"
                    ),
                    "Missing bundled glyphs for {:?}",
                    font.family,
                );
            }
        });
    });
    output.textures_delta.clear();
}
