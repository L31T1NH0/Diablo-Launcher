use eframe::egui::{self, Color32, Vec2};
pub(crate) const GOLD: Color32 = Color32::from_rgb(225, 166, 92);
pub(crate) const PRIMARY: Color32 = Color32::from_rgb(151, 39, 48);
pub(crate) const PRIMARY_HOVER: Color32 = Color32::from_rgb(183, 48, 57);
pub(crate) const PRIMARY_ACTIVE: Color32 = Color32::from_rgb(119, 27, 36);
pub(crate) const BACKGROUND: Color32 = Color32::from_rgb(14, 15, 18);
pub(crate) const MUTED: Color32 = Color32::from_rgb(160, 154, 148);

pub(crate) fn configure(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.override_text_color = Some(Color32::from_rgb(230, 229, 226));
    visuals.window_fill = Color32::from_rgb(22, 23, 27);
    visuals.selection.bg_fill = Color32::from_rgb(76, 36, 43);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(225, 122, 126));
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.corner_radius = egui::CornerRadius::same(6);
        widget.fg_stroke.color = Color32::from_rgb(232, 230, 226);
    }
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(31, 33, 40);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(31, 33, 40);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(65, 65, 74));
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(47, 48, 57);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(47, 48, 57);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(150, 76, 83));
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(69, 37, 44);
    visuals.widgets.active.bg_fill = Color32::from_rgb(69, 37, 44);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(203, 100, 105));
    visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(45, 45, 49);
    ctx.set_visuals(visuals);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(12.0, 10.0);
        s.spacing.button_padding = Vec2::new(16.0, 10.0);
    });
}

pub(crate) fn load_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let image = image::load_from_memory(bytes)
        .expect("Invalid bundled image")
        .to_rgba8();
    ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        ),
        egui::TextureOptions::LINEAR,
    )
}
