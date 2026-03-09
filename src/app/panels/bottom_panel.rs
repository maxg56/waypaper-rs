use egui::{Context, Ui};

use crate::app::helpers::{capitalize, color32_to_hex, hex_to_color32};
use crate::app::WaypaperApp;
use crate::options::{FILL_OPTIONS, SWWW_TRANSITION_TYPES, get_monitor_options};

impl WaypaperApp {
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
}
