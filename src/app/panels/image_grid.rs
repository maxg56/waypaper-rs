use egui::Context;

use crate::app::{LoadState, WaypaperApp};

impl WaypaperApp {
    pub(crate) fn show_image_grid(
        &mut self,
        ctx: &Context,
        state: &LoadState,
        visible: &[usize],
        loading: bool,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let cols = self.cf.number_of_columns;
                let available_width = ui.available_width();
                let thumb_size =
                    ((available_width - (cols as f32 - 1.0) * 4.0) / cols as f32).max(80.0);

                let chunks: Vec<&[usize]> = visible.chunks(cols).collect();
                for (row_idx, chunk) in chunks.iter().enumerate() {
                    ui.horizontal(|ui| {
                        for (col_idx, &img_idx) in chunk.iter().enumerate() {
                            let flat_idx = row_idx * cols + col_idx;
                            let path = &state.image_paths[img_idx];
                            let name = &state.image_names[img_idx];
                            let is_selected = flat_idx == self.selected_index;

                            let (rect, resp) = ui.allocate_exact_size(
                                egui::Vec2::splat(thumb_size),
                                egui::Sense::click(),
                            );

                            if is_selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    4.0,
                                    egui::Stroke::new(
                                        3.0,
                                        egui::Color32::from_rgb(100, 180, 255),
                                    ),
                                );
                            }

                            if let Some(tex) = self.get_or_load_texture(ctx, path) {
                                let uv = egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                );
                                ui.painter()
                                    .image(tex.id(), rect, uv, egui::Color32::WHITE);
                            } else {
                                ui.painter()
                                    .rect_filled(rect, 4.0, egui::Color32::from_gray(60));
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "?",
                                    egui::FontId::proportional(24.0),
                                    egui::Color32::GRAY,
                                );
                            }

                            if resp.hovered() {
                                egui::show_tooltip_text(
                                    ctx,
                                    ui.layer_id(),
                                    egui::Id::new(img_idx),
                                    name,
                                );
                            }

                            if resp.clicked() {
                                self.selected_index = flat_idx;
                                let path_clone = path.clone();
                                self.set_wallpaper(&path_clone);
                            }

                            ui.add_space(4.0);
                        }
                    });
                    ui.add_space(4.0);
                }

                if state.image_paths.is_empty() && !loading {
                    ui.centered_and_justified(|ui| {
                        ui.label("No images found. Choose a folder with the button above.");
                    });
                }
            });
        });
    }
}
