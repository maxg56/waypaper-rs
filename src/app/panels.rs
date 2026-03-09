use egui::{Context, Ui};

use crate::options::{FILL_OPTIONS, SORT_DISPLAYS, SWWW_TRANSITION_TYPES, get_monitor_options};

use super::{LoadState, WaypaperApp};
use super::helpers::{capitalize, color32_to_hex, hex_to_color32};

impl WaypaperApp {
    pub(crate) fn show_top_panel(&mut self, ctx: &Context) {
        if self.cf.zen_mode {
            return;
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.show_folder_button(ui);
                self.show_search_bar(ui);
                ui.separator();
                self.show_sort_combo(ui);
                self.show_action_buttons(ui, ctx);
            });
        });
    }

    /// Folder picker button
    fn show_folder_button(&mut self, ui: &mut Ui) {
        if ui.button("📁 Folder").clicked() {
            self.want_folder_dialog = true;
        }
    }

    /// Search input with clear button
    fn show_search_bar(&mut self, ui: &mut Ui) {
        ui.label("🔍");
        let resp = ui.text_edit_singleline(&mut self.search_query);
        if resp.changed() {
            self.selected_index = 0;
        }
        if ui.button("✕").clicked() {
            self.search_query.clear();
        }
    }

    /// Sort option combo box
    fn show_sort_combo(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_id_salt("sort")
            .selected_text(
                SORT_DISPLAYS
                    .iter()
                    .find(|(k, _)| *k == self.cf.sort_option.as_str())
                    .map(|(_, v)| *v)
                    .unwrap_or("Sort"),
            )
            .show_ui(ui, |ui| {
                for (key, label) in SORT_DISPLAYS {
                    if ui
                        .selectable_label(self.cf.sort_option == *key, *label)
                        .clicked()
                    {
                        self.cf.sort_option = key.to_string();
                        self.start_loading();
                    }
                }
            });
    }

    /// Refresh, Random, Options, Exit buttons
    fn show_action_buttons(&mut self, ui: &mut Ui, ctx: &Context) {
        if ui
            .button("⟳ Refresh")
            .on_hover_text("Clear cache and reload")
            .clicked()
        {
            let _ = std::fs::remove_dir_all(&self.cf.cache_dir);
            let _ = std::fs::create_dir_all(&self.cf.cache_dir);
            self.textures.clear();
            self.start_loading();
        }

        if ui
            .button("🎲 Random")
            .on_hover_text("Set a random wallpaper")
            .clicked()
        {
            self.set_random_wallpaper();
        }

        if ui.button("⚙ Options").clicked() {
            self.show_options = !self.show_options;
        }

        if ui.button("✕ Exit").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

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

    pub(crate) fn show_bottom_panel(&mut self, ctx: &Context, loading: bool) {
        if self.cf.zen_mode {
            return;
        }

        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.show_backend_selector(ui);
                self.show_monitor_selector(ui);
                self.show_fill_selector(ui);
                self.show_swww_options(ui);
                self.show_mpvpaper_controls(ui);
                self.show_columns_control(ui);
            });

            if loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Caching thumbnails...");
                });
            }
        });
    }

    /// Backend combo box selector
    fn show_backend_selector(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_id_salt("backend")
            .selected_text(&self.cf.backend)
            .show_ui(ui, |ui| {
                for b in &self.cf.installed_backends.clone() {
                    if ui.selectable_label(self.cf.backend == *b, b).clicked() {
                        self.cf.backend = b.clone();
                        self.cf.selected_monitor = "All".to_string();
                        self.monitor_options = get_monitor_options(&self.cf.backend);
                    }
                }
            });
    }

    /// Monitor selector — hidden for backends that don't support per-monitor
    fn show_monitor_selector(&mut self, ui: &mut Ui) {
        if ["feh", "wallutils", "none"].contains(&self.cf.backend.as_str()) {
            return;
        }
        egui::ComboBox::from_id_salt("monitor")
            .selected_text(&self.cf.selected_monitor)
            .show_ui(ui, |ui| {
                for m in &self.monitor_options.clone() {
                    if ui
                        .selectable_label(self.cf.selected_monitor == *m, m)
                        .clicked()
                    {
                        self.cf.selected_monitor = m.clone();
                    }
                }
            });
    }

    /// Fill mode selector + color picker — hidden for hyprpaper/none
    fn show_fill_selector(&mut self, ui: &mut Ui) {
        if ["hyprpaper", "none"].contains(&self.cf.backend.as_str()) {
            return;
        }
        egui::ComboBox::from_id_salt("fill")
            .selected_text(capitalize(&self.cf.fill_option))
            .show_ui(ui, |ui| {
                for f in FILL_OPTIONS {
                    if ui
                        .selectable_label(self.cf.fill_option == *f, capitalize(f))
                        .clicked()
                    {
                        self.cf.fill_option = f.to_string();
                    }
                }
            });

        let mut color = hex_to_color32(&self.cf.color);
        if ui.color_edit_button_srgba(&mut color).changed() {
            self.cf.color = color32_to_hex(color);
        }
    }

    /// Transition options for swww/awww backends
    fn show_swww_options(&mut self, ui: &mut Ui) {
        if !["swww", "awww"].contains(&self.cf.backend.as_str()) {
            return;
        }

        self.show_swww_transition_combo(ui);
        self.show_swww_text_fields(ui);
        self.sync_swww_fields();
    }

    /// Transition type combo box for swww/awww
    fn show_swww_transition_combo(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_id_salt("transition")
            .selected_text(&self.cf.swww_transition_type)
            .show_ui(ui, |ui| {
                for t in SWWW_TRANSITION_TYPES {
                    if ui
                        .selectable_label(self.cf.swww_transition_type == *t, *t)
                        .clicked()
                    {
                        self.cf.swww_transition_type = t.to_string();
                    }
                }
            });
    }

    /// Angle/steps/duration/fps text fields for swww/awww
    fn show_swww_text_fields(&mut self, ui: &mut Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.swww_angle_str)
                .desired_width(40.0)
                .hint_text("angle"),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.swww_steps_str)
                .desired_width(40.0)
                .hint_text("steps"),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.swww_duration_str)
                .desired_width(55.0)
                .hint_text("duration"),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.swww_fps_str)
                .desired_width(40.0)
                .hint_text("fps"),
        );
    }

    /// Parse swww text fields into config values
    fn sync_swww_fields(&mut self) {
        if let Ok(v) = self.swww_angle_str.parse::<u32>() {
            self.cf.swww_transition_angle = v;
        }
        if let Ok(v) = self.swww_steps_str.parse::<u32>() {
            self.cf.swww_transition_step = v;
        }
        if let Ok(v) = self.swww_duration_str.parse::<f32>() {
            self.cf.swww_transition_duration = v;
        }
        if let Ok(v) = self.swww_fps_str.parse::<u32>() {
            self.cf.swww_transition_fps = v;
        }
    }

    /// Mpvpaper pause/stop/sound controls
    fn show_mpvpaper_controls(&mut self, ui: &mut Ui) {
        if self.cf.backend != "mpvpaper" {
            return;
        }

        if ui.button("⏸ Pause").clicked() {
            self.mpvpaper_send_command("cycle pause");
        }
        if ui.button("⏹ Stop").clicked() {
            let _ = std::process::Command::new("killall")
                .arg("mpvpaper")
                .spawn();
        }
        if ui
            .checkbox(&mut self.cf.mpvpaper_sound, "Sound")
            .changed()
        {
            self.mpvpaper_send_command("cycle mute");
        }
    }

    /// Send a command to mpvpaper via its IPC socket
    fn mpvpaper_send_command(&self, command: &str) {
        let monitor = &self.cf.selected_monitor;
        let cmd = format!("echo '{command}' | socat - /tmp/mpv-socket-{monitor}");
        let _ = std::process::Command::new("sh")
            .args(["-c", &cmd])
            .spawn();
    }

    /// Column count drag control
    fn show_columns_control(&mut self, ui: &mut Ui) {
        ui.separator();
        ui.label("Cols:");
        let mut cols = self.cf.number_of_columns as i32;
        if ui
            .add(egui::DragValue::new(&mut cols).range(1..=10))
            .changed()
        {
            self.cf.number_of_columns = cols as usize;
        }
    }

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
