use egui::Context;

use crate::app::WaypaperApp;

impl WaypaperApp {
    pub(crate) fn show_options_popup(&mut self, ctx: &Context) {
        if !self.show_options {
            return;
        }

        egui::Window::new("Options")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.checkbox(&mut self.cf.show_gifs_only, "GIFs only");
                ui.checkbox(&mut self.cf.include_subfolders, "Include subfolders");
                if ui
                    .checkbox(&mut self.cf.include_all_subfolders, "Include all subfolders")
                    .changed()
                    && self.cf.include_all_subfolders
                {
                    self.cf.include_subfolders = true;
                }
                ui.checkbox(&mut self.cf.show_hidden, "Show hidden files");
                ui.checkbox(&mut self.cf.show_path_in_tooltip, "Show path in tooltip");
                ui.checkbox(&mut self.cf.zen_mode, "Zen mode");

                ui.separator();
                ui.label("Carousel");
                let was_active = self.carousel_active;
                if ui.checkbox(&mut self.carousel_active, "Enable carousel").changed() {
                    if self.carousel_active {
                        self.carousel_index = 0;
                        self.carousel_next_change = Some(
                            std::time::Instant::now()
                                + std::time::Duration::from_secs(self.cf.carousel_interval),
                        );
                    } else {
                        self.carousel_next_change = None;
                    }
                }
                if self.carousel_active || was_active {
                    ui.horizontal(|ui| {
                        ui.label("Interval (s):");
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut self.carousel_interval_str)
                                    .desired_width(60.0),
                            )
                            .changed()
                        {
                            if let Ok(v) = self.carousel_interval_str.parse::<u64>() {
                                if v > 0 {
                                    self.cf.carousel_interval = v;
                                }
                            }
                        }
                    });
                    ui.checkbox(&mut self.cf.carousel_random, "Random order");
                }

                if ui.button("Apply & Close").clicked() {
                    self.cf.save();
                    self.start_loading();
                    self.show_options = false;
                }
            });
    }
}
