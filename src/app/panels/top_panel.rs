use egui::{Context, Ui};

use crate::app::WaypaperApp;
use crate::options::SORT_DISPLAYS;

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
}
